# Design Doc - KPM & LSP Integration

## Introduction

This document outlines the design to strengthen the integration between the KCL Language Protocol Server and KPM (Package Manager for KCL). We aim to automate the resolution of the dependencies and the recompilation of the workspace when users update the `kcl.mod` file or use KPM to update dependencies externally. If the workspace is correctly recompiled after running `kcl mod add <package>` command, the IDE errors because of the missing packages (which have been imported in some files) can be automatically resolved.

## Problem Statement

In the current implementation, consider the following two scenarios:

1. A user modifies the `kcl.mod` and adds some new dependencies there using the IDE: The newly added dependencies are not fetched automatically by the LSP.

2. A user has some `Module not found` errors in some of his files, so he runs the `kcl mod add ... ` kpm command: The IDE would not detect the added packages and the errors would continue.

In the current implementation, it is required to manually download the dependencies and then update the workspace to have it detect the added dependencies.

We propose modifications to the existing file-watching implementation to solve this problem.

## Current Implementation

The existing implementation in [`state.rs`](https://github.com/kcl-lang/kcl/blob/main/kclvm/tools/src/LSP/src/state.rs) file uses the `notify`crate (generally used for file-watching) to detect changes to the `kcl.mod` file. 

The file is watched for three types of changes: **Create**, **Modify** and **Delete**. When a modification of the `kcl.mod` file is detected, the workspace is recompiled asynchronously. But the current implementation does not support automatic resolution of dependencies.

## Proposed Implementation

The changes we propose to the existing system would have the following features:

1. **File Watching**: It would watch over the `kcl.mod` file and other configuration files.
2. **Debounce Mechanism**: It would introduce a debounce duration to delay the processing of successive rapid events.
3. **KPM Integration**: It would run KPM commands to download or update the dependencies when `kcl.mod` changes.
4. **LSP and Workspace Recompilation**: After dependency update/download due to external triggers or `kcl.mod` modification, the LSP would be recompiled to resolve any missing dependency errors (if present) and refresh its diagnostics. The IDE workspace would also be updated to these changes.
5. **Log Enhancement**: It would show log messages to the user in case of a failure.

## Technical Details

**1. Debouncing Events**

We will introduce a `debounce_duration` whose default value can be 1000 ms or 1 second. A timer logic is implemented and the timer resets every time an event occurs. The kpm command execution is done when the timer completes the debounce duration and expires. This way, if new events arrive during the timer, the older events are discarded.

This debouncing logic can be implemented in the `next_event` function of the [`state.rs`](https://github.com/kcl-lang/kcl/blob/main/kclvm/tools/src/LSP/src/state.rs) file. A `HashMap` can be used to store the timers for every file being watched.

**2. KPM Command Execution**

We can use `std::process::Command` to run the kpm commands in the case when the dependencies need to be updated (`kcl.mod` is modified). This will automatically download or update the dependecies. 

This command execution logic can be implemented in the `handle_changed_confg_file` function of the [`state.rs`](https://github.com/kcl-lang/kcl/blob/main/kclvm/tools/src/LSP/src/state.rs) file. 

A feature that can be added here is printing the logs for the user if the command execution results in an error or failure.

**3. Recompiling LSP and Workspace**

We use the `async_compile` function from [`state.rs`](https://github.com/kcl-lang/kcl/blob/main/kclvm/tools/src/LSP/src/state.rs) to reload the LSP and the workspace after dependency update/download for error resolution. 

**4. Better Logging and Debugging**

If any errors or failures are encountered at any step of the whole workflow of this integration, we should display the logs and diagnostics of that to the IDE for the user to see.

Some notifications should also be printed for the status of the process currently running in this implementation to make debugging easier. This would also prove better insights to the file-watcher and event-handler behaviour.

## Testing Implementation

We can add unit tests for the two cases described in the problem statement:

1. Simulate running the command `kcl mod add ...` using `std::process::Command` to test for the case of external dependency updates.

2. Use the `std::fs::write` to modify the `kcl.mod` file by adding/updating a dependency and test for this case.

## Future improvements

1. We can add a feature which would enalble users to choose to disable this functionality of automatic dependency download.

2. We can add a limit to the number of KPM processes executing simultaneously.

3. We can add a feature that would skip the command execution if the `kcl.mod` contents haven't changed.

## Conclusion

The implementation proposed in this document reduces manual intervention for dependency resolution and gives a better user experience by the integration of KPM and LSP.