use crate::analysis::Analysis;
use crate::config::Config;
use crate::db::{AnalysisDatabase, DocumentVersion};
use crate::from_lsp::file_path_from_url;
use crate::to_lsp::{kcl_diag_to_lsp_diags, url};
use crate::util::{compile_with_params, get_file_name, to_json, Params};
use crate::word_index::build_word_index;
use anyhow::Result;
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use kclvm_parser::{KCLModuleCache, LoadProgramOptions};
use kclvm_sema::resolver::scope::KCLScopeCache;
use lsp_server::{ReqQueue, Request, Response};
use lsp_types::Url;
use lsp_types::{
    notification::{Notification, PublishDiagnostics},
    Diagnostic, InitializeParams, Location, PublishDiagnosticsParams,
};
use parking_lot::RwLock;
use ra_ap_vfs::{ChangeKind, ChangedFile, FileId, Vfs};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::{sync::Arc, time::Instant};

pub(crate) type RequestHandler = fn(&mut LanguageServerState, lsp_server::Response);

/// A `Task` is something that is send from async tasks to the entry point for processing. This
/// enables synchronizing resources like the connection with the client.
#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) enum Task {
    Response(Response),
    Notify(lsp_server::Notification),
    Retry(Request),
}

#[derive(Debug, Clone)]
pub(crate) enum Event {
    Task(Task),
    Lsp(lsp_server::Message),
}

pub(crate) struct Handle<H, C> {
    pub(crate) handle: H,
    pub(crate) _receiver: C,
}

pub(crate) type KCLVfs = Arc<RwLock<Vfs>>;
pub(crate) type KCLWordIndexMap = Arc<RwLock<HashMap<Url, HashMap<String, Vec<Location>>>>>;
pub(crate) type KCLCompileUnitCache =
    Arc<RwLock<HashMap<String, (Vec<String>, Option<LoadProgramOptions>)>>>;

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
    pub vfs: KCLVfs,

    /// True if the client requested that we shut down
    pub shutdown_requested: bool,

    /// Holds the state of the analysis process
    pub analysis: Analysis,

    /// Documents that are currently kept in memory from the client
    pub opened_files: Arc<RwLock<HashMap<FileId, DocumentVersion>>>,

    /// The VFS loader
    pub loader: Handle<Box<dyn ra_ap_vfs::loader::Handle>, Receiver<ra_ap_vfs::loader::Message>>,

    /// The word index map
    pub word_index_map: KCLWordIndexMap,

    /// KCL parse cache
    pub module_cache: KCLModuleCache,

    /// KCL resolver cache
    pub scope_cache: KCLScopeCache,

    /// KCL compile unit cache cache
    pub compile_unit_cache: KCLCompileUnitCache,
}

/// A snapshot of the state of the language server
#[allow(unused)]
pub(crate) struct LanguageServerSnapshot {
    /// The virtual filesystem that holds all the file contents
    pub vfs: Arc<RwLock<Vfs>>,
    /// Holds the state of the analysis process
    pub db: Arc<RwLock<HashMap<FileId, Arc<AnalysisDatabase>>>>,
    /// Documents that are currently kept in memory from the client
    pub opened_files: Arc<RwLock<HashMap<FileId, DocumentVersion>>>,
    /// The word index map
    pub word_index_map: KCLWordIndexMap,
    /// KCL parse cache
    pub module_cache: KCLModuleCache,
    /// KCL resolver cache
    pub scope_cache: KCLScopeCache,
    /// KCL compile unit cache cache
    pub compile_unit_cache: KCLCompileUnitCache,
}

#[allow(unused)]
impl LanguageServerState {
    pub fn new(
        sender: Sender<lsp_server::Message>,
        config: Config,
        initialize_params: InitializeParams,
    ) -> Self {
        let (task_sender, task_receiver) = unbounded::<Task>();

        let loader = {
            let (sender, _receiver) = unbounded::<ra_ap_vfs::loader::Message>();
            let handle: ra_ap_vfs_notify::NotifyHandle =
                ra_ap_vfs::loader::Handle::spawn(Box::new(move |msg| sender.send(msg).unwrap()));
            let handle = Box::new(handle) as Box<dyn ra_ap_vfs::loader::Handle>;
            Handle { handle, _receiver }
        };

        let state = LanguageServerState {
            sender,
            request_queue: ReqQueue::default(),
            _config: config,
            vfs: Arc::new(RwLock::new(Default::default())),
            thread_pool: threadpool::ThreadPool::default(),
            task_sender: task_sender.clone(),
            task_receiver,
            shutdown_requested: false,
            analysis: Analysis::default(),
            opened_files: Arc::new(RwLock::new(HashMap::new())),
            word_index_map: Arc::new(RwLock::new(HashMap::new())),
            loader,
            module_cache: KCLModuleCache::default(),
            scope_cache: KCLScopeCache::default(),
            compile_unit_cache: KCLCompileUnitCache::default(),
        };

        let word_index_map = state.word_index_map.clone();
        state.thread_pool.execute(move || {
            if let Err(err) = update_word_index_state(word_index_map, initialize_params, true) {
                log_message(err.to_string(), &task_sender);
            }
        });

        state
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
            Event::Task(task) => self.handle_task(task, start_time)?,
            Event::Lsp(msg) => {
                match msg {
                    lsp_server::Message::Request(req) => self.on_request(req, start_time)?,
                    lsp_server::Message::Notification(not) => self.on_notification(not)?,
                    // lsp_server::Message::Response(resp) => self.complete_request(resp),
                    _ => {}
                }
            }
        };

