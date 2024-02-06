#[allow(unused)]
mod test_timeout_executor {
    use std::{
        collections::{HashMap, HashSet},
        io,
        io::Write,
        panic,
        sync::{mpsc::channel, Arc, Mutex},
        thread,
        time::{Duration, Instant},
    };

    use anyhow::Result;
    use rand::Rng;

    use crate::{
        executor::{timeout::TimeoutExecutor, Executor},
        task::{
            event::TaskEvent,
            reporter::{
                file_reporter_init, report_event, report_event_details, report_event_short_message,
                std_reporter_init,
            },
            FinishedTask, Task, TaskInfo, TaskStatus,
        },
    };

    /// Prepare the expected events with stdout in the unit tests.
    fn generate_task_events_with_finished_stdout<T>(tasks: Vec<T>, stdout: String) -> Vec<TaskEvent>
    where
        T: Task + Clone,
    {
        let wait_tasks = tasks.clone();
        let mut res = vec![];
        let mut wait_events: Vec<TaskEvent> = wait_tasks
            .into_iter()
            .map(|t| TaskEvent::wait(t.info()))
            .collect();

        let finished_tasks = tasks;
        let mut finished_events: Vec<TaskEvent> = finished_tasks
            .into_iter()
            .map(|t| {
                TaskEvent::finished(
                    t.info(),
                    FinishedTask::new(
                        t.info(),
                        stdout.as_bytes().to_vec(),
                        vec![],
                        TaskStatus::Finished,
                    ),
                )
            })
            .collect();

        res.append(&mut wait_events);
        res.append(&mut finished_events);
        res
    }

    /// Collect events triggered during task execution.
    fn capture_events(event: TaskEvent, out: &mut Arc<Mutex<EventsCollector>>) -> Result<()> {
        writeln!(out.lock().unwrap(), "{}", event);
        Ok(())
    }

    #[derive(Clone)]
    /// Custom [`Task`] for testing
    struct MyTask {
        id: usize,
    }

    impl Task for MyTask {
        fn run(&self, ch: std::sync::mpsc::Sender<FinishedTask>) {
            ch.send(FinishedTask::new(
                TaskInfo::new(self.id.into(), "Task".to_string().into()),
                "Hello World".to_string().as_bytes().to_vec(),
                vec![],
                TaskStatus::Finished,
            ))
            .unwrap();
        }

        fn info(&self) -> TaskInfo {
            TaskInfo::new(self.id.into(), "Task".to_string().into())
        }
    }

    #[derive(Default)]
    /// [`EventsCollector`] used to collected [`TaskEvent`]s for testing.
    struct EventsCollector {
        pub(crate) events_str: String,
    }

    impl EventsCollector {
        pub fn clean(&mut self) -> &mut Self {
            self.events_str = String::new();
            self
        }
    }

    impl Write for EventsCollector {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if let Ok(s) = std::str::from_utf8(buf) {
                self.events_str.push_str(s)
            }
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    const NEW_LINE: &str = "\n";
    #[cfg(target_os = "windows")]
    const NEW_LINE: &'static str = "\r\n";

    fn run_my_task_with_thread_num(task_count: usize, thread_count: usize) {
        let mut tasks = vec![];

        for i in 0..task_count {
            tasks.push(MyTask { id: i })
        }

        let executor = TimeoutExecutor::new_with_thread_count_and_timeout(
            thread_count,
            Instant::now() + Duration::from_secs(120),
        );

        let mut events_collector = Arc::new(Mutex::new(EventsCollector::default()));

        let expected_events =
            generate_task_events_with_finished_stdout(tasks.clone(), "Hello World".to_string());

        expected_events.into_iter().for_each(|e| {
            capture_events(e, &mut Arc::clone(&events_collector));
        });

        let mut expected_events_strs: Vec<String> = events_collector
            .lock()
            .unwrap()
            .events_str
            .clone()
            .split(NEW_LINE)
            .map(|s| s.to_string())
            .collect();

        events_collector.lock().unwrap().clean();

        executor
            .run_all_tasks(&tasks, |e| {
                capture_events(e, &mut Arc::clone(&events_collector))
            })
            .unwrap();

        let mut got_events_strs: Vec<String> = events_collector
            .lock()
            .unwrap()
            .events_str
            .clone()
            .split(NEW_LINE)
            .map(|s| s.to_string())
            .collect();

        got_events_strs.sort();
        expected_events_strs.sort();
        assert_eq!(got_events_strs, expected_events_strs);
    }

    #[test]
    /// Run for 1 minute with a random number (0 to 100000)  of threads and tasks
    fn test_tasks_executor() {
        let start_time = Instant::now();

        loop {
            let random_thread_number = rand::thread_rng().gen_range(1..=100000);
            let random_task_number = rand::thread_rng().gen_range(1..=100000);

            run_my_task_with_thread_num(random_task_number, random_thread_number);

            if Instant::now().duration_since(start_time) > Duration::from_secs(60) {
                break;
            }
        }
    }

