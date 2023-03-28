//! This file provides method to report the task execution logs.
use std::sync::Once;

use crate::task::{
    event::{TaskEvent, TaskEventType},
    TaskStatus,
};
use anyhow::Result;
use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use log::{error, info, warn};

/// [`ReporterConfig`] is mainly responsible for the configuration of log display.
pub struct ReporterConfig {
    level_filter: log::LevelFilter,
    stdout: bool,
    stderr: bool,
    file_path: Option<String>,
}

impl ReporterConfig {
    /// New a default [`ReporterConfig`].
    ///
    /// Note: Since the default [`ReporterConfig`] does not define the level and destination of the log,
    /// it will display nothing if you only use the default configuration.
    pub fn default() -> Self {
        ReporterConfig {
            level_filter: log::LevelFilter::Off,
            stdout: false,
            stderr: false,
            file_path: None,
        }
    }

    /// Set the [`log::LevelFilter`] for the log.
    pub fn filter(mut self, level_filter: log::LevelFilter) -> Self {
        self.level_filter = level_filter;
        self
    }

    /// Set the destination flag [std::io::Stdout].
    /// If [`true`], the log will display to [std::io::Stdout].
    pub fn stdout(mut self, is_stdout: bool) -> Self {
        self.stdout = is_stdout;
        self
    }

    /// Set the destination flag [std::io::Stderr].
    /// If [`true`], the log will display to [std::io::Stderr].
    pub fn stderr(mut self, is_stderr: bool) -> Self {
        self.stdout = is_stderr;
        self
    }

    /// Set the destination flag to a file.
    /// The log will be displaied to a file in path [`file_path`].
    /// If [`file_path`] doesn't exist,
    /// the [`init_reporter_once`] method will throw an error.
    pub fn file(mut self, file_path: &str) -> Self {
        self.file_path = Some(file_path.to_string());
        self
    }
}

static TIME_PATTERN: &'static str = "[%Y-%m-%d][%H:%M:%S]";

/// Set the color config for the log in stdout/stderr.
fn set_reporter_color() -> ColoredLevelConfig {
    ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .debug(Color::Blue)
        .info(Color::Green)
        .trace(Color::Black)
}

/// Return the [`Dispatch`] logger, based on the [`ReporterConfig`].
fn set_reporter_conf(conf: &ReporterConfig) -> Result<Dispatch> {
    let mut dispatch = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format(TIME_PATTERN),
                record.target(),
                set_reporter_color().color(record.level()),
                message
            ))
        })
        .level(conf.level_filter);

    if conf.stdout {
        dispatch = dispatch.chain(std::io::stdout());
    }

    if conf.stderr {
        dispatch = dispatch.chain(std::io::stderr());
    }

    if let Some(file) = &conf.file_path {
        dispatch = dispatch
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format(TIME_PATTERN),
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .chain(fern::log_file(file)?);
    }
    Ok(dispatch)
}

/// Generate the [`Dispatch`] logger and apply it.
///
/// Note: The [`init_reporter`] method can be called only once,
/// so it should be run as early in the program as possible.
///
/// # Errors:
///
/// 1. This function will return an error if a global logger has already been
/// set to a previous logger.
///
/// 2. This function will return an error if the path to the file in [`ReporterConfig`]
/// used for logging does not exist.
fn init_reporter(conf: &ReporterConfig) -> Result<()> {
    set_reporter_conf(conf)?.apply()?;
    Ok(())
}

/// Generate the [`Dispatch`] logger and apply it.
///
/// Note: The [`init_reporter_once`] method can be called only once,
/// so it should be run as early in the program as possible.
///
/// # Errors:
///
/// This function will return an error if the path to the file in [`ReporterConfig`]
/// used for logging does not exist.
///
/// # Note:
///
/// Only the [`ReporterConfig`] used in the first call to the [`init_reporter_once`] is taken.
pub fn init_reporter_once(conf: &ReporterConfig) -> Result<()> {
    static INIT: Once = Once::new();
    let mut result = Ok(());
    INIT.call_once(|| result = init_reporter(conf));
    result
}

