# Cargo Watch - Implementation and Workflow

## Introduction

Cargo-watch is a very helpful tool for development work as it can watch over over Rust projects and run cargo commands when a change is detected automatically. It creates a feedback loop which detects the event when the files to be watched over are modified and helps us to trigger commands when changes are detected.

## Core Architecture

Cargo-watch tool has been built on the `watchexec` tool which is a file system event notification system used to run commands in case of modifications on the paths being watched. `Watchexec` can be used to automatically run unit tests, run linters/syntax checkers and rebuild artifacts. 
Cargo-watch uses the `notify` crate which is generally used to recieve file events occuring. The cargo-watch tool has the following components:

### 1. Entry Point and Setup

The `main` function of the tool is the entry point to its implementation when the `cargo-watch` command is run. The file-watcher is initialized here and it also helps in setting up the environment for file-watching.

It first parses the agruments passed with the command and sets up the log configuration. Then, it uses its own root module to find the root directory of the preoject to be watched and changes its location to it (The root dir is found using the `cargo locate-project` command). The environment variables are set according to the relevant agruments passed with the command.


### 2. Command Line Interface

This tool uses the `clap` pacakage to provide a command line interface which has support for a lot of options for the tool. The arguments passed with the command are parsed in a separate module called args. The tool provides various options, some of which are:
- -w: watching only some provided paths (The local deps are also not watched)
- -x: cargo commands to be executed on file changes
- -s: shell commands to be executed on file changes
- -d: setting a debounce delay in secs
- -i: ignoring a glob/gitignore-style patterns

### 3. Watch Implementation

The file-watching implementation in `cargo-watch` is implemented in the watch module. It imports the `watchexec` crate for the watch functionality.

A struct `CwHandler` is created to handle the command execution in the tool. When initialized, it formats the commands to be executed in case of an event. When a file modification is detected, it uses the `watchexec` crate  to execute the commmand in case of a manual trigger or a detected change. the status of command execution is returned. In case the desktop notifications are on when non-manual change occurs, it creates one with a summary of the changes. 

```rust
pub struct CwHandler {
    cmd: String,        // Stores the command  to execute
    once: bool,         // Flag for single execution
    quiet: bool,        // Controls output verbosity
    notify: bool,       // Controls desktop notifications
    inner: ExecHandler, // Handles actual command execution
}

impl Handler for CwHandler {

    //Handle command execution in case of a manual trigger
    fn on_manual(&self) -> Result<bool> {
        if self.once {
            Ok(true)
        } else {
            self.start();
            self.inner.on_manual()
        }
    }

    //Handle command execution in case of a detected change
    fn on_update(&self, ops: &[PathOp]) -> Result<bool> {
        self.start();
        self.inner.on_update(ops).map(|o| {
            if self.notify {
                notify_rust::Notification::new()
                    .summary("Cargo Watch observed a change")
                    .body("Cargo Watch has seen a change, the command may have restarted.")
                    .show()
                    .map(drop)
                    .unwrap_or_else(|err| {
                        log::warn!("Failed to send desktop notification: {}", err);
                    });
            }

            o
        })
    }
}
```

### 4. Debouncing Events

Cargo-watch has implemented a debounce mechanism to stop many consecutive executions occuring one after another in very quick sucession. We can set the the debounce to a desired duration through the `-d` CLI flag. 

The `ConfigBuilder` from `watchexec` crate is used to set the given (or default - 0.5 sec) duration as the `poll interval` which is the duration after which the watcher looks for changes in the paths being watched (again and again) and the `debounce` which is the time to be waited before command execution when a change is detected.

```rust
//Set the poll duration and debounce duration to d
pub fn set_debounce(builder: &mut ConfigBuilder, matches: &ArgMatches) {
    if matches.is_present("delay") {
        let debounce = value_t!(matches, "delay", f32).unwrap_or_else(|e| e.exit());
        debug!("File updates debounce: {} seconds", debounce);

        let d = Duration::from_millis((debounce * 1000.0) as u64);
        builder.poll_interval(d).debounce(d);
    }
}
```

This debouncing implementation ensures that rapid changes do not trigger multiple executions.

### 5. Local Dependency Collection

The `find_local_deps` function in cargo-watch is used to find the all local dependencies in a project. It executes `cargo metadata` to collect the metadata of the project. Two maps are created, one of which maps package IDs to their package info and the other maps the package IDs to their dependency nodes. It performs a BFS with a visited map (for packages) and a queue on the dependency graph made earlier. The remote packages are skipped. For every unvisited package, it extracts its directory path and adds it to a set (`HashSet`). The set of dir paths is converted to a list and returned. 

These are the directories which are watched over by cargo-watch in a Rust project.

## Workflow Analysis

The typical workflow of cargo-watch follows these steps:

1. **Initialization Phase**
   - Parse CLI arguments and tool configuration
   - Set up the file watcher
   - Initialize the event channel
   - Determine watch paths

2. **Monitoring Phase**
   - Continuously watch specified directories (look after every poll duration)
   - Filter out ignored paths
   - Collect and debounce file system events

3. **Execution Phase**
   - Process filtered events
   - Execute specified cargo/shell commands
   - Handle command output and errors
   - Reset for next event detection

## Conclusion

We can take inspiration from this detailed implementation of cargo-watch tool to implement a similar file-watching and command execution mechanism for `kcl.mod` file updates. The key features of cargo-watch like command handler, file-watcher and debouncer provide a good example for a similar implementation in KCL.