    #[test]
    fn test_tasks_executor_with_zero_thread() {
        let result: Result<(), Box<dyn std::any::Any + Send>> = std::panic::catch_unwind(|| {
            run_my_task_with_thread_num(1, 0);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_tasks_executor_with_zero_task() {
        run_my_task_with_thread_num(0, 1);
    }

    #[derive(Clone)]
    /// Custom [`Task`] for testing,
    /// [`OnlyPanicTask`] will do nothing in addition to panic.
    struct OnlyPanicTask {
        id: usize,
    }

    impl Task for OnlyPanicTask {
        /// Only panic.
        fn run(&self, ch: std::sync::mpsc::Sender<FinishedTask>) {
            panic!("This Task Panic.")
        }

        fn info(&self) -> TaskInfo {
            TaskInfo::new(self.id.into(), "PanicTask".to_string().into())
        }
    }

    #[test]
    /// If the task fails and returns nothing,
    /// it will wait for the task to complete,
    /// and a timeout message will be printed if the wait times out.
    fn test_panic_tasks_executor() {
        let mut tasks = vec![OnlyPanicTask { id: 0 }];

        let executor = TimeoutExecutor::new_with_thread_count(2);
        let mut events_collector = Arc::new(Mutex::new(EventsCollector::default()));

        events_collector.lock().unwrap().clean();

        let (tx, rx) = channel::<String>();
        let events_collector_tmp = Arc::clone(&events_collector);
        let handle = thread::spawn(move || {
            executor
                .run_all_tasks(&tasks, |e| {
                    capture_events(e, &mut Arc::clone(&events_collector_tmp))
                })
                .unwrap();
            tx.send("Unreachable Code".to_string());
        });

        let timeout = Duration::from_secs(70);
        match rx.recv_timeout(timeout) {
            Ok(_) => {
                panic!("unreachable code");
            }
            Err(_) => {
                assert_eq!(events_collector
                            .lock()
                            .unwrap()
                            .events_str,
                        "tname:PanicTask tid:0 event:waiting\ntname:PanicTask tid:0 event:timeout 59s\n");
                handle.thread().unpark();
            }
        }
    }

    #[derive(Clone)]
    /// Custom [`Task`] for testing,
    /// [`PanicAfterReturnTask`] will panic after return result.
    struct PanicAfterReturnTask {
        id: usize,
    }

    impl Task for PanicAfterReturnTask {
        /// panic and return.
        fn run(&self, ch: std::sync::mpsc::Sender<FinishedTask>) {
            ch.send(FinishedTask::new(
                TaskInfo::new(self.id.into(), "PanicAfterReturnTask".to_string().into()),
                "Hello World".to_string().as_bytes().to_vec(),
                vec![],
                TaskStatus::Finished,
            ))
            .unwrap();
            panic!("This task panic after return result.")
        }

        fn info(&self) -> TaskInfo {
            TaskInfo::new(self.id.into(), "PanicAfterReturnTask".to_string().into())
        }
    }

    #[test]
    /// If the task is done, but the thread panics after getting the task done,
    /// the [`run_all_tasks`] will return an [`io::Error`].
    fn test_panic_after_return_tasks_executor() {
        let mut tasks = vec![PanicAfterReturnTask { id: 0 }];
        let executor = TimeoutExecutor::new_with_thread_count(2);
        let mut events_collector = Arc::new(Mutex::new(EventsCollector::default()));
        tasks
            .clone()
            .into_iter()
            .map(|t| TaskEvent::wait(t.info()))
            .for_each(|e| {
                capture_events(e, &mut Arc::clone(&events_collector));
            });

        let mut expected_events_strs: Vec<String> = events_collector
            .lock()
            .unwrap()
            .events_str
            .clone()
            .split(NEW_LINE)
            .map(|s| s.to_string())
            .collect();

        events_collector.lock().unwrap().clean();

        let result: Result<Result<(), anyhow::Error>, Box<dyn std::any::Any + Send>> =
            std::panic::catch_unwind(|| {
                executor.run_all_tasks(&tasks, |e| {
                    capture_events(e, &mut Arc::clone(&events_collector))
                })
            });

        let mut got_events_strs: Vec<String> = events_collector
            .lock()
            .unwrap()
            .events_str
            .clone()
            .split(NEW_LINE)
            .map(|s| s.to_string())
            .collect();

        got_events_strs.sort();
        expected_events_strs.sort();
        assert_eq!(got_events_strs, expected_events_strs);

        match result {
            Ok(res) => match res {
                Ok(_) => {
                    panic!("unreachable code");
                }
                Err(err) => {
                    assert_eq!(
                        format!("{}", err),
                        format!(
                            "The task {} has completed, but the thread has failed",
                            PanicAfterReturnTask { id: 0 }.info()
                        )
                    );
                }
            },
            Err(_) => {
                panic!("unreachable code");
            }
        }
    }
}
