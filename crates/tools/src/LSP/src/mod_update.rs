//! Automatic dependency updates through the kcl mod toolchain.
//!
//! When a workspace is loaded whose third-party dependencies are missing or
//! outdated, the language server can update them through
//! [`kcl_driver::toolchain::Toolchain::update_dependencies`] (i.e. the
//! `kcl mod update` command for the default command toolchain) instead of
//! leaving the user with a bare `CannotFindModule` diagnostic.
//!
//! All entry points (a missing-module compile result, saving `kcl.mod`, and
//! the manual `kcl.updateDependencies` command / quick fix) funnel into
//! [`LanguageServerState::schedule_update_dependencies`], which runs the
//! update on the thread pool and recompiles the workspace on success.
//!
//! See https://github.com/kcl-lang/kcl/issues/1428.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use kcl_config::modfile::{KCL_MOD_FILE, get_pkg_root};
use kcl_driver::WorkSpaceKind;

/// LSP command (registered in `execute_command_provider`) that updates the
/// dependencies of the workspace whose `kcl.mod` lives in the given directory.
pub(crate) const UPDATE_DEPENDENCIES_COMMAND: &str = "kcl.updateDependencies";

/// What caused a dependency update to be scheduled. Manual triggers bypass
/// the per-generation guard so an explicit user action always runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum UpdateTrigger {
    /// Triggered automatically by a `CannotFindModule` compile result.
    Auto,
    /// Triggered by an explicit user action (quick fix, command, `kcl.mod` save).
    Manual,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UpdateState {
    InFlight,
    Done,
    Failed,
}

/// Records dependency-update attempts per workspace. Only accessed from the
/// main loop thread.
#[derive(Default)]
pub(crate) struct DependencyUpdater {
    /// The `SystemTime` is the mtime of the workspace's `kcl.mod` at the time
    /// of the attempt, used as a generation key: changing `kcl.mod` (e.g.
    /// through `kcl mod add`) re-arms the guard for automatic triggers.
    states: HashMap<WorkSpaceKind, (UpdateState, SystemTime)>,
}

impl DependencyUpdater {
    /// Begin an update attempt for `workspace` unless one is in flight, or an
    /// automatic attempt was already made for the same `kcl.mod` generation.
    pub(crate) fn try_begin(
        &mut self,
        workspace: &WorkSpaceKind,
        generation: SystemTime,
        trigger: UpdateTrigger,
    ) -> bool {
        let blocked = match self.states.get(workspace) {
            Some((UpdateState::InFlight, _)) => true,
            Some((_, recorded_gen))
                if trigger == UpdateTrigger::Auto && *recorded_gen == generation =>
            {
                true
            }
            _ => false,
        };
        if !blocked {
            self.states
                .insert(workspace.clone(), (UpdateState::InFlight, generation));
        }
        !blocked
    }

    fn mark(&mut self, workspace: &WorkSpaceKind, state: UpdateState) {
        if let Some(entry) = self.states.get_mut(workspace) {
            entry.0 = state;
        }
    }

    pub(crate) fn mark_done(&mut self, workspace: &WorkSpaceKind) {
        self.mark(workspace, UpdateState::Done);
    }

    pub(crate) fn mark_failed(&mut self, workspace: &WorkSpaceKind) {
        self.mark(workspace, UpdateState::Failed);
    }
}

/// The anchor path used to resolve a workspace's `kcl.mod`.
pub(crate) fn workspace_anchor(workspace: &WorkSpaceKind) -> Option<PathBuf> {
    match workspace {
        WorkSpaceKind::WorkFile(path)
        | WorkSpaceKind::ModFile(path)
        | WorkSpaceKind::SettingFile(path)
        | WorkSpaceKind::Folder(path)
        | WorkSpaceKind::File(path) => Some(path.clone()),
        WorkSpaceKind::NotFound => None,
    }
}

/// Resolve the directory containing the nearest `kcl.mod` for `anchor`,
/// together with the mtime of that `kcl.mod` as the generation key.
pub(crate) fn resolve_mod_dir(anchor: &Path) -> Option<(PathBuf, SystemTime)> {
    let dir = get_pkg_root(anchor.to_str()?)?;
    let modified = std::fs::metadata(Path::new(&dir).join(KCL_MOD_FILE))
        .ok()?
        .modified()
        .ok()?;
    Some((PathBuf::from(dir), modified))
}