/// Initialize a file reporter with the default configuration,
/// the log will be output to a [`log_file`].
pub fn file_reporter_init(log_file: &str) -> Result<()> {
    let conf = ReporterConfig::default()
        .filter(log::LevelFilter::Debug)
        .stdout(true)
        .stderr(true)
        .file(&log_file);
    init_reporter_once(&conf)
}

/// Initialize a stdout/stderr reporter with the default configuration,
/// the log will be output to stdout/stderr.
pub fn std_reporter_init(stdout: bool, stderr: bool) -> Result<()> {
    let conf = ReporterConfig::default()
        .filter(log::LevelFilter::Debug)
        .stdout(stdout)
        .stderr(stderr);
    init_reporter_once(&conf)
}

/// Display short logs based on the received [`TaskEvent`].
/// Based on the fecade implementation provided by [`log`].
/// If you want to change the logging engine, you just need to implement a new [`init_reporter_once`].
///
/// # Note:
///
/// Before you use this method to log, make sure that [`init_reporter_once`] is called once to initialize the logging engine.
pub fn report_event_short_message(event: TaskEvent) -> Result<()> {
    match event.ty() {
        TaskEventType::Start => {
            info!(target: &format!("{}", event.tinfo().tname()), "start")
        }
        TaskEventType::Wait => {
            info!(target: &format!("{}", event.tinfo().tname()), "waiting")
        }
        TaskEventType::Timeout(t) => {
            warn!(
                target: &format!("{}", event.tinfo().tname()),
                "It's been running for over {} seconds",
                t.as_secs()
            )
        }
        TaskEventType::Finished(fin_res) => match fin_res.status() {
            TaskStatus::Finished | TaskStatus::Waiting => {
                info!(
                    target: &format!("{}", event.tinfo().tname()),
                    "{}",
                    fin_res.status()
                )
            }
            TaskStatus::Failed(_) | TaskStatus::Bug(_) => {
                error!(
                    target: &format!("{}", event.tinfo().tname()),
                    "{}",
                    fin_res.status()
                )
            }
        },
    }
    Ok(())
}

/// Display detailed logs based on the received [`TaskEvent`].
/// Based on the fecade implementation provided by [`log`].
/// If you want to change the logging engine, you just need to implement a new [`init_reporter_once`].
///
/// # Note:
///
/// Before you use this method to log, make sure that [`init_reporter_once`] is called once to initialize the logging engine.
pub fn report_event_details(event: TaskEvent) -> Result<()> {
    match event.ty() {
        TaskEventType::Start => {
            info!(
                target: &format!("{}", event.tinfo().tname()),
                "\n{} start",
                event.tinfo()
            )
        }
        TaskEventType::Wait => {
            info!(
                target: &format!("{}", event.tinfo().tname()),
                "\n{} waiting",
                event.tinfo()
            )
        }
        TaskEventType::Timeout(t) => {
            warn!(
                target: &format!("{}", event.tinfo().tname()),
                "It's been running for over {} seconds",
                t.as_secs()
            )
        }
        TaskEventType::Finished(fin_res) => match fin_res.status() {
            TaskStatus::Finished | TaskStatus::Waiting => {
                info!(
                    target: &format!("{}", event.tinfo().tname()),
                    "\n{}", fin_res
                )
            }
            TaskStatus::Failed(_) | TaskStatus::Bug(_) => {
                error!(
                    target: &format!("{}", event.tinfo().tname()),
                    "\n{}", fin_res
                )
            }
        },
    }
    Ok(())
}

