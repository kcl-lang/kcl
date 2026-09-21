use crate::analysis::{Analysis, AnalysisDatabase, DBState, OpenFileInfo};
use crate::compile::{Params, compile};
use crate::from_lsp::file_path_from_url;
use crate::mod_update::{self, DependencyUpdater, UpdateTrigger};
use crate::to_lsp::{kcl_diag_to_lsp_diags, url_from_path};
use crate::util::{filter_kcl_config_file, get_file_name, to_json};
use crossbeam_channel::{Receiver, Sender, select, unbounded};
use kcl_driver::toolchain::{self, Toolchain};
use kcl_driver::{
    CompileUnitOptions, WorkSpaceKind, lookup_compile_workspace, lookup_compile_workspace_bounded,
    lookup_compile_workspaces_bounded,
};
use kcl_error::{DiagnosticId, ErrorKind};
use kcl_parser::KCLModuleCache;
use kcl_sema::core::global_state::GlobalState;
use kcl_sema::resolver::scope::KCLScopeCache;
use lsp_server::RequestId;
use lsp_server::{ReqQueue, Request, Response};
use lsp_types::{
    InitializeParams, MessageType, PublishDiagnosticsParams, WorkspaceFolder,
    notification::{Notification, PublishDiagnostics, ShowMessage},
};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use ra_ap_vfs::{ChangeKind, ChangedFile, FileId, Vfs};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::{sync::Arc, sync::mpsc, time::Instant};

pub(crate) type RequestHandler = fn(&mut LanguageServerState, lsp_server::Response);

