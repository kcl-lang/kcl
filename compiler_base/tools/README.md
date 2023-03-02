# [WIP] compiler_base_test

## Summary

`compiler_base_test` is a part of [`compiler_base_tools`] crate that defines the core logic for unit tests execution engine. It aims to provide reusable components and abstract some functionalities for future developments.

The `compiler_base_test` crate consists of three main components: Task, Executor, and Reporter.

## Task
[`Task`](./src/test/task/mod.rs) is the smallest executable unit, any unit test case can be considered a [`Task`](./src/test/task/mod.rs) can be executed by [`Executor`](./src/test/executor/mod.rs). Therefore, we provide a trait to define a [`Task`](./src/test/task/mod.rs).

```rust
pub trait Task {
    /// [`run`] will be executed of the [[`Task`](./src/test/task/mod.rs)],
    /// and the result of the execution is communicated with other threads through the [`ch`] which is a [`Sender<FinishedTask>`],
    /// so [`run`] method does not need to return a value.
    fn run(&self, ch: Sender<FinishedTask>);

    /// Return the [`TaskInfo`]
    fn info(&self) -> TaskInfo;
}
```

To develop the unit test part for a compiler in a `compiler_base_test`-way, the first step is to convert the unit tests into [`Task`](./src/test/task/mod.rs).

For more information about [`Task`](./src/test/task/mod.rs), see the docs in source code in `./src/test/task/mod.rs`.

## Executor

[`Executor`](./src/test/executor/mod.rs) is responsible for executing the [`Task`](./src/test/task/mod.rs).

We provide a [`TimeoutExecutor`](./src/test/executor/timeout.rs) that references the implementation of the rustc unit testing and mainly contains the following features:

- Tasks are executed concurrently based on the number of threads.

- A timeout queue is used to monitor the execution of the [`Task`](./src/test/task/mod.rs) has timed out. If it does, a warning will be reported, but the [`Task`](./src/test/task/mod.rs) will not stop and will run until it is manually interrupted.

However, the rustc-way may not be sufficient for all unit testing executions pattern.

Therefore, we also provide a trait to define a [`Executor`](./src/test/executor/mod.rs).

```rust
pub trait Executor {
    /// [`run_all_tasks`] will execute all tasks concurrently.
    /// [`notify_what_happened`] is a notifier that receives [`TaskEvent`] to output the [[`Task`](./src/test/task/mod.rs)] execution status in to the log.
    fn run_all_tasks<T, F>(self, tasks: Vec<T>, notify_what_happened: F) -> io::Result<()>
    where
        T: Task + Sync + Send + 'static,
        F: Fn(TaskEvent) -> io::Result<()>;

    /// The count for threads.
    fn concurrency_capacity(&self) -> usize;
}
```

For more information about [`Executor`](./src/test/executor/mod.rs), see docs in source code in `./src/test/executor/mod.rs`.

## [WIP] Reportor


