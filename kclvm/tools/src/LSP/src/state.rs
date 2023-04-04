use crate::config::Config;
use crate::to_lsp::{kcl_diag_to_lsp_diags, url};
use crate::util::{get_file_name, parse_param_and_compile, to_json, Param};
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use lsp_server::{ReqQueue, Response};
use lsp_types::{
    notification::{Notification, PublishDiagnostics},
    Diagnostic, PublishDiagnosticsParams,
};
use parking_lot::RwLock;
use ra_ap_vfs::Vfs;
use std::{sync::Arc, time::Instant};

pub(crate) type RequestHandler = fn(&mut LanguageServerState, lsp_server::Response);

/// A `Task` is something that is send from async tasks to the entry point for processing. This
/// enables synchronizing resources like the connection with the client.
#[allow(unused)]
#[derive(Debug)]
pub(crate) enum Task {
    Response(Response),
    Notify(lsp_server::Notification),
}

#[derive(Debug)]
pub(crate) enum Event {
    Task(Task),
    Lsp(lsp_server::Message),
}

/// State for the language server
pub(crate) struct LanguageServerState {
    /// Channel to send language server messages to the client
    pub(crate) sender: Sender<lsp_server::Message>,

    /// The request queue keeps track of all incoming and outgoing requests.
    pub(crate) request_queue: lsp_server::ReqQueue<(String, Instant), RequestHandler>,

    /// The configuration passed by the client
    pub _config: Config,

    /// Thread pool for async execution
    pub thread_pool: threadpool::ThreadPool,

    /// Channel to send tasks to from background operations
    pub task_sender: Sender<Task>,

    /// Channel to receive tasks on from background operations
    pub task_receiver: Receiver<Task>,

    /// The virtual filesystem that holds all the file contents
    pub vfs: Arc<RwLock<Vfs>>,

    /// True if the client requested that we shut down
    pub shutdown_requested: bool,
}

/// A snapshot of the state of the language server
#[allow(unused)]
pub(crate) struct LanguageServerSnapshot {
    /// The virtual filesystem that holds all the file contents
    pub vfs: Arc<RwLock<Vfs>>,
}

#[allow(unused)]
impl LanguageServerState {
    pub fn new(sender: Sender<lsp_server::Message>, config: Config) -> Self {
        let (task_sender, task_receiver) = unbounded::<Task>();
        LanguageServerState {
            sender,
            request_queue: ReqQueue::default(),
            _config: config,
            vfs: Arc::new(RwLock::new(Default::default())),
            thread_pool: threadpool::ThreadPool::default(),
            task_sender,
            task_receiver,
            shutdown_requested: false,
        }
    }

    /// Blocks until a new event is received from one of the many channels the language server
    /// listens to. Returns the first event that is received.
    fn next_event(&self, receiver: &Receiver<lsp_server::Message>) -> Option<Event> {
        select! {
            recv(receiver) -> msg => msg.ok().map(Event::Lsp),
            recv(self.task_receiver) -> task => Some(Event::Task(task.unwrap()))
        }
    }

    /// Runs the language server to completion
    pub fn run(mut self, receiver: Receiver<lsp_server::Message>) -> anyhow::Result<()> {
        while let Some(event) = self.next_event(&receiver) {
            if let Event::Lsp(lsp_server::Message::Notification(notification)) = &event {
                if notification.method == lsp_types::notification::Exit::METHOD {
                    return Ok(());
                }
            }
            self.handle_event(event)?;
        }
        Ok(())
    }

    /// Handles an event from one of the many sources that the language server subscribes to.
    fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        let start_time = Instant::now();
        // 1. Process the incoming event
        match event {
            Event::Task(task) => self.handle_task(task)?,
            Event::Lsp(msg) => match msg {
                lsp_server::Message::Request(req) => self.on_request(req, start_time)?,
                lsp_server::Message::Notification(not) => self.on_notification(not)?,
                // lsp_server::Message::Response(resp) => self.complete_request(resp),
                _ => {}
            },
        };

        // 2. Process changes
        // Todo: recompile and store result in db. Handle request and push diagnostis with db
        // let state_changed: bool = self.process_vfs_changes();