/// A `Task` is something that is send from async tasks to the entry point for processing. This
/// enables synchronizing resources like the connection with the client.
#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) enum Task {
    Response(Response),
    Notify(lsp_server::Notification),
    Retry(Request),
    ChangedFile(FileId, ChangeKind),
    ReOpenFile(FileId, ChangeKind),
    /// The compile of this workspace reported `CannotFindModule` — consider
    /// automatically updating its dependencies. See `mod_update`.
    MissingDependencies(WorkSpaceKind),
    /// Schedule a dependency update for every workspace whose `kcl.mod` lives
    /// in the given directory (all workspaces with a `kcl.mod` when `None`).
    RequestUpdateDependencies(Option<PathBuf>),
    /// Result of a `Toolchain::update_dependencies` run from the thread pool.
    DependenciesUpdated {
        workspace: WorkSpaceKind,
        result: Result<(), String>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum Event {
    Task(Task),
    Lsp(lsp_server::Message),
    FileWatcher(FileWatcherEvent),
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) enum FileWatcherEvent {
    Changed(Vec<PathBuf>),
    Removed(Vec<PathBuf>),
    Create(Vec<PathBuf>),
}

pub(crate) struct Handle<H, C> {
    pub(crate) handle: H,
    pub(crate) _receiver: C,
}

pub(crate) type KCLVfs = Arc<RwLock<Vfs>>;

pub(crate) type KCLWorkSpaceConfigCache = Arc<RwLock<HashMap<WorkSpaceKind, CompileUnitOptions>>>;

pub(crate) type KCLToolChain = Arc<RwLock<dyn Toolchain>>;
pub(crate) type KCLGlobalStateCache = Arc<Mutex<GlobalState>>;
pub(crate) type FSEventWatcher = Handle<
    Box<RecommendedWatcher>,
    mpsc::Receiver<std::result::Result<notify::Event, notify::Error>>,
>;

/// State for the language server
pub(crate) struct LanguageServerState {
    /// Channel to send language server messages to the client
    pub(crate) sender: Sender<lsp_server::Message>,
    /// The request queue keeps track of all incoming and outgoing requests.
    pub(crate) request_queue: lsp_server::ReqQueue<(String, Instant), RequestHandler>,
    /// Thread pool for async execution
    pub thread_pool: threadpool::ThreadPool,
    /// Channel to send tasks to from background operations
    pub task_sender: Sender<Task>,
    /// Channel to receive tasks on from background operations
    pub task_receiver: Receiver<Task>,
    /// True if the client requested that we shut down
    pub shutdown_requested: bool,
    /// The virtual filesystem that holds all the file contents
    pub vfs: KCLVfs,
    /// Holds the state of the analysis process
    pub analysis: Analysis,
    /// Documents that are currently kept in memory from the client
    pub opened_files: Arc<RwLock<HashMap<FileId, OpenFileInfo>>>,
    /// The VFS loader
    pub loader: Handle<Box<dyn ra_ap_vfs::loader::Handle>, Receiver<ra_ap_vfs::loader::Message>>,
    /// request retry time
    pub request_retry: Arc<RwLock<HashMap<RequestId, i32>>>,
    /// KCL parse cache
    pub module_cache: KCLModuleCache,
    /// KCL resolver cache
    pub scope_cache: KCLScopeCache,
    /// Toolchain is used to provider KCL tool features for the language server.
    pub tool: KCLToolChain,
    /// KCL globalstate cache
    pub gs_cache: KCLGlobalStateCache,
    /// Compile config cache
    pub workspace_config_cache: KCLWorkSpaceConfigCache,
    /// Process files that are not in any defined workspace and delete the workspace when closing the file
    pub temporary_workspace: Arc<RwLock<HashMap<FileId, Option<WorkSpaceKind>>>>,
    pub workspace_folders: Option<Vec<WorkspaceFolder>>,
    /// Actively monitor file system changes. These changes will not be notified through lsp,
    /// e.g., execute `kcl mod add xxx`, `kcl fmt xxx`
    pub fs_event_watcher: Option<FSEventWatcher>,
    /// Last time (per path) we forwarded a `kcl.mod` / `kcl.yaml` Modify
    /// event to the file-watcher pipeline. Used to debounce rapid bursts
    /// of Modify events (editors flush several writes per save, and a
    /// `kcl mod` invocation may modify kcl.mod multiple times before the
    /// vendor files land on disk). Without this guard, a future
    /// self-induced write to kcl.mod could trigger an infinite recompile
    /// loop; with it, only a single ingest pass runs per debounce
    /// window. See issue #1623.
    last_mod_change: Arc<RwLock<ModChangeDebouncer>>,
    /// Guards and records dependency-update attempts per workspace.
    pub(crate) dep_updater: DependencyUpdater,
    /// Whether a missing-module compile result automatically triggers
    /// `kcl mod update` for the workspace. Controlled by the client
    /// initialization option `kcl.mod.autoUpdate` (default: true).
    pub(crate) mod_auto_update: bool,
}

/// Debounce window for `kcl.mod` / `kcl.yaml` Modify events.
const MOD_CHANGE_DEBOUNCE: Duration = Duration::from_millis(500);

/// Per-path timestamp tracker used to coalesce `kcl.mod` / `kcl.yaml`
/// Modify events that arrive close together. Wrapped in its own tiny
/// type so it is trivially unit-testable.
#[derive(Default)]
pub(crate) struct ModChangeDebouncer {
    last: HashMap<PathBuf, Instant>,
}

impl ModChangeDebouncer {
    /// Returns `true` when at least one of `paths` was seen within the
    /// debounce window. The caller should drop the Modify event in that
    /// case.
    pub(crate) fn should_skip(&self, paths: &[PathBuf]) -> bool {
        let now = Instant::now();
        paths.iter().any(|p| {
            self.last
                .get(p)
                .map(|t| now.duration_since(*t) < MOD_CHANGE_DEBOUNCE)
                .unwrap_or(false)
        })
    }

    /// Record the current instant for every `path` so subsequent
    /// [`Self::should_skip`] calls report them as recently handled.
    pub(crate) fn mark(&mut self, paths: &[PathBuf]) {
        let now = Instant::now();
        for p in paths {
            self.last.insert(p.clone(), now);
        }
    }
}

/// A snapshot of the state of the language server
#[allow(unused)]
pub(crate) struct LanguageServerSnapshot {
    /// The virtual filesystem that holds all the file contents
    pub vfs: Arc<RwLock<Vfs>>,
    /// Holds the state of the analysis process
    pub workspaces: Arc<RwLock<HashMap<WorkSpaceKind, DBState>>>,
    /// Documents that are currently kept in memory from the client
    pub opened_files: Arc<RwLock<HashMap<FileId, OpenFileInfo>>>,
    /// request retry time
    pub request_retry: Arc<RwLock<HashMap<RequestId, i32>>>,
    /// KCL parse cache
    pub module_cache: KCLModuleCache,
    /// KCL resolver cache
    pub scope_cache: KCLScopeCache,
    /// Toolchain is used to provider KCL tool features for the language server.
    pub tool: KCLToolChain,
    /// Process files that are not in any defined workspace and delete the work
    pub temporary_workspace: Arc<RwLock<HashMap<FileId, Option<WorkSpaceKind>>>>,
    /// Compile config cache
    pub workspace_config_cache: KCLWorkSpaceConfigCache,
}

#[allow(unused)]
impl LanguageServerState {
    pub fn new(sender: Sender<lsp_server::Message>, initialize_params: InitializeParams) -> Self {
        let (task_sender, task_receiver) = unbounded::<Task>();

        let loader = {
            let (sender, _receiver) = unbounded::<ra_ap_vfs::loader::Message>();
            let handle: ra_ap_vfs_notify::NotifyHandle =
                ra_ap_vfs::loader::Handle::spawn(Box::new(move |msg| sender.send(msg).unwrap()));
            let handle = Box::new(handle) as Box<dyn ra_ap_vfs::loader::Handle>;
            Handle { handle, _receiver }
        };

        let fs_event_watcher = {
            let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
            match notify::recommended_watcher(tx) {
                Ok(watcher) => {
                    let handle = Box::new(watcher);
                    Some(Handle {
                        handle,
                        _receiver: rx,
                    })
                }
                Err(e) => {
                    log_message(
                        format!("Failed to init fs event watcher: {:?}", e),
                        &task_sender,
                    );
                    None
                }
            }
        };

        let mut state = LanguageServerState {
            sender,
            request_queue: ReqQueue::default(),
            vfs: Arc::new(RwLock::new(Default::default())),
            thread_pool: threadpool::ThreadPool::default(),
            task_sender: task_sender.clone(),
            task_receiver,
            shutdown_requested: false,
            analysis: Analysis::default(),
            opened_files: Arc::new(RwLock::new(HashMap::new())),
            loader,
            module_cache: KCLModuleCache::default(),
            scope_cache: KCLScopeCache::default(),
            tool: Arc::new(RwLock::new(toolchain::default())),
            gs_cache: KCLGlobalStateCache::default(),
            request_retry: Arc::new(RwLock::new(HashMap::new())),
            workspace_config_cache: KCLWorkSpaceConfigCache::default(),
            temporary_workspace: Arc::new(RwLock::new(HashMap::new())),
            workspace_folders: initialize_params
                .workspace_folders
                .clone()
                .filter(|folders| !folders.is_empty())
                .or_else(|| {
                    initialize_params.root_uri.as_ref().map(|uri| {
                        vec![WorkspaceFolder {
                            name: uri
                                .path()
                                .rsplit('/')
                                .next()
                                .unwrap_or_default()
                                .to_string(),
                            uri: uri.clone(),
                        }]
                    })
                }),
            fs_event_watcher,
            last_mod_change: Arc::new(RwLock::new(ModChangeDebouncer::default())),
            dep_updater: DependencyUpdater::default(),
            mod_auto_update: initialize_params
                .initialization_options
                .as_ref()
                .and_then(|options| options.pointer("/kcl/mod/autoUpdate"))
                .and_then(|value| value.as_bool())
                .unwrap_or(true),
        };

        state.init_workspaces();

        state
    }

    /// Blocks until a new event is received from one of the many channels the language server
    /// listens to. Returns the first event that is received.
    fn next_event(&self, receiver: &Receiver<lsp_server::Message>) -> Option<Event> {
        if let Some(fs_event_watcher) = &self.fs_event_watcher {
            for e in fs_event_watcher._receiver.try_iter().flatten() {
                match e.kind {
                    notify::EventKind::Modify(kind) => {
                        if let notify::event::ModifyKind::Data(data_change) = kind
                            && let notify::event::DataChange::Content = data_change
                        {
                            let paths = e.paths;
                            let kcl_config_file: Vec<PathBuf> = filter_kcl_config_file(&paths);
                            if !kcl_config_file.is_empty()
                                && !self.skip_recent_mod_change(&kcl_config_file)
                            {
                                self.mark_mod_change_handled(&kcl_config_file);
                                return Some(Event::FileWatcher(FileWatcherEvent::Changed(
                                    kcl_config_file,
                                )));
                            }
                        }
                    }
                    notify::EventKind::Remove(notify::event::RemoveKind::File) => {
                        let paths = e.paths;
                        let kcl_config_file: Vec<PathBuf> = filter_kcl_config_file(&paths);
                        if !kcl_config_file.is_empty() {
                            return Some(Event::FileWatcher(FileWatcherEvent::Removed(
                                kcl_config_file,
                            )));
                        }
                    }

                    notify::EventKind::Create(notify::event::CreateKind::File) => {
                        let paths = e.paths;
                        let kcl_config_file: Vec<PathBuf> = filter_kcl_config_file(&paths);
                        if !kcl_config_file.is_empty() {
                            return Some(Event::FileWatcher(FileWatcherEvent::Create(
                                kcl_config_file,
                            )));
                        }
                    }
                    _ => {}
                }
            }
        }

        select! {
            recv(receiver) -> msg => msg.ok().map(Event::Lsp),
            recv(self.task_receiver) -> task => Some(Event::Task(task.unwrap())),
        }
    }

    /// Returns `true` when at least one of `paths` is a `kcl.mod` /
    /// `kcl.yaml` Modify event that arrived inside the debounce window
    /// tracked by [`Self::mark_mod_change_handled`]. Used to dedupe
    /// editor flushes and avoid the self-recompile concern from #1623.
    fn skip_recent_mod_change(&self, paths: &[PathBuf]) -> bool {
        self.last_mod_change.read().should_skip(paths)
    }

    /// Record that we are about to handle a Modify event for the given
    /// config paths so that any subsequent Modify events within the
    /// debounce window are coalesced.
    fn mark_mod_change_handled(&self, paths: &[PathBuf]) {
        self.last_mod_change.write().mark(paths)
    }

    /// Runs the language server to completion
    pub fn run(mut self, receiver: Receiver<lsp_server::Message>) -> anyhow::Result<()> {
        while let Some(event) = self.next_event(&receiver) {
            if let Event::Lsp(lsp_server::Message::Notification(notification)) = &event
                && notification.method == lsp_types::notification::Exit::METHOD
            {
                return Ok(());
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
            Event::FileWatcher(file_watcher_event) => {
                self.handle_file_watcher_event(file_watcher_event)?
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

    /// Process vfs changed file. Update db cache when create(did_open_file), modify(did_change) or delete(did_close_file) vfs files.
    pub(crate) fn process_changed_file(&mut self, file: ChangedFile) {
        match file.change_kind {
            // open file
            ChangeKind::Create => {
                let filename = get_file_name(self.vfs.read(), file.file_id);
                self.log_message(format!("Process changed file, open {:?}", filename));
                match filename {
                    Ok(filename) => {
                        let uri = url_from_path(&filename).unwrap();
                        let mut state_workspaces = self.analysis.workspaces.read();
                        self.temporary_workspace.write().insert(file.file_id, None);

                        let mut may_contain = false;

                        // If some workspace has compiled this file, record open file's workspace
                        for (workspace, state) in state_workspaces.iter() {
                            match state {
                                DBState::Ready(db) => {
                                    if db.prog.modules.contains_key(&filename) {
                                        // The file may be present in a workspace's compiled
                                        // modules without being tracked in `opened_files` when
                                        // it was created externally (e.g. by `kcl import` or a
                                        // build step) and never received a `did_open`
                                        // notification from the editor. Skip the per-workspace
                                        // tracking in that case instead of panicking — the
                                        // outer `may_contain = true` still records that the
                                        // workspace already knows about the file.
                                        let mut openfiles = self.opened_files.write();
                                        match openfiles.get_mut(&file.file_id) {
                                            Some(file_info) => {
                                                file_info.workspaces.insert(workspace.clone());
                                            }
                                            None => {
                                                self.log_message(format!(
                                                    "File {:?} (file_id {:?}) is in workspace {:?} modules but not in opened_files; skipping per-workspace tracking",
                                                    filename, file.file_id, workspace
                                                ));
                                            }
                                        }
                                        drop(openfiles);
                                        may_contain = true;
                                    }
                                }
                                DBState::Compiling(_) | DBState::Init => {
                                    may_contain = true;
                                    self.task_sender
                                        .send(Task::ChangedFile(file.file_id, file.change_kind))
                                        .unwrap();
                                }
                                DBState::Failed(_) => continue,
                            }
                        }

                        if !may_contain {
                            self.log_message(format!(
                                "Not contains in any workspace, compile: {:?}",
                                filename
                            ));

                            let max_root = self.workspace_folders.as_ref().and_then(|folders| {
                                let file_path = Path::new(&filename);
                                folders
                                    .iter()
                                    .filter_map(|folder| file_path_from_url(&folder.uri).ok())
                                    .map(PathBuf::from)
                                    .filter(|folder_path| file_path.starts_with(folder_path))
                                    .max_by_key(|folder_path| folder_path.components().count())
                            });

                            let tool = Arc::clone(&self.tool);
                            let workspaces = lookup_compile_workspace_bounded(
                                &*tool.read(),
                                &filename,
                                true,
                                max_root.as_deref(),
                            )
                            .unwrap_or_default();

                            if workspaces.is_empty() {
                                self.temporary_workspace.write().remove(&file.file_id);
                                self.log_message(format!(
                                    "Not found any workspace for {:?}",
                                    filename
                                ));
                            } else {
                                for (mut workspace, opts) in workspaces {
                                    if matches!(workspace, WorkSpaceKind::NotFound)
                                        && let Some(parent) = Path::new(&filename).parent()
                                    {
                                        workspace = WorkSpaceKind::Folder(parent.to_path_buf());
                                    }
                                    match self.analysis.workspaces.read().get(&workspace).cloned() {
                                        Some(DBState::Ready(_)) => {
                                            let mut openfiles = self.opened_files.write();
                                            if let Some(file_info) =
                                                openfiles.get_mut(&file.file_id)
                                            {
                                                file_info.workspaces.insert(workspace.clone());
                                            }
                                            drop(openfiles);
                                            self.temporary_workspace.write().remove(&file.file_id);
                                        }
                                        Some(DBState::Compiling(_)) => {
                                            self.task_sender
                                                .send(Task::ChangedFile(
                                                    file.file_id,
                                                    file.change_kind,
                                                ))
                                                .unwrap();
                                        }
                                        Some(DBState::Init) | Some(DBState::Failed(_)) | None => {
                                            self.async_compile(
                                                workspace,
                                                opts,
                                                Some(file.file_id),
                                                true,
                                            );
                                        }
                                    }
                                }
                            }
                        } else {
                            self.temporary_workspace.write().remove(&file.file_id);
                        }
                    }
                    Err(err) => self.log_message(format!("{:?} not found: {}", file.file_id, err)),
                }
            }
            // edit file
            ChangeKind::Modify => {
                let filename = get_file_name(self.vfs.read(), file.file_id);
                self.log_message(format!("Process changed file, modify {:?}", filename));
                match filename {
                    Ok(filename) => {
                        let opened_files = self.opened_files.read();
                        let file_workspaces =
                            opened_files.get(&file.file_id).unwrap().workspaces.clone();

                        // In workspace
                        if !file_workspaces.is_empty() {
                            for workspace in file_workspaces {
                                let opts = self
                                    .workspace_config_cache
                                    .read()
                                    .get(&workspace)
                                    .unwrap()
                                    .clone();

                                self.async_compile(workspace, opts, Some(file.file_id), false);
                            }
                        } else {
                            // In temporary_workspace
                            let workspace = match self.temporary_workspace.read().get(&file.file_id)
                            {
                                Some(w) => match w {
                                    Some(w) => Some(w.clone()),
                                    None => {
                                        // In compiling, retry and wait for compile complete
                                        self.task_sender
                                            .send(Task::ChangedFile(file.file_id, file.change_kind))
                                            .unwrap();
                                        None
                                    }
                                },
                                None => None,
                            };
                            match workspace {
                                Some(workspace) => {
                                    let opts = self
                                        .workspace_config_cache
                                        .read()
                                        .get(&workspace)
                                        .unwrap()
                                        .clone();

                                    self.async_compile(workspace, opts, Some(file.file_id), true);
                                }
                                None => {
                                    self.log_message(format!(
                                        "Internal Bug: not found any workspace for file {:?}. Try to reload",
                                        filename
                                    ));

                                    self.task_sender
                                        .send(Task::ReOpenFile(file.file_id, ChangeKind::Create))
                                        .unwrap();
                                }
                            }
                        }
                    }
                    Err(err) => {
                        self.log_message(format!("{:?} not found: {}", file.file_id, err));
                    }
                }
            }
            // close file
            ChangeKind::Delete => {
                let filename = get_file_name(self.vfs.read(), file.file_id);
                self.log_message(format!("Process changed file, close {:?}", filename));

                let mut temporary_workspace = self.temporary_workspace.write();
                if let Some(workspace) = temporary_workspace.remove(&file.file_id) {
                    let mut workspaces = self.analysis.workspaces.write();
                    if let Some(w) = workspace {
                        let opened_file = self.opened_files.read();
                        let contain = opened_file.values().any(|f| f.workspaces.contains(&w));

                        if !contain {
                            self.log_message(format!("Remove workspace {:?}", w));
                            workspaces.remove(&w);
                        }
                    }
                }
            }
        }
    }

    /// Schedules a dependency update (`kcl mod update`) for the workspace
    /// through the configured toolchain, unless an attempt is already in
    /// flight or an automatic attempt was already made for the current
    /// `kcl.mod` generation. The update runs on the thread pool; its result
    /// comes back as `Task::DependenciesUpdated`.
    pub(crate) fn schedule_update_dependencies(
        &mut self,
        workspace: WorkSpaceKind,
        trigger: UpdateTrigger,
    ) {
        let Some(anchor) = mod_update::workspace_anchor(&workspace) else {
            return;
        };
        let Some((mod_dir, generation)) = mod_update::resolve_mod_dir(&anchor) else {
            self.log_message(format!(
                "Skip dependency update, no kcl.mod found for workspace {:?}",
                workspace
            ));
            return;
        };
        if !self.dep_updater.try_begin(&workspace, generation, trigger) {
            return;
        }
        self.log_message(format!(
            "Updating dependencies for workspace {:?} with `kcl mod update` in {:?}",
            workspace, mod_dir
        ));
        let tool = Arc::clone(&self.tool);
        let sender = self.task_sender.clone();
        self.thread_pool.execute(move || {
            // Catch panics (e.g. from the native toolchain) so the updater
            // guard never stays stuck in `InFlight` and the failure is still
            // surfaced to the user.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tool.read()
                    .update_dependencies(mod_dir.clone())
                    .map_err(|err| err.to_string())
            }))
            .unwrap_or_else(|_| Err("kcl mod update panicked".to_string()));
            let _ = sender.send(Task::DependenciesUpdated { workspace, result });
        });
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
            Task::ChangedFile(file_id, change_kind) => {
                thread::sleep(Duration::from_millis(20));
                self.process_changed_file(ChangedFile {
                    file_id,
                    change_kind,
                })
            }
            Task::ReOpenFile(file_id, change_kind) => self.process_changed_file(ChangedFile {
                file_id,
                change_kind,
            }),
            Task::MissingDependencies(workspace) => {
                if self.mod_auto_update {
                    self.schedule_update_dependencies(workspace, UpdateTrigger::Auto);
                }
            }
            Task::RequestUpdateDependencies(mod_dir) => {
                let mut targets: Vec<WorkSpaceKind> = Vec::new();
                let workspaces: Vec<WorkSpaceKind> =
                    self.analysis.workspaces.read().keys().cloned().collect();
                let temporary_workspaces: Vec<WorkSpaceKind> = self
                    .temporary_workspace
                    .read()
                    .values()
                    .flatten()
                    .cloned()
                    .collect();
                for workspace in workspaces.into_iter().chain(temporary_workspaces) {
                    if targets.contains(&workspace) {
                        continue;
                    }
                    let matches = match &mod_dir {
                        Some(dir) => {
                            mod_update::workspace_mod_dir(&workspace).as_deref()
                                == Some(dir.as_path())
                        }
                        None => mod_update::workspace_mod_dir(&workspace).is_some(),
                    };
                    if matches {
                        targets.push(workspace);
                    }
                }
                if targets.is_empty() {
                    self.log_message(format!(
                        "No workspace with a kcl.mod matched dependency update request {:?}",
                        mod_dir
                    ));
                }
                for workspace in targets {
                    self.schedule_update_dependencies(workspace, UpdateTrigger::Manual);
                }
            }
            Task::DependenciesUpdated { workspace, result } => match result {
                Ok(()) => {
                    self.dep_updater.mark_done(&workspace);
                    self.log_message(format!(
                        "Dependencies updated for workspace {:?}",
                        workspace
                    ));
                    // Refresh the workspace metadata and recompile so that the
                    // newly downloaded dependencies are picked up.
                    if let Some(anchor) = mod_update::workspace_anchor(&workspace) {
                        let opts = lookup_compile_workspace(
                            &*self.tool.read(),
                            anchor.to_string_lossy().as_ref(),
                            true,
                        );
                        self.async_compile(workspace, opts, None, false);
                    }
                }
                Err(err) => {
                    self.dep_updater.mark_failed(&workspace);
                    self.log_message(format!(
                        "Failed to update dependencies for workspace {:?}: {}",
                        workspace, err
                    ));
                    // Toolchain stderr can be long — keep the toast to the
                    // most recent part; the full output stays in the log.
                    // Prepend `...` only when truncation actually happened so
                    // short messages stay verbatim.
                    let tail: String = err
                        .chars()
                        .rev()
                        .take(300)
                        .collect::<Vec<char>>()
                        .into_iter()
                        .rev()
                        .collect();
                    let tail = if tail.chars().count() < err.chars().count() {
                        format!("...{tail}")
                    } else {
                        tail
                    };
                    self.show_message(
                        MessageType::WARNING,
                        format!("Failed to update KCL dependencies: {tail}"),
                    );
                }
            },
        }
        Ok(())
    }

    /// Handles a task sent by another async task
    #[allow(clippy::unnecessary_wraps)]
    fn handle_file_watcher_event(&mut self, event: FileWatcherEvent) -> anyhow::Result<()> {
        match event {
            FileWatcherEvent::Changed(paths) => self.handle_changed_confg_file(&paths),
            FileWatcherEvent::Create(paths) => self.handle_create_confg_file(&paths),
            FileWatcherEvent::Removed(paths) => self.handle_remove_confg_file(&paths),
        }
        Ok(())
    }

    /// Sends a response to the client. This method logs the time it took us to reply
    /// to a request from the client.
    pub(super) fn respond(&mut self, response: lsp_server::Response) -> anyhow::Result<()> {
        if let Some((method, start)) = self.request_queue.incoming.complete(&response.id) {
            let duration = start.elapsed();
            self.send(response.into())?;
            self.log_message(format!(
                "Finished request {:?} in {:?} micros",
                method,
                duration.as_micros()
            ));
        }
        Ok(())
    }

    /// Sends a message to the client
    pub(crate) fn send(&self, message: lsp_server::Message) -> anyhow::Result<()> {
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
            opened_files: self.opened_files.clone(),
            module_cache: self.module_cache.clone(),
            scope_cache: self.scope_cache.clone(),
            tool: self.tool.clone(),
            request_retry: self.request_retry.clone(),
            workspaces: self.analysis.workspaces.clone(),
            temporary_workspace: self.temporary_workspace.clone(),
            workspace_config_cache: self.workspace_config_cache.clone(),
        }
    }

    pub fn log_message(&self, message: String) {
        let typ = lsp_types::MessageType::INFO;
        let not = lsp_server::Notification::new(
            lsp_types::notification::LogMessage::METHOD.to_string(),
            lsp_types::LogMessageParams { typ, message },
        );
        self.send(not.into());
    }

    /// Send a `window/showMessage` notification to the client.
    ///
    /// Used for user-facing warnings/errors that the user should see in the
    /// editor UI (toast/notification area), as opposed to `log_message` which
    /// only emits INFO-level entries into the language client log.
    pub fn show_message(&self, typ: MessageType, message: String) {
        let not = lsp_server::Notification::new(
            ShowMessage::METHOD.to_string(),
            lsp_types::ShowMessageParams { typ, message },
        );
        self.send(not.into());
    }

    /// Handle errors from `notify::Watcher::watch`, surfacing a friendly
    /// `window/showMessage` warning when the OS file-watcher limit was hit.
    fn handle_watch_error(&self, path: &Path, err: &notify::Error) {
        self.log_message(format!("Failed watch {:?}: {:?}", path, err));
        if matches!(err.kind, notify::ErrorKind::MaxFilesWatch) {
            self.handle_watch_error_toast(path);
        }
    }

    /// Emit a `window/showMessage` WARNING telling the user how to recover
    /// from a `notify::ErrorKind::MaxFilesWatch` error. Split out from
    /// [`Self::handle_watch_error`] so the caller can free the mutable
    /// borrow of the watcher handle before we touch `self` again.
    fn handle_watch_error_toast(&self, path: &Path) {
        let msg = format!(
            "KCL language server failed to watch {:?}: the OS file watcher \
 limit was reached. On Linux you can increase it with:\n  \
             sudo sysctl fs.inotify.max_user_watches=<N>\n  \
             sudo sysctl -p\n\
             You can also add the offending directories (e.g. `.direnv/`, \
             `node_modules/`, `target/`) to `.gitignore` so the language \
             server can skip them.",
            path
        );
        self.show_message(MessageType::WARNING, msg);
    }

    pub(crate) fn is_completed(&self, request: &lsp_server::Request) -> bool {
        self.request_queue.incoming.is_completed(&request.id)
    }

    pub(crate) fn init_workspaces(&mut self) {
        self.log_message("Init workspaces".to_string());
        if let Some(workspace_folders) = &self.workspace_folders {
            for folder in workspace_folders {
                let path = file_path_from_url(&folder.uri).unwrap();
                let watch_targets = crate::util::collect_watch_paths(Path::new(&path));

                // Register watches, collecting any errors so we can report
                // them *after* the mutable borrow of the watcher handle is
                // released (so we can call `self.log_message` / `self.show_message`).
                let mut watched: usize = 0;
                let mut errors: Vec<(PathBuf, notify::Error)> = Vec::new();
                {
                    let Some(fs_event_watcher) = &mut self.fs_event_watcher else {
                        continue;
                    };
                    let watcher = &mut fs_event_watcher.handle;

                    if watch_targets.is_empty() {
                        // Nothing non-ignored to watch — fall back to the root
                        // so we still receive events if files appear later.
                        match watcher.watch(Path::new(&path), RecursiveMode::Recursive) {
                            Ok(_) => watched = 1,
                            Err(e) => errors.push((PathBuf::from(&path), e)),
                        }
                    } else {
                        for target in &watch_targets {
                            match watcher.watch(target, RecursiveMode::Recursive) {
                                Ok(_) => watched += 1,
                                Err(e) => errors.push((target.clone(), e)),
                            }
                        }
                    }
                }

                // Mutable borrow released; safe to call `self` methods.
                let mut max_files_watch: Option<PathBuf> = None;
                for (p, e) in &errors {
                    self.log_message(format!("Failed watch {:?}: {:?}", p, e));
                    if matches!(e.kind, notify::ErrorKind::MaxFilesWatch) {
                        max_files_watch.get_or_insert_with(|| p.clone());
                    }
                }
                if watch_targets.is_empty() {
                    if errors.is_empty() {
                        self.log_message(format!("Start watch {:?}", path));
                    }
                } else {
                    self.log_message(format!("Start watch {:?} ({} entries)", path, watched));
                }
                if let Some(p) = max_files_watch {
                    self.handle_watch_error_toast(&p);
                }

                let tool = Arc::clone(&self.tool);
                let (workspaces, failed) = lookup_compile_workspaces_bounded(
                    &*tool.read(),
                    &path,
                    true,
                    Some(Path::new(&path)),
                );

                if let Some(failed) = failed {
                    for (key, err) in failed {
                        self.log_message(format!("parse kcl.work failed: {}: {}", key, err));
                    }
                }

                for (workspace, opts) in workspaces {
                    self.async_compile(workspace, opts, None, false);
                }
            }
        }
    }

    pub(crate) fn async_compile(
        &self,
        workspace: WorkSpaceKind,
        opts: CompileUnitOptions,
        changed_file_id: Option<FileId>,
        temp: bool,
    ) {
        let filename = match changed_file_id {
            Some(id) => get_file_name(self.vfs.read(), id).ok(),
            None => None,
        };

        let mut workspace_config_cache = self.workspace_config_cache.write();
        workspace_config_cache.insert(workspace.clone(), opts.clone());
        drop(workspace_config_cache);

        self.thread_pool.execute({
            let mut snapshot = self.snapshot();
            let sender = self.task_sender.clone();
            let module_cache = Arc::clone(&self.module_cache);
            let scope_cache = Arc::clone(&self.scope_cache);
            let tool = Arc::clone(&self.tool);
            let gs_cache = Arc::clone(&self.gs_cache);

            let mut files = opts.0.clone();
            move || {
                let old_diags = {
                    match snapshot.workspaces.read().get(&workspace) {
                        Some(option_db) => match option_db {
                            DBState::Ready(db) => db.diags.clone(),
                            DBState::Compiling(db) => db.diags.clone(),
                            DBState::Init | DBState::Failed(_) => Default::default(),
                        },
                        None => Default::default(),
                    }
                };

                {
                    let mut workspaces = snapshot.workspaces.write();
                    let state = match workspaces.get_mut(&workspace) {
                        Some(state) => match state {
                            DBState::Ready(db) => DBState::Compiling(db.clone()),
                            DBState::Compiling(db) => DBState::Compiling(db.clone()),
                            DBState::Init | DBState::Failed(_) => DBState::Init,
                        },
                        None => DBState::Init,
                    };
                    workspaces.insert(workspace.clone(), state);
                }
                let start = Instant::now();

                let (diags, compile_res) = compile(
                    Params {
                        file: filename.clone(),
                        module_cache: Some(module_cache),
                        scope_cache: Some(scope_cache),
                        vfs: Some(snapshot.vfs),
                        gs_cache: Some(gs_cache),
                    },
                    &mut files,
                    opts.1.clone(),
                );

                log_message(
                    format!(
                        "Compile workspace: {:?}, main_pkg files: {:?}, changed file: {:?}, options: {:?}, metadate: {:?}, use {:?} micros",
                        workspace,
                        files,
                        filename,
                        opts.1,
                        opts.2,
                        start.elapsed().as_micros()
                    ),
                    &sender,
                );

                let mut old_diags_maps = HashMap::new();
                for diag in &old_diags {
                    let lsp_diag = kcl_diag_to_lsp_diags(diag);
                    for (key, value) in lsp_diag {
                        old_diags_maps.entry(key).or_insert(vec![]).extend(value);
                    }
                }

                // publish diags
                let mut new_diags_maps = HashMap::new();

                for diag in &diags {
                    let lsp_diag = kcl_diag_to_lsp_diags(diag);
                    for (key, value) in lsp_diag {
                        new_diags_maps.entry(key).or_insert(vec![]).extend(value);
                    }
                }

                for (file, diags) in old_diags_maps {
                    if !new_diags_maps.contains_key(&file)
                        && let Ok(uri) = url_from_path(file) {
                            sender.send(Task::Notify(lsp_server::Notification {
                                method: PublishDiagnostics::METHOD.to_owned(),
                                params: to_json(PublishDiagnosticsParams {
                                    uri: uri.clone(),
                                    diagnostics: vec![],
                                    version: None,
                                })
                                .unwrap(),
                            }));
                        }
                }

                for (filename, diagnostics) in new_diags_maps {
                    if let Ok(uri) = url_from_path(filename) {
                        sender.send(Task::Notify(lsp_server::Notification {
                            method: PublishDiagnostics::METHOD.to_owned(),
                            params: to_json(PublishDiagnosticsParams {
                                uri: uri.clone(),
                                diagnostics,
                                version: None,
                            })
                            .unwrap(),
                        }));
                    }
                }

                // If the workspace imports a module that cannot be found, its
                // dependencies may not be updated yet. Ask the main loop to
                // schedule a `kcl mod update` (guarded against repeated
                // attempts, see `mod_update`).
                if diags.iter().any(|diag| {
                    matches!(
                        &diag.code,
                        Some(DiagnosticId::Error(ErrorKind::CannotFindModule))
                    )
                }) {
                    let _ = sender.send(Task::MissingDependencies(workspace.clone()));
                }

                match compile_res {
                    Ok((prog, schema_map, gs)) => {
                        let mut workspaces = snapshot.workspaces.write();
                        log_message(
                            format!(
                                "Workspace {:?} compile success",workspace
                            ),
                            &sender,
                        );
                        workspaces.insert(
                            workspace.clone(),
                            DBState::Ready(Arc::new(AnalysisDatabase { prog, gs, diags,schema_map })),
                        );
                        drop(workspaces);
                        if temp && let Some(changed_file_id) = changed_file_id {
                            let mut temporary_workspace = snapshot.temporary_workspace.write();

                            log_message(
                                format!(
                                    "Insert file {:?} and workspace {:?} to temporary workspace", filename, workspace
                                ),
                                &sender,
                            );
                            temporary_workspace
                                .insert(changed_file_id, Some(workspace.clone()));
                            drop(temporary_workspace);
                        }
                    }
                    Err(e) => {
                        let mut workspaces = snapshot.workspaces.write();
                        log_message(
                            format!(
                                "Workspace {:?} compile failed: {:?}",workspace, e
                            ),
                            &sender,
                        );
                        workspaces.insert(workspace, DBState::Failed(e.to_string()));
                        if temp && let Some(changed_file_id) = changed_file_id {
                            let mut temporary_workspace = snapshot.temporary_workspace.write();
                            log_message(
                                format!(
                                    "Reomve temporary workspace file id: {:?}", changed_file_id
                                ),
                                &sender,
                            );
                            temporary_workspace.remove(&changed_file_id);
                            drop(temporary_workspace);
                        }
                    }
                }
            }
        })
    }

    // Configuration file modifications that do not occur on the IDE client side, e.g., `kcl mod add xxx``
    pub(crate) fn handle_changed_confg_file(&self, paths: &[PathBuf]) {
        for path in paths {
            self.log_message(format!("Changed config file {:?}", path));
            // In workspaces
            let mut workspaces = self.analysis.workspaces.write();
            for workspace in workspaces.keys() {
                if let Some(p) = match workspace {
                    WorkSpaceKind::ModFile(path_buf) => Some(path_buf.clone()),
                    WorkSpaceKind::SettingFile(path_buf) => Some(path_buf.clone()),
                    _ => None,
                } {
                    let opts =
                        lookup_compile_workspace(&*self.tool.read(), p.to_str().unwrap(), true);
                    self.async_compile(workspace.clone(), opts, None, false);
                }
            }
            drop(workspaces);

            // In temp workspaces
            let mut temp_workspace = self.temporary_workspace.write();

            for (file_id, workspace) in temp_workspace.iter_mut() {
                if let Some(p) = if let Some(w) = workspace {
                    match w {
                        WorkSpaceKind::ModFile(path_buf) => Some(path_buf.clone()),
                        WorkSpaceKind::SettingFile(path_buf) => Some(path_buf.clone()),
                        _ => None,
                    }
                } else {
                    None
                } {
                    let opts =
                        lookup_compile_workspace(&*self.tool.read(), p.to_str().unwrap(), true);
                    self.async_compile(workspace.clone().unwrap(), opts, Some(*file_id), false);
                }
            }
        }
    }

    fn handle_create_confg_file(&self, paths: &[PathBuf]) {
        for path in paths {
            // Just log, nothing to do
            self.log_message(format!("Create config file: {:?}", path));
        }
    }

    fn handle_remove_confg_file(&self, paths: &[PathBuf]) {
        for path in paths {
            self.log_message(format!("Remove config file: {:?}", path));
            // todo: clear cache
        }
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
