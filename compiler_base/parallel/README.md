# [WIP] compiler_base_parallel

## Summary

`compiler_base_parallel` defines the core logic for multitasking execution engine. It aims to provide reusable components and accumulate some general concurrency models for compiler developments.

The `compiler_base_parallel` crate consists of three main components: Task, Executor, and Reporter.

## Task
[`Task`](./src/task/mod.rs) is the smallest executable unit, anything can be considered as a [`Task`](./src/task/mod.rs) can be executed by [`Executor`](./src/executor/mod.rs). Therefore, we provide a trait to define a [`Task`](./src/task/mod.rs).

```rust
pub trait Task {
    /// [`run`] will be executed of the [[`Task`](./src/task/mod.rs)],
    /// and the result of the execution is communicated with other threads through the [`ch`] which is a [`Sender<FinishedTask>`],
    /// so [`run`] method does not need to return a value.
    fn run(&self, ch: Sender<FinishedTask>);

    /// Return the [`TaskInfo`]
    fn info(&self) -> TaskInfo;
}
```

To develop a concurrency mechanism for a compiler in a `compiler_base_parallel`-way, the first step is to create a [`Task`](./src/task/mod.rs).

For more information about [`Task`](./src/task/mod.rs), see the docs in source code in `./src/task/mod.rs`.

## Executor

[`Executor`](./src/executor/mod.rs) is responsible for executing the [`Task`](./src/task/mod.rs).

We also provide a trait to define a [`Executor`](./src/executor/mod.rs).

```rust
pub trait Executor {
    /// [`run_all_tasks`] will execute all tasks concurrently.
    /// [`notify_what_happened`] is a notifier that receives [`TaskEvent`] to output the [[`Task`](./src/task/mod.rs)] execution status in to the log.
    fn run_all_tasks<T, F>(self, tasks: Vec<T>, notify_what_happened: F) -> Result<()>
    where
        T: Task + Sync + Send + 'static,
        F: Fn(TaskEvent) -> Result<()>;

    /// The count for threads.
    fn concurrency_capacity(&self) -> usize;
}
```

For more information about [`Executor`](./src/executor/mod.rs), see docs in source code in `./src/executor/mod.rs`.

### TimeoutExecutor

[`TimeoutExecutor`](./src/executor/timeout.rs) refers to the concurrency mechanism adopted by rustc in the rust unit testing and mainly contains the following features:

- Tasks are executed concurrently based on the number of threads.

- A timeout queue is used to monitor the execution of the [`Task`](./src/task/mod.rs) has timed out. If it does, a warning will be reported, but the [`Task`](./src/task/mod.rs) will not stop and will run until it is manually interrupted.

If you want to implement unit testing, fuzz, bench or other you want to do in parallel in your compiler using the same workflow as rustc testing, you can use the [`TimeoutExecutor`](./src/executor/timeout.rs). If this workflow is not suitable for your compiler, you can choose to implement your own [`Executor`](./src/executor/mod.rs).

## [WIP] Reporter