        // 2. Process changes
        self.process_vfs_changes();
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

        // Construct an AnalysisChange to apply to the analysis
        for file in changed_files {
            self.process_changed_file(file);
        }
        true
    }

    /// Process vfs changed file. Update db cache when creat(did_open_file), modity(did_change) or delete(did_close_file) vfs files.
    fn process_changed_file(&mut self, file: ChangedFile) {
        match file.change_kind {
            ChangeKind::Create | ChangeKind::Modify => {
                let filename = get_file_name(self.vfs.read(), file.file_id);
                match filename {
                    Ok(filename) => {
                        self.thread_pool.execute({
                            let mut snapshot = self.snapshot();
                            let sender = self.task_sender.clone();
                            let module_cache = Arc::clone(&self.module_cache);
                            let scope_cache = Arc::clone(&self.scope_cache);
                            let compile_unit_cache = Arc::clone(&self.compile_unit_cache);
                            move || match url(&snapshot, file.file_id) {
                                Ok(uri) => {
                                    let version =
                                        snapshot.opened_files.read().get(&file.file_id).cloned();
                                    let mut db = snapshot.db.write();
                                    match compile_with_params(Params {
                                        file: filename.clone(),
                                        module_cache: Some(module_cache),
                                        scope_cache: Some(scope_cache),
                                        vfs: Some(snapshot.vfs),
                                        compile_unit_cache: Some(compile_unit_cache),
                                    }) {
                                        Ok((prog, diags, gs)) => {
                                            let current_version = snapshot
                                                .opened_files
                                                .read()
                                                .get(&file.file_id)
                                                .cloned();
                                            match (version, current_version) {
                                                (Some(version), Some(current_version)) => {
                                                    // If the text is updated during compilation(current_version > version), the current compilation result will not be output.
                                                    if current_version == version {
                                                        db.insert(
                                                            file.file_id,
                                                            Arc::new(AnalysisDatabase {
                                                                prog,
                                                                diags: diags.clone(),
                                                                gs,
                                                                version,
                                                            }),
                                                        );

                                                        let diagnostics = diags
                                                            .iter()
                                                            .flat_map(|diag| {
                                                                kcl_diag_to_lsp_diags(
                                                                    diag,
                                                                    filename.as_str(),
                                                                )
                                                            })
                                                            .collect::<Vec<Diagnostic>>();
                                                        sender.send(Task::Notify(
                                                            lsp_server::Notification {
                                                                method: PublishDiagnostics::METHOD
                                                                    .to_owned(),
                                                                params: to_json(
                                                                    PublishDiagnosticsParams {
                                                                        uri,
                                                                        diagnostics,
                                                                        version: None,
                                                                    },
                                                                )
                                                                .unwrap(),
                                                            },
                                                        ));
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        Err(err) => {
                                            db.remove(&file.file_id);
                                            log_message(
                                                format!("Compile failed: {:?}", err.to_string()),
                                                &sender,
                                            );
                                        }
                                    }
                                }
                                Err(_) => {
                                    log_message(
                                        format!("Internal bug: not a valid file: {:?}", filename),
                                        &sender,
                                    );
                                }
                            }
                        });
                    }
                    Err(_) => {
                        self.log_message(format!("{:?} not found", file.file_id));
                    }
                }
            }
            ChangeKind::Delete => {
                let mut db = self.analysis.db.write();
                db.remove(&file.file_id);
            }
        }
    }

    /// Handles a task sent by another async task
    #[allow(clippy::unnecessary_wraps)]
    fn handle_task(&mut self, task: Task, request_received: Instant) -> anyhow::Result<()> {
        match task {
            Task::Notify(notification) => {
                self.send(notification.into());
            }
            Task::Response(response) => self.respond(response)?,
            Task::Retry(req) if !self.is_completed(&req) => {
                thread::sleep(Duration::from_millis(20));
                self.on_request(req, request_received)?
            }
            Task::Retry(_) => (),
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
            db: self.analysis.db.clone(),
            opened_files: self.opened_files.clone(),
            word_index_map: self.word_index_map.clone(),
            module_cache: self.module_cache.clone(),
            scope_cache: self.scope_cache.clone(),
            compile_unit_cache: self.compile_unit_cache.clone(),
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

    pub(crate) fn is_completed(&self, request: &lsp_server::Request) -> bool {
        self.request_queue.incoming.is_completed(&request.id)
    }
}

pub(crate) fn log_message(message: String, sender: &Sender<Task>) -> anyhow::Result<()> {
    let typ = lsp_types::MessageType::INFO;
    sender.send(Task::Notify(lsp_server::Notification::new(
        lsp_types::notification::LogMessage::METHOD.to_string(),
        lsp_types::LogMessageParams { typ, message },
    )))?;
    Ok(())
}

fn update_word_index_state(
    word_index_map: KCLWordIndexMap,
    initialize_params: InitializeParams,
    prune: bool,
) -> Result<()> {
    if let Some(workspace_folders) = initialize_params.workspace_folders {
        for folder in workspace_folders {
            let path = file_path_from_url(&folder.uri)?;
            if let Ok(word_index) = build_word_index(&path, prune) {
                word_index_map.write().insert(folder.uri, word_index);
            }
        }
    } else if let Some(root_uri) = initialize_params.root_uri {
        let path = file_path_from_url(&root_uri)?;
        if let Ok(word_index) = build_word_index(path, prune) {
            word_index_map.write().insert(root_uri, word_index);
        }
    }
    Ok(())
}
