//！This file provides a concrete implementation [`TimeoutExecutor`] of [`Executor`].
//! [`TimeoutExecutor`] is a [`Executor`] with a timeout queue that can monitor the timeout situation of [`Task`]s.
use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc::{channel, RecvTimeoutError},
    time::{Duration, Instant},
};

use crate::task::{
    event::TaskEvent, FinishedTask, RunningTask, Task, TaskId, TaskInfo, TaskStatus,
};

use super::{start_task, Executor};
use anyhow::{bail, Result};

/// [`TimeoutSituation`] is an internal structure for the timeout situation of a [`Task`].
pub(crate) struct TimeoutSituation {
    tinfo: TaskInfo,
    deadline: Instant,
}

/// [`TimeoutExecutor`] is a [`Executor`] with a timeout queue.
/// [`TimeoutExecutor`] is used in the same way as [`Executor`],
/// for more information, see doc [`Executor`].
pub struct TimeoutExecutor {
    timeout_queue: VecDeque<TimeoutSituation>,
    capacity: usize,
    timeout: Option<Instant>,
}

impl TimeoutExecutor {
    /// New a [`TimeoutExecutor`] with [`thread_count`].
    pub fn new_with_thread_count(thread_count: usize) -> Self {
        debug_assert!(
            thread_count > 0,
            "At least one thread is required to execute the task."
        );
        TimeoutExecutor {
            timeout_queue: VecDeque::default(),
            capacity: thread_count,
            timeout: Some(default_deadline_60_seconds()),
        }
    }

    /// New a [`TimeoutExecutor`] with [`thread_count`] and [`timeout`].
    pub fn new_with_thread_count_and_timeout(thread_count: usize, timeout: Instant) -> Self {
        debug_assert!(
            thread_count > 0,
            "At least one thread is required to execute the task."
        );
        TimeoutExecutor {
            timeout_queue: VecDeque::default(),
            capacity: thread_count,
            timeout: Some(timeout),
        }
    }

    /// Find all the timeout [`Task`] from the running tasks and return their [`TaskId`].
    fn all_timed_out_tasks_info(
        &mut self,
        running_tasks: &HashMap<TaskId, RunningTask>,
    ) -> Vec<TaskInfo> {
        let now = Instant::now();
        let mut timed_out = Vec::new();
        while let Some(timeout_entry) = self.timeout_queue.front() {
            if now < timeout_entry.deadline {
                break;
            }
            // Note: [`TimeoutSituation`]s of [`Task`]s that have timed out are removed from the queue.
            let timeout_entry = self.timeout_queue.pop_front().unwrap();
            if running_tasks.contains_key(&timeout_entry.tinfo.tid()) {
                timed_out.push(timeout_entry.tinfo);
            }
        }
        timed_out
    }

    /// [`deadline`] will return how long until the front of the queue has timed out,
    /// [`Duration::new(0, 0)`] if it has already timed out, and [`None`] if the queue is empty.
    fn deadline(&self) -> Option<Duration> {
        self.timeout_queue
            .front()
            .map(|&TimeoutSituation { deadline, .. }| {
                let now = Instant::now();
                if deadline >= now {
                    deadline - now
                } else {
                    Duration::new(0, 0)
                }
            })
    }
}

impl Executor for TimeoutExecutor {
    fn run_all_tasks<T, F>(mut self, tasks: &[T], notify_what_happened: F) -> Result<()>
    where
        T: Task + Clone + Sync + Send + 'static,
        F: Fn(TaskEvent) -> Result<()>,
    {
        // The channel for communication.
        let (tx, rx) = channel::<FinishedTask>();
        // All the [`Task`]s are waiting to be loaded into the thread.
        let mut waiting_tasks = VecDeque::from(tasks.to_vec());
        let mut running_tasks = HashMap::<TaskId, RunningTask>::default();
        let mut running_count = 0;

        // Load tasks into threads
        while running_count > 0 || !waiting_tasks.is_empty() {
            while running_count < self.concurrency_capacity() && !waiting_tasks.is_empty() {
                // Pop a [`Task`].
                let task = waiting_tasks.pop_front().unwrap();
                let tid = task.info().tid();
                let tinfo = task.info();

                // Calculate the deadline.
                let deadline = if let Some(timeout) = self.timeout {
                    timeout
                } else {
                    default_deadline_60_seconds()
                };

                // Notify the log that the [`Task`] is waiting to be executed.
                let event = TaskEvent::wait(task.info());
                notify_what_happened(event)?;

                // Load the [`Task`] into the thread for execution,
                // and return the [`JoinHandler`] corresponding to the thread.
                let join_handle = start_task(task, tx.clone());

                // Create [`RunningTask`] to manage thread after startup

                running_tasks.insert(tid, RunningTask { join_handle });

                // The [`TimeoutSituation`] of the current task is added to the queue.
                self.timeout_queue
                    .push_back(TimeoutSituation { tinfo, deadline });
                running_count += 1;
            }

            // Wait for the result of the [`Task`] execution
            let mut res;
            loop {
                if let Some(timeout) = self.deadline() {
                    // Waiting.
                    res = rx.recv_timeout(timeout);
                    // Notify the log that the [`Task`] is timeout.
                    for tid in self.all_timed_out_tasks_info(&running_tasks) {
                        notify_what_happened(TaskEvent::time_out(tid, timeout))?;
                    }
                    // Note: If the result of [`Task`] is not ready, it will wait for the result.
                    if res.is_ok() {
                        break;
                    };
                } else {
                    res = rx.recv().map_err(|_| RecvTimeoutError::Disconnected);
                    break;
                }
            }

            // Get the result of [`Task`] execution from channel.
            let mut finished_task = res.unwrap();

            // Get the thread [`JoinHandler<()>`] corresponding to the [`Task`].
            let running_task = match running_tasks.remove(&finished_task.tinfo().tid()) {
                Some(rs) => rs,
                None => {
                    panic!(
                        "size id {},  {}",
                        running_tasks.len(),
                        finished_task.tinfo()
                    )
                }
            };

            // And wait for the end of thread execution through [`join`].
            running_task.join(&mut finished_task);

            let fail = match finished_task.status() {
                // Only a bug will stop the testing process immediately,
                TaskStatus::Bug(_) => {
                    std::mem::forget(rx);
                    bail!(
                        "The task {} has completed, but the thread has failed",
                        finished_task.tinfo()
                    );
                }
                _ => false,
            };

            // Notify the log that the [`Task`] finished.
            let event = TaskEvent::finished(finished_task.tinfo(), finished_task);

            notify_what_happened(event)?;
            running_count -= 1;

            if fail {
                // Prevent remaining threads from panicking
                std::mem::forget(rx);
                return Ok(());
            }
        }
        Ok(())
    }

    fn concurrency_capacity(&self) -> usize {
        self.capacity
    }
}

/// Calculate the result of current time add the default timeout 60 seconds.
pub(crate) fn default_deadline_60_seconds() -> Instant {
    pub(crate) const _TIMEOUT_S: u64 = 60;
    Instant::now() + Duration::from_secs(_TIMEOUT_S)
}
