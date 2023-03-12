//! This file provides everything to define a [`Task`] that an [`crate::executor::Executor`] can execute.
use std::{fmt, sync::mpsc::Sender, thread};

use serde::{Deserialize, Serialize};
pub mod event;
pub mod reporter;

/// [`Task`] is the unit that [`crate::executor::Executor`] can execute concurrently.
///
/// # Example
///
/// ```rust
/// use compiler_base_parallel::task::Task;
/// use compiler_base_parallel::task::FinishedTask;
/// use std::sync::mpsc::channel;
/// use std::sync::mpsc::Sender;
/// use compiler_base_parallel::task::TaskName;
/// use compiler_base_parallel::task::TaskId;
/// use compiler_base_parallel::task::TaskInfo;
/// use compiler_base_parallel::task::TaskStatus;
///
/// // 1. Define a custom task [`MyTask`].
/// struct MyTask {
///     id: usize,
///     name: String,
/// }
///
/// // 2. Implement trait [`Task`] for [`MyTask`].
/// impl Task for MyTask {
///     fn run(&self, ch: Sender<FinishedTask>) {
///         // [`FinishedTask`] is constructed here passed to other threads via [`ch`].
///         let res = FinishedTask::new(self.info(), vec![], vec![], TaskStatus::Finished);
///         ch.send(res).unwrap();
///     }
///
///     fn info(&self) -> TaskInfo {
///         TaskInfo::new(self.id.into(), self.name.to_string().into())
///     }
/// }
///
/// impl MyTask {
///     pub fn new(id: usize, name: String) -> Self {
///         Self { id, name }
///    }
/// }
///
/// // 3. Create [`channel`] to pass [`FinishedTask`].
/// let (tx, rx) = channel::<FinishedTask>();
/// let my_task = MyTask::new(0, "MyTask 0".to_string());
/// my_task.run(tx);
///
/// // 4. [`FinishedTask`] created in [`Task`] will be got from channel.
/// match rx.recv() {
///     Ok(res) => {
///         assert_eq!(res.tinfo().tid(), 0.into());
///         assert_eq!(res.tinfo().tname(), "MyTask 0".to_string().into());
///         assert_eq!(res.status(), TaskStatus::Finished);
///     },
///     Err(_) => panic!("unreachable code")
/// }
/// ```
pub trait Task {
    /// [`run`] will be executed of the [`Task`],
    /// and the result of the execution is communicated with other threads through the [`ch`] which is a [`Sender<FinishedTask>`],
    /// so [`run`] method does not need to return a value.
    ///
    /// Note: If the [`run`] method panics before returning the result through the [`ch`],
    /// nothing will be output, and the outside world will not be able to get the running status of the task.
    /// Therefore, when implementing the [`run`] method,
    /// please try to handle the failure case as much as possible to ensure that all result can be sent to [`ch`].
    ///
    /// If you can not get the results properly and you are confident that all the possible results are returned through [`ch`],
    /// please contact us, this maybe a bug.
    fn run(&self, ch: Sender<FinishedTask>);

    /// Return the [`TaskInfo`]
    fn info(&self) -> TaskInfo;
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TaskInfo {
    tid: TaskId,
    tname: TaskName,
}

impl TaskInfo {
    pub fn new(tid: TaskId, tname: TaskName) -> Self {
        Self { tid, tname }
    }

    pub fn tid(&self) -> TaskId {
        self.tid
    }