        // 3. Handle Diagnostics
        let mut snapshot = self.snapshot();
        let task_sender = self.task_sender.clone();
        // Spawn the diagnostics in the threadpool
        self.thread_pool.execute(move || {
            let _result = handle_diagnostics(snapshot, task_sender);
        });

        Ok(())
    }

    /// Processes any and all changes that have been applied to the virtual filesystem. Generates
    /// an `AnalysisChange` and applies it if there are changes. True is returned if things changed,
    /// otherwise false.
    pub fn process_vfs_changes(&mut self) -> bool {
        // Get all the changes since the last time we processed
        let changed_files = {
            let mut vfs = self.vfs.write();
            vfs.take_changes()
        };
        if changed_files.is_empty() {
            return false;
        }
        self.log_message("process_vfs_changes".to_string());

        // Construct an AnalysisChange to apply to the analysis
        let vfs = self.vfs.read();
        for file in changed_files {
            // todo: recompile and record context
        }
        true
    }

    /// Handles a task sent by another async task
    #[allow(clippy::unnecessary_wraps)]
    fn handle_task(&mut self, task: Task) -> anyhow::Result<()> {
        match task {
            Task::Notify(notification) => {
                self.send(notification.into());
            }
            Task::Response(response) => self.respond(response)?,
        }
        Ok(())
    }

    /// Sends a response to the client. This method logs the time it took us to reply
    /// to a request from the client.
    pub(super) fn respond(&mut self, response: lsp_server::Response) -> anyhow::Result<()> {
        if let Some((_method, start)) = self.request_queue.incoming.complete(response.id.clone()) {
            let duration = start.elapsed();
            self.send(response.into())?;
        }
        Ok(())
    }

    /// Sends a message to the client
    pub(crate) fn send(&mut self, message: lsp_server::Message) -> anyhow::Result<()> {
        self.sender.send(message)?;
        Ok(())
    }

    /// Registers a request with the server. We register all these request to make sure they all get
    /// handled and so we can measure the time it takes for them to complete from the point of view
    /// of the client.
    pub(crate) fn register_request(
        &mut self,
        request: &lsp_server::Request,
        request_received: Instant,
    ) {
        self.request_queue.incoming.register(
            request.id.clone(),
            (request.method.clone(), request_received),
        )
    }

    pub fn snapshot(&self) -> LanguageServerSnapshot {
        LanguageServerSnapshot {
            vfs: self.vfs.clone(),
        }
    }

    pub fn log_message(&mut self, message: String) {
        let typ = lsp_types::MessageType::INFO;
        let not = lsp_server::Notification::new(
            lsp_types::notification::LogMessage::METHOD.to_string(),
            lsp_types::LogMessageParams { typ, message },
        );
        self.send(not.into());
    }
}

// todo: `handle_diagnostics` only gets diag from db and converts them to lsp diagnostics.
fn handle_diagnostics(
    snapshot: LanguageServerSnapshot,
    sender: Sender<Task>,
) -> anyhow::Result<()> {
    let changed_files = {
        let mut vfs = snapshot.vfs.write();
        vfs.take_changes()
    };
    for file in changed_files {
        let (filename, uri) = {
            let vfs = snapshot.vfs.read();
            let filename = get_file_name(vfs, file.file_id)?;
            let uri = url(&snapshot, file.file_id)?;
            (filename, uri)
        };
        let (_, _, diags) = parse_param_and_compile(
            Param {
                file: filename.clone(),
            },
            Some(snapshot.vfs.clone()),
        )
        .unwrap();

        let diagnostics = diags
            .iter()
            .flat_map(|diag| kcl_diag_to_lsp_diags(diag, filename.as_str()))
            .collect::<Vec<Diagnostic>>();
        sender.send(Task::Notify(lsp_server::Notification {
            method: PublishDiagnostics::METHOD.to_owned(),
            params: to_json(PublishDiagnosticsParams {
                uri,
                diagnostics,
                version: None,
            })?,
        }))?;
    }
    Ok(())
}

pub(crate) fn log_message(message: String, sender: &Sender<Task>) -> anyhow::Result<()> {
    let typ = lsp_types::MessageType::INFO;
    sender.send(Task::Notify(lsp_server::Notification::new(
        lsp_types::notification::LogMessage::METHOD.to_string(),
        lsp_types::LogMessageParams { typ, message },
    )))?;
    Ok(())
}