/// The directory containing the nearest `kcl.mod` of the workspace, if any.
pub(crate) fn workspace_mod_dir(workspace: &WorkSpaceKind) -> Option<PathBuf> {
    workspace_anchor(workspace).and_then(|anchor| resolve_mod_dir(&anchor).map(|(dir, _)| dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::LanguageServerState;
    use crate::util::to_json;
    use crossbeam_channel::{Receiver, Sender, unbounded};
    use kcl_driver::toolchain::{Metadata, Toolchain};
    use lsp_types::notification::{
        DidOpenTextDocument, DidSaveTextDocument, LogMessage, Notification as _,
    };
    use lsp_types::{
        DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializeParams,
        TextDocumentIdentifier, TextDocumentItem, Url,
    };
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    /// A `Toolchain` that counts `update_dependencies` calls and never touches
    /// the network or a subprocess.
    struct UpdateCountingToolchain {
        updates: Arc<AtomicUsize>,
        fail: bool,
    }

    impl Toolchain for UpdateCountingToolchain {
        fn fetch_metadata(&self, _manifest_path: PathBuf) -> anyhow::Result<Metadata> {
            Ok(Metadata::default())
        }

        fn update_dependencies(&self, _manifest_path: PathBuf) -> anyhow::Result<()> {
            if self.fail {
                anyhow::bail!("fake update failure");
            }
            self.updates.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn temp_workspace() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "kcl-lsp-mod-update-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(KCL_MOD_FILE),
            "[package]\nname = \"mod_update_test\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("main.k"),
            "import nonexistent_pkg\n\na = nonexistent_pkg.value\n",
        )
        .unwrap();
        dir
    }

    /// Run a real `LanguageServerState` on a memory connection with the fake
    /// toolchain installed. Returns the client-side ends of the connection and
    /// the update counter.
    fn spawn_state(
        tool: UpdateCountingToolchain,
        initialize_params: InitializeParams,
    ) -> (
        Receiver<lsp_server::Message>,
        Sender<lsp_server::Message>,
        Arc<AtomicUsize>,
    ) {
        let (server_tx, server_rx) = unbounded::<lsp_server::Message>();
        let (client_tx, client_rx) = unbounded::<lsp_server::Message>();
        let updates = Arc::clone(&tool.updates);
        std::thread::spawn(move || {
            let mut state = LanguageServerState::new(server_tx, initialize_params);
            state.tool = Arc::new(RwLock::new(tool));
            let _ = state.run(client_rx);
        });
        (server_rx, client_tx, updates)
    }

    fn open_file(client_tx: &Sender<lsp_server::Message>, path: &Path) {
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path).unwrap(),
                language_id: "kcl".to_string(),
                version: 1,
                text: std::fs::read_to_string(path).unwrap(),
            },
        };
        client_tx
            .send(lsp_server::Message::Notification(
                lsp_server::Notification::new(
                    DidOpenTextDocument::METHOD.to_string(),
                    to_json(&params).unwrap(),
                ),
            ))
            .unwrap();
    }

    fn save_file(client_tx: &Sender<lsp_server::Message>, path: &Path) {
        let params = DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(path).unwrap(),
            },
            text: None,
        };
        client_tx
            .send(lsp_server::Message::Notification(
                lsp_server::Notification::new(
                    DidSaveTextDocument::METHOD.to_string(),
                    to_json(&params).unwrap(),
                ),
            ))
            .unwrap();
    }

    fn wait_for_notification(
        rx: &Receiver<lsp_server::Message>,
        pred: impl Fn(&lsp_server::Notification) -> bool,
    ) -> lsp_server::Notification {
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            match rx.recv_timeout(remaining) {
                Ok(lsp_server::Message::Notification(not)) if pred(&not) => return not,
                Ok(_) => {}
                Err(_) => panic!("timed out waiting for server notification"),
            }
        }
    }

    fn is_log_containing(needle: &'static str) -> impl Fn(&lsp_server::Notification) -> bool {
        move |not| {
            not.method == LogMessage::METHOD
                && serde_json::from_value::<lsp_types::LogMessageParams>(not.params.clone())
                    .map(|params| params.message.contains(needle))
                    .unwrap_or(false)
        }
    }

    #[test]
    fn dependency_updater_guard_test() {
        let mut updater = DependencyUpdater::default();
        let workspace = WorkSpaceKind::Folder(PathBuf::from("/tmp/dep-updater-test"));
        let generation = SystemTime::now();

        assert!(updater.try_begin(&workspace, generation, UpdateTrigger::Auto));
        // A concurrent attempt is skipped while one is in flight.
        assert!(!updater.try_begin(&workspace, generation, UpdateTrigger::Auto));
        updater.mark_done(&workspace);
        // The same kcl.mod generation is not auto-updated twice.
        assert!(!updater.try_begin(&workspace, generation, UpdateTrigger::Auto));
        // Manual triggers always bypass the guard.
        assert!(updater.try_begin(&workspace, generation, UpdateTrigger::Manual));
        updater.mark_failed(&workspace);
        assert!(!updater.try_begin(&workspace, generation, UpdateTrigger::Auto));
        // A new kcl.mod generation (e.g. after `kcl mod add`) re-arms the guard.
        let new_generation = generation + Duration::from_secs(1);
        assert!(updater.try_begin(&workspace, new_generation, UpdateTrigger::Auto));
    }

    #[test]
    fn resolve_mod_dir_finds_nearest_kcl_mod() {
        let base = std::env::temp_dir().join(format!(
            "kcl-lsp-resolve-mod-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let sub = base.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(
            base.join(KCL_MOD_FILE),
            "[package]\nname = \"resolve_test\"\n",
        )
        .unwrap();
        std::fs::write(sub.join("main.k"), "a = 1\n").unwrap();

        let (dir, generation) = resolve_mod_dir(&sub.join("main.k")).unwrap();
        assert_eq!(dir, base.canonicalize().unwrap());
        let (_, generation_again) = resolve_mod_dir(&sub.join("main.k")).unwrap();
        assert_eq!(generation, generation_again);

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn auto_updates_dependencies_when_module_not_found() {
        let dir = temp_workspace();
        let (server_rx, client_tx, updates) = spawn_state(
            UpdateCountingToolchain {
                updates: Arc::new(AtomicUsize::new(0)),
                fail: false,
            },
            InitializeParams::default(),
        );
        open_file(&client_tx, &dir.join("main.k"));

        // The missing `nonexistent_pkg` import makes the compile report
        // `CannotFindModule`, which triggers the automatic update.
        wait_for_notification(&server_rx, is_log_containing("Dependencies updated"));
        // The guard prevents further automatic attempts for the same kcl.mod.
        assert_eq!(updates.load(Ordering::SeqCst), 1);

        drop(client_tx);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn update_failure_is_reported_to_the_client() {
        let dir = temp_workspace();
        let (server_rx, client_tx, updates) = spawn_state(
            UpdateCountingToolchain {
                updates: Arc::new(AtomicUsize::new(0)),
                fail: true,
            },
            InitializeParams::default(),
        );
        open_file(&client_tx, &dir.join("main.k"));

        let not = wait_for_notification(&server_rx, |not| {
            not.method == lsp_types::notification::ShowMessage::METHOD
        });
        let params = serde_json::from_value::<lsp_types::ShowMessageParams>(not.params).unwrap();
        assert_eq!(params.typ, lsp_types::MessageType::WARNING);
        assert!(params.message.contains("fake update failure"));
        assert_eq!(updates.load(Ordering::SeqCst), 0);

        drop(client_tx);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn auto_update_can_be_disabled() {
        let dir = temp_workspace();
        let initialize_params = InitializeParams {
            initialization_options: Some(
                serde_json::json!({ "kcl": { "mod": { "autoUpdate": false } } }),
            ),
            ..Default::default()
        };
        let (server_rx, client_tx, updates) = spawn_state(
            UpdateCountingToolchain {
                updates: Arc::new(AtomicUsize::new(0)),
                fail: false,
            },
            initialize_params,
        );
        open_file(&client_tx, &dir.join("main.k"));

        // Wait until the compile has finished and reported diagnostics, then
        // give a potential (unwanted) update time to run.
        // The published diagnostics URI is canonicalized (macOS `/var` ->
        // `/private/var`), so compare against the canonicalized file URL.
        let main_uri = Url::from_file_path(dir.join("main.k").canonicalize().unwrap()).unwrap();
        wait_for_notification(&server_rx, move |not| {
            not.method == lsp_types::notification::PublishDiagnostics::METHOD
                && serde_json::from_value::<lsp_types::PublishDiagnosticsParams>(not.params.clone())
                    .map(|params| params.uri == main_uri && !params.diagnostics.is_empty())
                    .unwrap_or(false)
        });
        std::thread::sleep(Duration::from_millis(1000));
        assert_eq!(updates.load(Ordering::SeqCst), 0);

        drop(client_tx);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn saving_kcl_mod_triggers_dependency_update() {
        let dir = temp_workspace();
        let (server_rx, client_tx, updates) = spawn_state(
            UpdateCountingToolchain {
                updates: Arc::new(AtomicUsize::new(0)),
                fail: false,
            },
            InitializeParams::default(),
        );
        open_file(&client_tx, &dir.join("main.k"));
        // The first update is triggered automatically by the failing import.
        wait_for_notification(&server_rx, is_log_containing("Dependencies updated"));
        assert_eq!(updates.load(Ordering::SeqCst), 1);

        // Saving `kcl.mod` is an explicit user action and runs the update again.
        save_file(&client_tx, &dir.join(KCL_MOD_FILE));
        wait_for_notification(&server_rx, is_log_containing("Dependencies updated"));
        assert_eq!(updates.load(Ordering::SeqCst), 2);

        drop(client_tx);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