    pub fn tname(&self) -> TaskName {
        self.tname.clone()
    }
}

impl From<TaskInfo> for String {
    fn from(info: TaskInfo) -> Self {
        format!("{}", info)
    }
}

impl std::fmt::Display for TaskInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tname:{} tid:{}", self.tname(), self.tid())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
/// The ID for the [`Task`].
/// [`TaskId`] will be used as the key of the [`HashMap`], [`TaskId`] is a type alias of [`usize`].
/// so [`TaskID`] should be unique to each [`Task`].
pub struct TaskId(usize);

impl From<usize> for TaskId {
    fn from(u: usize) -> Self {
        TaskId(u)
    }
}

impl TaskId {
    /// New a [`TaskId`]
    pub fn new(id: usize) -> Self {
        TaskId(id)
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
/// The name for the [`Task`].
/// [`TaskName`] will be used to log displaying, [`TaskName`] is a type alias of [`String`].
pub struct TaskName(String);

impl From<String> for TaskName {
    fn from(s: String) -> Self {
        TaskName(s)
    }
}

impl std::fmt::Display for TaskName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<TaskName> for String {
    fn from(task_name: TaskName) -> Self {
        task_name.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
/// [`TaskStatus`] is the execution status of [`Task`] and is part of the result returned.
/// At present, it mainly includes three parts:
/// - [`TaskStatus::Finished`]: The [`Task`] has been finished.
/// - [`TaskStatus::Waiting`]: The [`Task`] is running or waiting and can not get the results.
/// - [`TaskStatus::Failed(String)`]: The failure status contains a String argument that holds some information about the exception.
/// - [`TaskStatus::Bug(String)`]: Bug means that the failure of the [`Task`] is caused by a bug.
pub enum TaskStatus {
    /// The [`Task`] has been finished.
    Finished,
    /// The [`Task`] is running or waiting and can not get the results.
    Waiting,
    /// The [`Task`] is failed, and this status contains a [`String`] argument that holds some information about the exception.
    Failed(String),
    /// Bug means that the failure of the [`Task`] is caused by a bug.
    Bug(String),
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskStatus::Finished => {
                write!(f, "{}", "finished")
            }
            TaskStatus::Waiting => {
                write!(f, "{}", "waiting")
            }
            TaskStatus::Failed(reason) => {
                write!(f, "{}:{}", "failed", reason)
            }
            TaskStatus::Bug(reason) => {
                write!(f, "{}:{}", "bug", reason)
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
/// [`FinishedTask`] represents the execution result of the [`Task`].
pub struct FinishedTask {
    tinfo: TaskInfo,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status: TaskStatus,
}

impl PartialEq for FinishedTask {
    fn eq(&self, other: &Self) -> bool {
        self.tinfo == other.tinfo
            && self.stdout == other.stdout
            && self.stderr == other.stderr
            && self.status == other.status
    }
}

impl std::fmt::Display for FinishedTask {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} finished\nstdout:\n{}\nstderr:\n{}\nstatus:{}",
            self.tinfo(),
            String::from_utf8_lossy(&self.stdout()),
            String::from_utf8_lossy(&self.stderr()),
            self.status
        )
    }
}

impl FinishedTask {
    /// New a [`FinishedTask`]
    pub fn new(tinfo: TaskInfo, stdout: Vec<u8>, stderr: Vec<u8>, status: TaskStatus) -> Self {
        Self {
            tinfo,
            stdout,
            stderr,
            status,
        }
    }

    /// Get [`TaskInfo`]
    pub fn tinfo(&self) -> TaskInfo {
        self.tinfo.clone()
    }

    /// Get [`TaskStatus`]
    pub fn status(&self) -> TaskStatus {
        self.status.clone()
    }

    /// Get the stdout for the [`Task`] in [`Vec<u8>`].
    pub fn stdout(&self) -> Vec<u8> {
        self.stdout.clone()
    }

    /// Get the stderr for the [`Task`] in [`Vec<u8>`].
    pub fn stderr(&self) -> Vec<u8> {
        self.stderr.clone()
    }

    /// Find a bug and set status.
    pub fn find_bug(&mut self, info: String) {
        self.status = TaskStatus::Bug(info)
    }

    /// Display the short message for [`FinishedTask`].
    pub fn short_msg(&self) -> String {
        format!("{} finished, status:{}\n", self.tinfo(), self.status)
    }

    /// Display the detailed message for [`FinishedTask`].
    pub fn details(&self) -> String {
        format!(
            "\n{} finished\nstdout:{}\nstderr:{}\nstatus:{}\n",
            self.tinfo(),
            String::from_utf8_lossy(&self.stdout()),
            String::from_utf8_lossy(&self.stderr()),
            self.status
        )
    }
}

/// [`RunningTask`] is an internal structure to manage threads after startup.
/// It contains only one member, whose type is the [`std::thread::JoinHandle`] returned by [`std::thread::spawn`].
/// Once [`Task`] has been loaded into the thread and started,
/// [`crate::executor::Executor`] controls the running threads by [`RunningTask`].
pub(crate) struct RunningTask {
    pub(crate) join_handle: Option<thread::JoinHandle<()>>,
}

impl RunningTask {
    /// Call the [`join`] and wait for the associated thread to finish.
    pub(crate) fn join(self, task: &mut FinishedTask) {
        if let Some(join_handle) = self.join_handle {
            // If [`Task`] returns [`TaskStatus::Finished`], that means [`Task`] is running correctly,
            // but the thread executing the [`Task`] returns an error, that means there's a bug.
            if join_handle.join().is_err() {
                if let TaskStatus::Finished = task.status() {
                    task.find_bug(format!(
                        "Exception occurs after task '{}' reporting success",
                        task.tinfo()
                    ));
                }
            }
        }
    }
}

#[allow(unused)]
mod test {
    use std::{
        sync::mpsc::{channel, Sender},
        thread,
    };

    use crate::task::RunningTask;

    use super::{FinishedTask, Task, TaskInfo, TaskStatus};

    // 1. Define a custom task [`MyTask`] for test.
    struct MyTask {
        id: usize,
        name: String,
    }

    impl MyTask {
        pub fn new(id: usize, name: String) -> Self {
            Self { id, name }
        }
    }
    // 2. Implement trait [`Task`] for [`MyTask`].
    impl Task for MyTask {
        fn run(&self, ch: Sender<super::FinishedTask>) {
            // [`FinishedTask`] is constructed here passed to other threads via [`ch`].
            ch.send(FinishedTask::new(
                TaskInfo::new(self.id.into(), self.name.clone().into()),
                vec![],
                vec![],
                TaskStatus::Finished,
            ))
            .unwrap();
        }

        fn info(&self) -> TaskInfo {
            TaskInfo::new(self.id.into(), self.name.clone().into())
        }
    }

    #[test]
    fn test_get_finish_task_from_mytask() {
        // Create [`channel`] to pass [`FinishedTask`].
        let (tx, rx) = channel::<FinishedTask>();
        let my_task = MyTask::new(0, "MyTask 0".to_string());
        my_task.run(tx);

        // [`FinishedTask`] created in [`Task`] will be got from channel.
        match rx.recv() {
            Ok(res) => {
                assert_eq!(res.tinfo().tid(), 0.into());
                assert_eq!(res.tinfo().tname(), "MyTask 0".to_string().into());
                assert_eq!(res.status, TaskStatus::Finished);
            }
            Err(_) => panic!("unreachable code"),
        }
    }

    #[test]
    fn test_running_task_join() {
        // Create [`channel`] to pass [`FinishedTask`].
        let (tx, rx) = channel::<FinishedTask>();
        let my_task = MyTask::new(0, "MyTask 0".to_string());
        let running_task = RunningTask {
            join_handle: Some(thread::spawn(move || {
                my_task.run(tx);
            })),
        };

        let mut res = rx.recv().unwrap();

        running_task.join(&mut res);

        assert_eq!(res.status(), TaskStatus::Finished);
    }
}
