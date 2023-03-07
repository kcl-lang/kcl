//! This file provides everything to define a [`Executor`] that can execute [`crate::task::Task`].
use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    thread::{self, JoinHandle},
};

use super::task::{event::TaskEvent, FinishedTask, Task};
use anyhow::Result;

pub mod timeout;

#[cfg(test)]
mod tests;

/// [`Executor`] can execute [`Task`] concurrently.
///
/// The following example is just to show how [`Task`] and [`Executor`] work.
/// It's not a guarantee that the following code is bug-free.
///
/// # Examples
///
/// ```rust
/// use compiler_base_parallel::task::TaskInfo;
/// use compiler_base_parallel::task::event::TaskEvent;
/// use compiler_base_parallel::task::Task;
/// use compiler_base_parallel::task::TaskStatus;
/// use std::sync::mpsc::Sender;
/// use std::sync::mpsc::channel;
/// use compiler_base_parallel::task::FinishedTask;
/// use compiler_base_parallel::executor::Executor;
/// use compiler_base_parallel::task::event::TaskEventType;
/// use std::thread;
/// use std::io;
/// use anyhow::Result;
///
/// // 1. First, we need to prepare a method to display to the log.
/// // Print the information.
/// fn print_log(event: TaskEvent) -> Result<()> {
///     match event.ty() {
///         TaskEventType::Start => {
///             println!("Task {} start.", event.tinfo())
///         }
///         TaskEventType::Wait => {
///             println!("Task {} waiting.", event.tinfo())
///         }
///         TaskEventType::Timeout(_) => {
///             println!("Task {} timeout.", event.tinfo())
///         }
///         TaskEventType::Finished(ft) => {
///             println!("Task {} finished {}", event.tinfo(), ft)
///         }
///     }
///     Ok(())
/// }
///
/// // 2. Define a custom executor [`MyExec`] for test.
/// pub(crate) struct MyExec {
///     pub(crate) num: usize,
/// }
///
/// // 3. Implement trait [`Executor`] for [`MyExec`].
/// impl Executor for MyExec {
///     fn run_all_tasks<T, F>(self, tasks: &[T], _notify_what_happened: F) -> Result<()>
///     where
///         T: Task + Clone + Sync + Send + 'static,
///         F: Fn(TaskEvent) -> Result<()>,
///    {
///         // The channel for communication.
///         let (tx, rx) = channel::<FinishedTask>();
///
///         // Load all tasks into the thread and execute.
///         let tasks = tasks.to_vec();
///         let mut threads = vec![];
///         let mut t_infos = vec![];
///         for t in tasks {
///             t_infos.push(t.info());
///             let ch = tx.clone();
///             threads.push(thread::spawn(move || t.run(ch)));
///         }
///
///         // Get all the task results and display to the log.
///         for ti in t_infos {
///             let _res = rx.recv().unwrap();
///             _notify_what_happened(TaskEvent::finished(ti, _res))?;
///         }
///         Ok(())
///     }
///
///     fn concurrency_capacity(&self) -> usize {
///         self.num
///     }
/// }
///
/// // 4. Define a custom task [`MyTask`] for test.
/// #[derive(Clone)]
/// struct MyTask {
///     id: usize,
///     name: String,
/// }
///
/// impl MyTask {
///     pub fn new(id: usize, name: String) -> Self {
///         Self { id, name }
///     }
/// }
/// // 5. Implement trait [`Task`] for [`MyTask`].
/// impl Task for MyTask {
///     fn run(&self, ch: Sender<FinishedTask>) {
///         // [`FinishedTask`] is constructed here passed to other threads via [`ch`].
///         ch.send(FinishedTask::new(
///             TaskInfo::new(self.id.into(), self.name.clone().into()),
///             vec![],
///             vec![],
///             TaskStatus::Finished,
///         ))
///         .unwrap();
///     }
///
///     fn info(&self) -> TaskInfo {
///         TaskInfo::new(self.id.into(), self.name.clone().into())
///     }
/// }
///
/// // Create an [`Executor`] with thread count 10.
/// let my_exec = MyExec { num: 10 };
///
/// // Create [`Task`]s
/// let my_tasks = vec![
///     MyTask { id: 0, name:"MyTask0".to_string() },
///     MyTask { id: 1, name:"MyTask1".to_string() },
///     MyTask { id: 2, name:"MyTask2".to_string() },
/// ];
/// my_exec.run_all_tasks(&my_tasks, |x| print_log(x)).unwrap();
/// ```
pub trait Executor {
    /// [`run_all_tasks`] will execute all tasks concurrently.
    /// [`notify_what_happened`] is a notifier that receives [`TaskEvent`] to output the [`Task`] execution status in to the log.
    fn run_all_tasks<T, F>(self, tasks: &[T], notify_what_happened: F) -> Result<()>
    where
        T: Task + Clone + Sync + Send + 'static,
        F: Fn(TaskEvent) -> Result<()>;

    /// The count for threads.
    fn concurrency_capacity(&self) -> usize;
}

/// [`start_task`] is mainly used to load the method of [`Task`] into the thread and start the execution,
/// and return the [`JoinHandle<()>`] of the corresponding thread.
/// If the current platform does not support multi-threading,
/// it directly executes the method and returns [`None`].
///
/// [`ch`] is used to communicate with the method loaded into the thread.
pub(crate) fn start_task<T>(task: T, ch: Sender<FinishedTask>) -> Option<JoinHandle<()>>
where
    T: Task + Sync + Send + 'static,
{
    let tname = task.info();
    let run_task = move || task.run(ch);
    let supports_threads = !cfg!(target_os = "emscripten") && !cfg!(target_family = "wasm");
    if supports_threads {
        let tb = thread::Builder::new().name(tname.into());
        let run_task = Arc::new(Mutex::new(Some(run_task)));
        match tb.spawn(move || run_task.lock().unwrap().take().unwrap()()) {
            Ok(handle) => Some(handle),
            Err(e) => panic!("failed to spawn thread to run test: {e}"),
        }
    } else {
        run_task();
        None
    }
}
