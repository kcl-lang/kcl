//! This file provides [`TaskEvent`],
//! which tells the logging system to display information.
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::{FinishedTask, TaskInfo};

#[derive(Clone, Serialize, Deserialize)]
/// [`TaskEvent`] is an event that triggers the log displaying.
pub struct TaskEvent {
    tinfo: TaskInfo,
    ty: TaskEventType,
}

impl std::fmt::Display for TaskEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} event:{}", self.tinfo(), self.ty)
    }
}

impl PartialEq for TaskEvent {
    fn eq(&self, other: &Self) -> bool {
        self.tinfo == other.tinfo && self.ty == other.ty
    }
}

impl TaskEvent {
    /// Get [`super::TaskInfo`] about the [`super::Task`] that emitted this event.
    pub fn tinfo(&self) -> TaskInfo {
        self.tinfo.clone()
    }

    /// Get [`TaskEventType`]
    pub fn ty(&self) -> TaskEventType {
        self.ty.clone()
    }

    /// New a [`TaskEvent`] with [TaskEventType::Start].
    pub fn start(tinfo: TaskInfo) -> Self {
        Self {
            tinfo,
            ty: TaskEventType::Start,
        }
    }

    /// New a [`TaskEvent`] with [TaskEventType::Wait].
    pub fn wait(tinfo: TaskInfo) -> Self {
        Self {
            tinfo,
            ty: TaskEventType::Wait,
        }
    }

    /// New a [`TaskEvent`] with [TaskEventType::Timeout].
    pub fn time_out(tinfo: TaskInfo, deadline: Duration) -> Self {
        Self {
            tinfo,
            ty: TaskEventType::Timeout(deadline),
        }
    }

    /// New a [`TaskEvent`] with [TaskEventType::Finished].
    pub fn finished(tinfo: TaskInfo, finished_task: FinishedTask) -> Self {
        Self {
            tinfo,
            ty: TaskEventType::Finished(finished_task),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
/// [`TaskEventType`] is the event type of [`TaskEvent`].
pub enum TaskEventType {
    Start,
    Wait,
    Timeout(Duration),
    Finished(FinishedTask),
}

impl std::fmt::Display for TaskEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TaskEventType::Start => write!(f, "start"),
            TaskEventType::Wait => write!(f, "waiting"),
            TaskEventType::Timeout(t) => write!(f, "timeout {}s", t.as_secs()),
            TaskEventType::Finished(ft) => write!(f, "finished {}", ft),
        }
    }
}

impl PartialEq for TaskEventType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TaskEventType::Start, TaskEventType::Start)
            | (TaskEventType::Wait, TaskEventType::Wait) => true,
            (TaskEventType::Timeout(t1), TaskEventType::Timeout(t2)) => t1 == t2,
            (TaskEventType::Finished(f1), TaskEventType::Finished(f2)) => f1 == f2,
            _ => false,
        }
    }
}