/// Display logs based on the received [`TaskEvent`].
/// Short messages will be output if the task executed normally,
/// and detailed messages will be output if the task failed.
pub fn report_event(event: TaskEvent) -> Result<()> {
    match event.ty() {
        TaskEventType::Start | TaskEventType::Wait | TaskEventType::Timeout(_) => {
            report_event_short_message(event)?;
        }
        TaskEventType::Finished(fin_res) => match fin_res.status() {
            TaskStatus::Finished | TaskStatus::Waiting => {
                report_event_short_message(event)?;
            }
            TaskStatus::Failed(_) | TaskStatus::Bug(_) => {
                report_event_details(event)?;
            }
        },
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::{
        fs::{self, File},
        io::BufReader,
        path::Path,
    };

    use crate::task::{
        event::TaskEvent,
        reporter::{report_event_short_message, ReporterConfig},
    };

    use super::{init_reporter_once, report_event_details, set_reporter_conf};
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    pub fn report_event_file<F>(name: &str, report_func: &F)
    where
        F: Fn(TaskEvent) -> Result<()>,
    {
        let event_path = Path::new(EVENT_PATH).join(name);
        let event_json = event_path.join(format!("{}.json", name));

        let event: TaskEvent =
            serde_json::from_reader(BufReader::new(File::open(event_json).unwrap())).unwrap();

        report_func(event).unwrap();
    }

    pub fn expect_event_report(name: &str, expect_file_name: &str) -> String {
        let event_path = Path::new(EVENT_PATH).join(name);
        let expect_path = event_path.join(expect_file_name);
        format!(
            "{}{}",
            chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
            fs::read_to_string(&expect_path.display().to_string())
                .expect("Something went wrong reading the file")
        )
    }

    const EVENT_PATH: &str = "./src/task/test_datas/test_event_reporter";
    const EVENTS: [&'static str; 7] = [
        "start_event",
        "wait_event",
        "timeout_event",
        "finished_event",
        "finished_event_waiting",
        "finished_event_failed",
        "finished_event_bug",
    ];

    fn test_event_reporter<F>(got: String, expect_short_msg: String, report_func: F)
    where
        F: Fn(TaskEvent) -> Result<()>,
    {
        let event_path = Path::new(EVENT_PATH);
        let output_path = event_path.join(got.to_string());
        let file = File::create(output_path.clone()).expect("Failed to create file");
        file.set_len(0).expect("Failed to clear file");

        let conf = ReporterConfig::default()
            .filter(log::LevelFilter::Debug)
            .file(&output_path.display().to_string());
        init_reporter_once(&conf).unwrap();

        let mut expected_result = String::new();
        for event in EVENTS {
            report_event_file(event, &report_func);
            expected_result.push_str(&format!(
                "{}",
                expect_event_report(event, &expect_short_msg)
            ));
        }
        expected_result.push_str("\n");
        let got_result = fs::read_to_string(&output_path.display().to_string())
            .expect("Something went wrong reading the file");

        assert_eq!(got_result, expected_result);
    }

    fn test_file_not_exist() {
        let conf = ReporterConfig::default()
            .filter(log::LevelFilter::Debug)
            .file("./not_exist_dir/not_exist");
        match set_reporter_conf(&conf) {
            Ok(_) => {
                panic!("Unreachable Code")
            }
            Err(err) => {
                assert_eq!("No such file or directory (os error 2)", format!("{}", err))
            }
        };
    }

    #[test]
    /// Since [`init_reporter_once`] can only be called once,
    /// the test cases must be executed in a certain order.
    fn test_in_order() {
        let event_path = Path::new(EVENT_PATH);
        let output_path = event_path.join("output");
        let file = File::create(output_path.clone()).expect("Failed to create file");
        file.set_len(0).expect("Failed to clear file");
        let conf = ReporterConfig::default()
            .filter(log::LevelFilter::Debug)
            .file(&output_path.display().to_string());

        init_reporter_once(&conf).unwrap();

        test_file_not_exist();
        test_event_reporter(
            "output".to_string(),
            "output_short".to_string(),
            report_event_short_message,
        );
        test_event_reporter(
            "output".to_string(),
            "output_detail".to_string(),
            report_event_details,
        );
    }
}
