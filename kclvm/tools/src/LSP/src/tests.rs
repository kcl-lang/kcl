use crossbeam_channel::after;
use crossbeam_channel::select;
use indexmap::IndexMap;
use indexmap::IndexSet;
use kclvm_driver::lookup_compile_workspace;
use kclvm_driver::toolchain;
use kclvm_driver::toolchain::Metadata;
use kclvm_driver::WorkSpaceKind;
use kclvm_sema::core::global_state::GlobalState;
use kclvm_sema::ty::SchemaType;
use kclvm_utils::path::PathPrefix;

use kclvm_sema::resolver::scope::KCLScopeCache;
use lsp_server::RequestId;
use lsp_server::Response;
use lsp_types::notification::Exit;
use lsp_types::request::GotoTypeDefinitionResponse;
use lsp_types::CompletionContext;
use lsp_types::CompletionItem;
use lsp_types::CompletionItemKind;
use lsp_types::CompletionParams;
use lsp_types::CompletionResponse;
use lsp_types::CompletionTriggerKind;
use lsp_types::DocumentFormattingParams;
use lsp_types::DocumentSymbolParams;
use lsp_types::GotoDefinitionParams;
use lsp_types::GotoDefinitionResponse;
use lsp_types::Hover;
use lsp_types::HoverContents;
use lsp_types::HoverParams;
use lsp_types::InitializeParams;
use lsp_types::MarkedString;
use lsp_types::PartialResultParams;
use lsp_types::PublishDiagnosticsParams;
use lsp_types::ReferenceContext;
use lsp_types::ReferenceParams;
use lsp_types::RenameParams;
use lsp_types::SemanticTokensParams;
use lsp_types::TextDocumentIdentifier;
use lsp_types::TextDocumentItem;
use lsp_types::TextDocumentPositionParams;
use lsp_types::TextEdit;
use lsp_types::Url;
use lsp_types::WorkDoneProgressParams;
use lsp_types::WorkspaceEdit;
use lsp_types::WorkspaceFolder;

use parking_lot::lock_api::RwLock;
use serde::Serialize;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use kclvm_ast::ast::Program;
use kclvm_error::Diagnostic as KCLDiagnostic;
use kclvm_error::Position as KCLPos;
use kclvm_parser::KCLModuleCache;

use lsp_types::Diagnostic;
use lsp_types::DiagnosticRelatedInformation;
use lsp_types::DiagnosticSeverity;
use lsp_types::Location;
use lsp_types::NumberOrString;
use lsp_types::{Position, Range, TextDocumentContentChangeEvent};

use proc_macro_crate::bench_test;

use lsp_server::{Connection, Message, Notification, Request};

use crate::compile::compile_with_params;
use crate::completion::completion;
use crate::from_lsp::file_path_from_url;

use crate::app::main_loop;
use crate::compile::Params;
use crate::goto_def::goto_def;
use crate::hover::hover;
use crate::state::KCLGlobalStateCache;
use crate::state::KCLVfs;
use crate::to_lsp::kcl_diag_to_lsp_diags_by_file;
use crate::util::apply_document_changes;
use crate::util::to_json;

macro_rules! wait_async {
    () => {
        thread::sleep(Duration::from_millis(100));
    };
    ($time_ms:expr) => {
        thread::sleep(Duration::from_millis($time_ms));
    };
}

pub(crate) fn compare_goto_res(
    res: Option<GotoTypeDefinitionResponse>,
    pos: (&String, u32, u32, u32, u32),
) {
    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = file_path_from_url(&loc.uri).unwrap();
            assert_eq!(
                got_path.adjust_canonicalization(),
                pos.0.to_string().adjust_canonicalization()
            );

            let (got_start, got_end) = (loc.range.start, loc.range.end);

            let expected_start = Position {
                line: pos.1, // zero-based
                character: pos.2,
            };

            let expected_end = Position {
                line: pos.3, // zero-based
                character: pos.4,
            };
            assert_eq!(got_start, expected_start);
            assert_eq!(got_end, expected_end);
        }
        _ => {
            unreachable!("test error")
        }
    }
}

pub(crate) fn compile_test_file(
    testfile: &str,
) -> (
    String,
    Program,
    IndexSet<KCLDiagnostic>,
    GlobalState,
    IndexMap<String, Vec<SchemaType>>,
) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = path
        .join(testfile)
        .canonicalize()
        .unwrap()
        .display()
        .to_string()
        .adjust_canonicalization();

    let (diags, compile_res) = compile_with_params(Params {
        file: Some(file.clone()),
        module_cache: Some(KCLModuleCache::default()),
        scope_cache: Some(KCLScopeCache::default()),
        vfs: Some(KCLVfs::default()),
        gs_cache: Some(KCLGlobalStateCache::default()),
    });
    let (program, schema_map, gs) = compile_res.unwrap();
    (file, program, diags, gs, schema_map)
}

pub(crate) fn compile_test_file_and_metadata(
    testfile: &str,
) -> (
    String,
    Program,
    IndexSet<KCLDiagnostic>,
    GlobalState,
    Option<Metadata>,
    IndexMap<String, Vec<SchemaType>>,
) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let file = path
        .join(testfile)
        .canonicalize()
        .unwrap()
        .display()
        .to_string()
        .adjust_canonicalization();

    let metadata = lookup_compile_workspace(&toolchain::default(), &file, true).2;
    let (diags, compile_res) = compile_with_params(Params {
        file: Some(file.clone()),
        module_cache: Some(KCLModuleCache::default()),
        scope_cache: Some(KCLScopeCache::default()),
        vfs: Some(KCLVfs::default()),
        gs_cache: Some(KCLGlobalStateCache::default()),
    });
    let (program, schema_map, gs) = compile_res.unwrap();

    (file, program, diags, gs, metadata, schema_map)
}

type Info = (String, (u32, u32, u32, u32), String);

fn build_lsp_diag(
    pos: (u32, u32, u32, u32),
    message: String,
    severity: Option<DiagnosticSeverity>,
    related_info: Vec<Info>,
    code: Option<NumberOrString>,
    data: Option<serde_json::Value>,
) -> Diagnostic {
    let related_information = if related_info.is_empty() {
        None
    } else {
        Some(
            related_info
                .iter()
                .map(|(file, pos, msg)| DiagnosticRelatedInformation {
                    location: Location {
                        uri: Url::from_file_path(file).unwrap(),
                        range: Range {
                            start: Position {
                                line: pos.0,
                                character: pos.1,
                            },
                            end: Position {
                                line: pos.2,
                                character: pos.3,
                            },
                        },
                    },
                    message: msg.clone(),
                })
                .collect(),
        )
    };
    Diagnostic {
        range: lsp_types::Range {
            start: Position {
                line: pos.0,
                character: pos.1,
            },
            end: Position {
                line: pos.2,
                character: pos.3,
            },
        },
        severity,
        code,
        code_description: None,
        source: None,
        message,
        related_information,
        tags: None,
        data,
    }
}

fn build_expect_diags() -> Vec<Diagnostic> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = path.clone();
    test_file.push("src/test_data/diagnostics/diagnostics.k");
    let file = test_file.to_str().unwrap();
    let expected_diags: Vec<Diagnostic> = vec![
        build_lsp_diag(
            (1, 4, 2, 0),
            "expected one of [\"identifier\", \"literal\", \"(\", \"[\", \"{\"] got newline"
                .to_string(),
            Some(DiagnosticSeverity::ERROR),
            vec![],
            Some(NumberOrString::String("InvalidSyntax".to_string())),
            None,
        ),
        build_lsp_diag(
            (0, 0, 0, 10),
            "pkgpath abc not found in the program".to_string(),
            Some(DiagnosticSeverity::ERROR),
            vec![],
            Some(NumberOrString::String("CannotFindModule".to_string())),
            None,
        ),
        build_lsp_diag(
            (0, 0, 0, 10),
            format!(
                "Cannot find the module abc from {}/src/test_data/diagnostics/abc",
                path.to_str().unwrap()
            ),
            Some(DiagnosticSeverity::ERROR),
            vec![],
            Some(NumberOrString::String("CannotFindModule".to_string())),
            None,
        ),
        build_lsp_diag(
            (8, 0, 8, 1),
            "Can not change the value of 'd', because it was declared immutable".to_string(),
            Some(DiagnosticSeverity::ERROR),
            vec![(
                file.to_string(),
                (7, 0, 7, 1),
                "The variable 'd' is declared here".to_string(),
            )],
            Some(NumberOrString::String("ImmutableError".to_string())),
            None,
        ),
        build_lsp_diag(
            (7, 0, 7, 1),
            "The variable 'd' is declared here".to_string(),
            Some(DiagnosticSeverity::ERROR),
            vec![(
                file.to_string(),
                (8, 0, 8, 1),
                "Can not change the value of 'd', because it was declared immutable".to_string(),
            )],
            Some(NumberOrString::String("ImmutableError".to_string())),
            None,
        ),
        build_lsp_diag(
            (2, 0, 2, 1),
            "expected str, got int(1)".to_string(),
            Some(DiagnosticSeverity::ERROR),
            vec![],
            Some(NumberOrString::String("TypeError".to_string())),
            None,
        ),
        build_lsp_diag(
            (10, 8, 10, 10),
            "name 'nu' is not defined, did you mean '[\"number\"]'?".to_string(),
            Some(DiagnosticSeverity::ERROR),
            vec![],
            Some(NumberOrString::String("CompileError".to_string())),
            Some(serde_json::json!({ "suggested_replacement": ["number"] })),
        ),
        build_lsp_diag(
            (0, 0, 0, 10),
            "Module 'abc' imported but unused".to_string(),
            Some(DiagnosticSeverity::WARNING),
            vec![],
            Some(NumberOrString::String("UnusedImportWarning".to_string())),
            None,
        ),
    ];
    expected_diags
}

#[test]
#[bench_test]
fn diagnostics_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = path.clone();
    test_file.push("src/test_data/diagnostics/diagnostics.k");
    let file = test_file.to_str().unwrap();

    let diags = compile_with_params(Params {
        file: Some(file.to_string()),
        module_cache: None,
        scope_cache: None,
        vfs: Some(KCLVfs::default()),
        gs_cache: Some(KCLGlobalStateCache::default()),
    })
    .0;

    let diagnostics = diags
        .iter()
        .flat_map(|diag| kcl_diag_to_lsp_diags_by_file(diag, file))
        .collect::<Vec<Diagnostic>>();

    let expected_diags: Vec<Diagnostic> = build_expect_diags();

    for (get, expected) in diagnostics.iter().zip(expected_diags.iter()) {
        assert_eq!(get, expected)
    }
}

#[test]
#[bench_test]
fn test_apply_document_changes() {
    macro_rules! change {
        [$($sl:expr, $sc:expr; $el:expr, $ec:expr => $text:expr),+] => {
            vec![$(TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position { line: $sl, character: $sc },
                    end: Position { line: $el, character: $ec },
                }),
                range_length: None,
                text: String::from($text),
            }),+]
        };
    }

    let mut text = String::new();
    apply_document_changes(&mut text, vec![]);
    assert_eq!(text, "");

    // Test if full updates work (without a range)
    apply_document_changes(
        &mut text,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: String::from("the"),
        }],
    );

    assert_eq!(text, "the");
    apply_document_changes(&mut text, change![0, 3; 0, 3 => " quick"]);
    assert_eq!(text, "the quick");

    apply_document_changes(&mut text, change![0, 0; 0, 4 => "", 0, 5; 0, 5 => " foxes"]);
    assert_eq!(text, "quick foxes");

    apply_document_changes(&mut text, change![0, 11; 0, 11 => "\ndream"]);
    assert_eq!(text, "quick foxes\ndream");

    apply_document_changes(&mut text, change![1, 0; 1, 0 => "have "]);
    assert_eq!(text, "quick foxes\nhave dream");

    apply_document_changes(
        &mut text,
        change![0, 0; 0, 0 => "the ", 1, 4; 1, 4 => " quiet", 1, 16; 1, 16 => "s\n"],
    );
    assert_eq!(text, "the quick foxes\nhave quiet dreams\n");

    apply_document_changes(
        &mut text,
        change![0, 15; 0, 15 => "\n", 2, 17; 2, 17 => "\n"],
    );
    assert_eq!(text, "the quick foxes\n\nhave quiet dreams\n\n");

    apply_document_changes(
        &mut text,
        change![1, 0; 1, 0 => "DREAM", 2, 0; 2, 0 => "they ", 3, 0; 3, 0 => "DON'T THEY?"],
    );
    assert_eq!(
        text,
        "the quick foxes\nDREAM\nthey have quiet dreams\nDON'T THEY?\n"
    );

    apply_document_changes(&mut text, change![0, 10; 1, 5 => "", 2, 0; 2, 12 => ""]);
    assert_eq!(text, "the quick \nthey have quiet dreams\n");

    text = String::from("❤️");
    apply_document_changes(&mut text, change![0, 0; 0, 0 => "a"]);
    assert_eq!(text, "a❤️");

    // todo: Non-ASCII char
    // text = String::from("a\nb");
    // apply_document_changes(&mut text, change![0, 1; 1, 0 => "\nțc", 0, 1; 1, 1 => "d"]);
    // assert_eq!(text, "adcb");

    // text = String::from("a\nb");
    // apply_document_changes(&mut text, change![0, 1; 1, 0 => "ț\nc", 0, 2; 0, 2 => "c"]);
    // assert_eq!(text, "ațc\ncb");
}

#[test]
#[bench_test]
fn file_path_from_url_test() {
    if cfg!(windows) {
        let url =
            Url::parse("file:///c%3A/Users/abc/Desktop/%E4%B8%AD%E6%96%87/ab%20c/abc.k").unwrap();
        let path = file_path_from_url(&url).unwrap();
        assert_eq!(path, "C:\\Users\\abc\\Desktop\\中文\\ab c\\abc.k");
    } else {
        let url = Url::parse("file:///Users/abc/Desktop/%E4%B8%AD%E6%96%87/ab%20c/abc.k").unwrap();
        let path = file_path_from_url(&url).unwrap();
        assert_eq!(path, "/Users/abc/Desktop/中文/ab c/abc.k");
    }
}

#[test]
fn test_lsp_with_kcl_mod_in_order() {
    goto_import_external_file_test();
    println!("goto_import_external_file_test PASS");
    goto_import_pkg_with_line_test();
    println!("goto_import_pkg_with_line_test PASS");
    complete_import_external_file_test();
    println!("complete_import_external_file_test PASS");
}

fn goto_import_pkg_with_line_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (file, _program, _, gs, _) =
        compile_test_file("src/test_data/goto_def_with_line_test/main_pkg/main.k");
    let pos = KCLPos {
        filename: file.adjust_canonicalization(),
        line: 1,
        column: Some(27),
    };

    let res = goto_def(&pos, &gs);

    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = file_path_from_url(&loc.uri).unwrap();
            let expected_path = path
                .join("src/test_data/goto_def_with_line_test/dep-with-line/main.k")
                .canonicalize()
                .unwrap()
                .display()
                .to_string()
                .adjust_canonicalization();
            assert_eq!(got_path, expected_path)
        }
        _ => {
            unreachable!("test error")
        }
    }
}

fn complete_import_external_file_test() {
    let path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("completion_test")
        .join("import")
        .join("external")
        .join("external_0")
        .join("main.k")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();

    let _ = Command::new("kcl")
        .arg("mod")
        .arg("metadata")
        .arg("--update")
        .current_dir(
            PathBuf::from(".")
                .join("src")
                .join("test_data")
                .join("completion_test")
                .join("import")
                .join("external")
                .join("external_0")
                .canonicalize()
                .unwrap()
                .display()
                .to_string(),
        )
        .output()
        .unwrap();

    let (program, schema_map, gs) = compile_with_params(Params {
        file: Some(path.to_string()),
        module_cache: None,
        scope_cache: None,
        vfs: Some(KCLVfs::default()),
        gs_cache: Some(KCLGlobalStateCache::default()),
    })
    .1
    .unwrap();

    let pos = KCLPos {
        filename: path.to_string().adjust_canonicalization(),
        line: 1,
        column: Some(11),
    };
    let tool = toolchain::default();
    let res = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();

    let got_labels: Vec<String> = match &res {
        CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
        CompletionResponse::List(_) => panic!("test failed"),
    };
    let expected_labels: Vec<&str> = vec![
        "api",
        "apiextensions_apiserver",
        "apimachinery",
        "kube_aggregator",
        "vendor",
    ];
    assert_eq!(got_labels, expected_labels);
}

fn goto_import_external_file_test() {
    let path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("goto_import_def_test")
        .join("main.k")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();

    let _ = Command::new("kcl")
        .arg("mod")
        .arg("metadata")
        .arg("--update")
        .current_dir(
            PathBuf::from(".")
                .join("src")
                .join("test_data")
                .join("goto_import_def_test")
                .canonicalize()
                .unwrap()
                .display()
                .to_string(),
        )
        .output()
        .unwrap();

    let (diags, compile_res) = compile_with_params(Params {
        file: Some(path.to_string()),
        module_cache: None,
        scope_cache: None,
        vfs: Some(KCLVfs::default()),
        gs_cache: Some(KCLGlobalStateCache::default()),
    });
    let gs = compile_res.unwrap().2;

    assert_eq!(diags.len(), 0);

    // test goto import file: import .pkg.schema_def
    let pos = KCLPos {
        filename: path.to_string().adjust_canonicalization(),
        line: 1,
        column: Some(57),
    };
    let res = goto_def(&pos, &gs);
    assert!(res.is_some());
}

// LSP e2e test

/// A `Project` represents a project that a language server can work with. Call the [`server`]
/// method to instantiate a language server that will serve information about the project.
pub struct Project {}

impl Project {
    /// Instantiates a language server for this project.
    pub fn server(self, initialize_params: InitializeParams) -> Server {
        Server::new(initialize_params)
    }
}

/// An object that runs the language server main loop and enables sending and receiving messages
/// to and from it.
pub struct Server {
    next_request_id: Cell<i32>,
    worker: Option<std::thread::JoinHandle<()>>,
    client: Connection,
    messages: RefCell<Vec<Message>>,
}

impl Server {
    /// Constructs and initializes a new `Server`
    pub fn new(initialize_params: InitializeParams) -> Self {
        let (connection, client) = Connection::memory();

        let worker = std::thread::spawn(move || {
            main_loop(connection, initialize_params).unwrap();
        });

        Self {
            next_request_id: Cell::new(1),
            worker: Some(worker),
            client,
            messages: RefCell::new(Vec::new()),
        }
    }

    /// Sends a request to the language server, returning the response
    pub fn send_request<R: lsp_types::request::Request>(&self, params: R::Params) {
        let id = self.next_request_id.get();
        self.next_request_id.set(id.wrapping_add(1));
        let r = Request::new(id.into(), R::METHOD.to_string(), params);
        self.client.sender.send(r.into()).unwrap();
    }

    /// Sends an LSP notification to the main loop.
    pub(crate) fn notification<N: lsp_types::notification::Notification>(&self, params: N::Params)
    where
        N::Params: Serialize,
    {
        let r = Notification::new(N::METHOD.to_string(), params);
        self.send_notification(r);
    }

    /// Sends a server notification to the main loop
    fn send_notification(&self, not: Notification) {
        self.client.sender.send(Message::Notification(not)).unwrap();
        wait_async!();
    }

    /// A function to wait for a specific message to arrive
    fn wait_for_message_cond(&self, n: usize, cond: &dyn Fn(&Message) -> bool) {
        let mut total = 0;
        for msg in self.messages.borrow().iter() {
            if cond(msg) {
                total += 1
            }
        }
        while total < n {
            let msg = self.recv().expect("no response");
            if cond(&msg) {
                total += 1;
            }
        }
    }

    /// Receives a message from the message or timeout.
    pub(crate) fn recv(&self) -> Option<Message> {
        let timeout = Duration::from_secs(5);
        let msg = select! {
            recv(self.client.receiver) -> msg => msg.ok(),
            recv(after(timeout)) -> _ => panic!("timed out"),
        };
        if let Some(ref msg) = msg {
            self.messages.borrow_mut().push(msg.clone());
        }
        msg
    }

    /// Receives a message from the message, if timeout, return None.
    pub(crate) fn recv_without_timeout(&self) -> Option<Message> {
        let timeout = Duration::from_secs(5);
        let msg = select! {
            recv(self.client.receiver) -> msg => msg.ok(),
            recv(after(timeout)) -> _ => return None,
        };
        if let Some(ref msg) = msg {
            self.messages.borrow_mut().push(msg.clone());
        }
        msg
    }

    /// Sends a request to the main loop and receives its response
    fn send_and_receive(&self, r: Request) -> Response {
        let id = r.id.clone();
        self.client.sender.send(r.into()).unwrap();
        while let Some(msg) = self.recv() {
            match msg {
                Message::Request(req) => {
                    panic!("did not expect a request as a response to a request: {req:?}")
                }
                Message::Notification(_) => (),
                Message::Response(res) => {
                    assert_eq!(res.id, id);
                    return res;
                }
            }
        }
        panic!("did not receive a response to our request");
    }

    fn receive_response(&self, id: RequestId) -> Option<Response> {
        while let Some(msg) = self.recv_without_timeout() {
            match msg {
                Message::Request(req) => {
                    panic!("did not expect a request as a response to a request: {req:?}")
                }
                Message::Notification(_) => (),
                Message::Response(res) => {
                    if res.id == id {
                        return Some(res);
                    }
                }
            }
        }
        None
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        // Send the proper shutdown sequence to ensure the main loop terminates properly
        self.notification::<Exit>(());

        // Cancel the main_loop
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}

#[test]
fn notification_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut path = root.clone();

    path.push("src/test_data/diagnostics/diagnostics.k");

    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let server = Project {}.server(InitializeParams::default());

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    // Wait for first "textDocument/publishDiagnostics" notification
    server.wait_for_message_cond(1, &|msg: &Message| match msg {
        Message::Notification(not) => not.method == "textDocument/publishDiagnostics",
        _ => false,
    });

    let msgs = server.messages.borrow();
    for msg in msgs.iter() {
        match msg {
            Message::Notification(not) => {
                if let Some(uri) = not.params.get("uri") {
                    if uri.clone() == to_json(Url::from_file_path(path).unwrap()).unwrap() {
                        assert_eq!(
                            not.params,
                            to_json(PublishDiagnosticsParams {
                                uri: Url::from_file_path(path).unwrap(),
                                diagnostics: build_expect_diags(),
                                version: None,
                            })
                            .unwrap()
                        );
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}

#[test]
fn close_file_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut path = root.clone();

    path.push("src/test_data/diagnostics/diagnostics.k");

    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let server = Project {}.server(InitializeParams::default());

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src.clone(),
            },
        },
    );

    // Mock close file
    server.notification::<lsp_types::notification::DidCloseTextDocument>(
        lsp_types::DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(path).unwrap(),
            },
        },
    );

    // Mock reopen file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );
}

#[test]
fn non_kcl_file_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let server = Project {}.server(InitializeParams::default());
    let mut path = root.clone();
    path.push("src/test_data/diagnostics.kcl");

    // Mock open a Non-KCL file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path.clone()).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: "".to_string(),
            },
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/documentSymbol".to_string(),
        DocumentSymbolParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(path).unwrap(),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);
    assert!(res.result.is_some());
}

#[test]
fn cancel_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut path = root.clone();

    path.push("src/test_data/goto_def_test/goto_def.k");

    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let server = Project {}.server(InitializeParams::default());

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    // send request
    server.send_request::<lsp_types::request::GotoDefinition>(GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(path).unwrap(),
            },
            position: Position::new(23, 9),
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    });

    // cancel request
    server.notification::<lsp_types::notification::Cancel>(lsp_types::CancelParams {
        id: NumberOrString::Number(id),
    });

    assert!(server.receive_response(id.into()).is_none());
}

#[test]
fn goto_def_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.clone();
    let path = path
        .join("src")
        .join("test_data")
        .join("goto_def_test")
        .join("goto_def.k");

    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let server = Project {}.server(InitializeParams::default());

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/definition".to_string(),
        GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(path).unwrap(),
                },
                position: Position::new(23, 9),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);

    assert_eq!(
        res.result.unwrap(),
        to_json(GotoDefinitionResponse::Scalar(Location {
            uri: Url::from_file_path(path).unwrap(),
            range: Range {
                start: Position::new(20, 7),
                end: Position::new(20, 13),
            },
        }))
        .unwrap()
    );
}

#[test]
fn complete_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.clone();
    let path = path
        .join("src")
        .join("test_data")
        .join("completion_test")
        .join("dot")
        .join("completion")
        .join("completion.k");

    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let server = Project {}.server(InitializeParams::default());

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/completion".to_string(),
        CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(path).unwrap(),
                },
                position: Position::new(11, 7),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: Some(CompletionContext {
                trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
                trigger_character: Some(".".to_string()),
            }),
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);

    assert_eq!(
        res.result.unwrap(),
        to_json(CompletionResponse::Array(vec![
            CompletionItem {
                label: "name".to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some("name: str".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "age".to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some("age: int".to_string()),
                ..Default::default()
            }
        ]))
        .unwrap()
    )
}

#[test]
fn complete_with_version_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.clone();
    let path = path
        .join("src")
        .join("test_data")
        .join("completion_test")
        .join("newline")
        .join("newline_with_version")
        .join("newline_with_version.k");
    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let server = Project {}.server(InitializeParams::default());

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    server.notification::<lsp_types::notification::DidChangeTextDocument>(
        lsp_types::DidChangeTextDocumentParams {
            text_document: lsp_types::VersionedTextDocumentIdentifier {
                uri: Url::from_file_path(path).unwrap(),
                version: 1,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "schema Name:\n    name: str\n\nname1 = \"\"\n\nname: Name{\n    \n}"
                    .to_string(),
            }],
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/completion".to_string(),
        CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(path).unwrap(),
                },
                position: Position::new(6, 4),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: Some(CompletionContext {
                trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
                trigger_character: Some("\n".to_string()),
            }),
        },
    );

    let id = r.id.clone();
    server.client.sender.send(r.into()).unwrap();

    while let Some(msg) = server.recv() {
        match msg {
            Message::Request(req) => {
                panic!("did not expect a request as a response to a request: {req:?}")
            }
            Message::Notification(_) => (),
            Message::Response(res) => {
                assert_eq!(res.id, id);
                assert_eq!(
                    res.result.unwrap(),
                    to_json(CompletionResponse::Array(vec![CompletionItem {
                        label: "name".to_string(),
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some("name: str".to_string()),
                        ..Default::default()
                    },]))
                    .unwrap()
                );
                break;
            }
        }
    }
}

#[test]
fn hover_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root
        .join("src")
        .join("test_data")
        .join("hover_test")
        .join("hover.k");
    let path = path.to_str().unwrap();

    let src = std::fs::read_to_string(path).unwrap();
    let server = Project {}.server(InitializeParams::default());

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/hover".to_string(),
        HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(path).unwrap(),
                },
                position: Position::new(15, 7),
            },
            work_done_progress_params: Default::default(),
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);

    assert_eq!(
        res.result.unwrap(),
        to_json(Hover {
            contents: HoverContents::Array(vec![
                MarkedString::String("__main__".to_string()),
                MarkedString::LanguageString(lsp_types::LanguageString {
                    language: "KCL".to_string(),
                    value: "schema Person:\n    name: str\n    age?: int".to_string()
                }),
                MarkedString::String("hover doc test".to_string()),
            ]),
            range: None
        })
        .unwrap()
    )
}

#[test]
fn hover_assign_in_lambda_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut path = root.clone();

    path.push("src/test_data/hover_test/assign_in_lambda.k");

    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let server = Project {}.server(InitializeParams::default());

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/hover".to_string(),
        HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(path).unwrap(),
                },
                position: Position::new(4, 7),
            },
            work_done_progress_params: Default::default(),
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);

    assert_eq!(
        res.result.unwrap(),
        to_json(Hover {
            contents: HoverContents::Scalar(MarkedString::LanguageString(
                lsp_types::LanguageString {
                    language: "KCL".to_string(),
                    value: "images: [str]".to_string()
                }
            )),
            range: None
        })
        .unwrap()
    )
}

#[test]
fn formatting_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut path = root.clone();

    path.push("src/test_data/format/format_range.k");

    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let server = Project {}.server(InitializeParams::default());

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/formatting".to_string(),
        DocumentFormattingParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(path).unwrap(),
            },
            options: Default::default(),
            work_done_progress_params: Default::default(),
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);

    assert_eq!(
        res.result.unwrap(),
        to_json(Some(vec![TextEdit {
            range: Range::new(
                Position::new(0, 0),
                Position::new(i32::MAX as u32, i32::MAX as u32),
            ),
            new_text: "a = 1\n\nb = 2\n\nc = 3\n\nd = 4\n".to_string()
        }]))
        .unwrap()
    )
}

#[test]
fn formatting_unsaved_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut path = root.clone();

    path.push("src/test_data/format/format_range.k");

    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let server = Project {}.server(InitializeParams::default());

    let uri = Url::from_file_path(path).unwrap();

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    // Mock edit file
    server.notification::<lsp_types::notification::DidChangeTextDocument>(
        lsp_types::DidChangeTextDocumentParams {
            text_document: lsp_types::VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 1,
            },
            content_changes: vec![lsp_types::TextDocumentContentChangeEvent {
                range: Some(lsp_types::Range::new(
                    lsp_types::Position::new(0, 0),
                    lsp_types::Position::new(0, 0),
                )),
                range_length: Some(0),
                text: String::from("unsaved = 0\n"),
            }],
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/formatting".to_string(),
        DocumentFormattingParams {
            text_document: TextDocumentIdentifier {
                uri: Url::from_file_path(path).unwrap(),
            },
            options: Default::default(),
            work_done_progress_params: Default::default(),
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);

    assert_eq!(
        res.result.unwrap(),
        to_json(Some(vec![TextEdit {
            range: Range::new(
                Position::new(0, 0),
                Position::new(i32::MAX as u32, i32::MAX as u32),
            ),
            new_text: "unsaved = 0\n\na = 1\n\nb = 2\n\nc = 3\n\nd = 4\n".to_string()
        }]))
        .unwrap()
    )
}

#[test]
fn complete_import_external_file_e2e_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_data")
        .join("completion_test")
        .join("import")
        .join("external")
        .join("external_1");
    let path = root
        .join("main.k")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    let _ = Command::new("kcl")
        .arg("mod")
        .arg("metadata")
        .arg("--update")
        .current_dir(
            PathBuf::from(".")
                .join("src")
                .join("test_data")
                .join("completion_test")
                .join("import")
                .join("external")
                .join("external_1")
                .canonicalize()
                .unwrap()
                .display()
                .to_string(),
        )
        .output()
        .unwrap();

    let src = std::fs::read_to_string(path.clone()).unwrap();

    let initialize_params = InitializeParams {
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: Url::from_file_path(root.clone()).unwrap(),
            name: "test".to_string(),
        }]),
        ..Default::default()
    };
    let server = Project {}.server(initialize_params);

    // FIXME: It takes longer to parse the k8s package on Windows
    #[cfg(target_os = "windows")]
    wait_async!(20000);

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(path.clone()).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );
    wait_async!(5000);

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/completion".to_string(),
        CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(path).unwrap(),
                },
                position: Position::new(0, 7),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);
    match res.result.unwrap() {
        serde_json::Value::Array(vec) => {
            assert!(
                (vec.iter()
                    .find(|v| match v {
                        serde_json::Value::Object(map) => {
                            map.get("label").unwrap() == "k8s"
                        }
                        _ => false,
                    })
                    .is_some()),
                ""
            );
        }
        _ => panic!("test failed"),
    }
}

// TODO: wait for fix `kcl mod metadata` to read only. Otherwise it will lead to an infinite loop
#[allow(dead_code)]
// #[test]
fn mod_file_watcher_test() {
    let path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("watcher")
        .join("modify")
        .canonicalize()
        .unwrap();

    let mod_file_path = path.join("kcl.mod");
    let main_path = path.join("main.k");

    let mod_src_bac = std::fs::read_to_string(mod_file_path.clone()).unwrap();
    let main_src = std::fs::read_to_string(main_path.clone()).unwrap();

    let initialize_params = InitializeParams {
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: Url::from_file_path(path.clone()).unwrap(),
            name: "test".to_string(),
        }]),
        ..Default::default()
    };
    let server = Project {}.server(initialize_params);

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::from_file_path(main_path.clone()).unwrap(),
                language_id: "KCL".to_string(),
                version: 0,
                text: main_src,
            },
        },
    );

    let _ = Command::new("kcl")
        .arg("mod")
        .arg("add")
        .arg("helloworld")
        .current_dir(path)
        .output()
        .unwrap();

    // wait for download dependence
    wait_async!(2000);

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/hover".to_string(),
        HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Url::from_file_path(main_path).unwrap(),
                },
                position: Position::new(0, 8),
            },
            work_done_progress_params: Default::default(),
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);

    std::fs::write(mod_file_path, mod_src_bac).unwrap();
    assert_eq!(
        res.result.unwrap(),
        to_json(Hover {
            contents: HoverContents::Scalar(MarkedString::LanguageString(
                lsp_types::LanguageString {
                    language: "KCL".to_owned(),
                    value: "helloworld: ".to_string(),
                }
            )),
            range: None
        })
        .unwrap()
    )
}

// Integration testing of lsp and konfig
fn konfig_path() -> PathBuf {
    let konfig_path = Path::new(".")
        .canonicalize()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test")
        .join("integration")
        .join("konfig");
    konfig_path
}

#[test]
fn konfig_goto_def_test_base() {
    let konfig_path = konfig_path();
    let base_path = konfig_path
        .clone()
        .join("appops")
        .join("nginx-example")
        .join("base")
        .join("base.k");
    let base_path_str = base_path.to_str().unwrap().to_string();
    let (_program, _, gs) = compile_with_params(Params {
        file: Some(base_path_str.clone()),
        module_cache: None,
        scope_cache: None,
        vfs: Some(KCLVfs::default()),
        gs_cache: Some(KCLGlobalStateCache::default()),
    })
    .1
    .unwrap();

    // schema def
    let pos = KCLPos {
        filename: base_path_str.clone().adjust_canonicalization(),
        line: 7,
        column: Some(30),
    };
    let res = goto_def(&pos, &gs);
    let expected_path = konfig_path
        .clone()
        .join("base")
        .join("pkg")
        .join("kusion_models")
        .join("kube")
        .join("frontend")
        .join("server.k");

    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 12, 7, 12, 13),
    );

    // schema def
    let pos = KCLPos {
        filename: base_path_str.clone().adjust_canonicalization(),
        line: 9,
        column: Some(32),
    };
    let res = goto_def(&pos, &gs);
    let expected_path = konfig_path
        .clone()
        .join("base")
        .join("pkg")
        .join("kusion_models")
        .join("kube")
        .join("frontend")
        .join("container")
        .join("container.k");
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 5, 7, 5, 11),
    );

    // schema attr
    let pos = KCLPos {
        filename: base_path_str.clone().adjust_canonicalization(),
        line: 9,
        column: Some(9),
    };
    let res = goto_def(&pos, &gs);
    let expected_path = konfig_path
        .clone()
        .join("base")
        .join("pkg")
        .join("kusion_models")
        .join("kube")
        .join("frontend")
        .join("server.k");
    compare_goto_res(
        res,
        (
            &expected_path.to_str().unwrap().to_string(),
            115,
            4,
            115,
            17,
        ),
    );

    // schema attr
    let pos = KCLPos {
        filename: base_path_str.clone().adjust_canonicalization(),
        line: 10,
        column: Some(10),
    };
    let res = goto_def(&pos, &gs);
    let expected_path = konfig_path
        .clone()
        .join("base")
        .join("pkg")
        .join("kusion_models")
        .join("kube")
        .join("frontend")
        .join("container")
        .join("container.k");
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 69, 4, 69, 9),
    );

    // import pkg
    let pos = KCLPos {
        filename: base_path_str.clone().adjust_canonicalization(),
        line: 2,
        column: Some(49),
    };
    let res = goto_def(&pos, &gs);

    let expected_path = konfig_path
        .clone()
        .join("base")
        .join("pkg")
        .join("kusion_models")
        .join("kube")
        .join("frontend")
        .join("service")
        .join("service.k");
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 0, 0, 0, 0),
    );
}

#[test]
fn konfig_goto_def_test_main() {
    let konfig_path = konfig_path();
    let main_path = konfig_path
        .clone()
        .join("appops")
        .join("nginx-example")
        .join("dev")
        .join("main.k");
    let main_path_str = main_path.to_str().unwrap().to_string();
    let (_program, _, gs) = compile_with_params(Params {
        file: Some(main_path_str.clone()),
        module_cache: None,
        scope_cache: None,
        vfs: Some(KCLVfs::default()),
        gs_cache: Some(KCLGlobalStateCache::default()),
    })
    .1
    .unwrap();

    // schema def
    let pos = KCLPos {
        filename: main_path_str.clone().adjust_canonicalization(),
        line: 6,
        column: Some(31),
    };
    let res = goto_def(&pos, &gs);
    let expected_path = konfig_path
        .clone()
        .join("base")
        .join("pkg")
        .join("kusion_models")
        .join("kube")
        .join("frontend")
        .join("server.k");
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 12, 7, 12, 13),
    );

    // schema attr
    let pos = KCLPos {
        filename: main_path_str.clone().adjust_canonicalization(),
        line: 7,
        column: Some(14),
    };
    let res = goto_def(&pos, &gs);
    let expected_path = konfig_path
        .clone()
        .join("base")
        .join("pkg")
        .join("kusion_models")
        .join("kube")
        .join("frontend")
        .join("server.k");
    compare_goto_res(
        res,
        (
            &expected_path.to_str().unwrap().to_string(),
            112,
            4,
            112,
            22,
        ),
    );

    // import pkg
    let pos = KCLPos {
        filename: main_path_str.clone().adjust_canonicalization(),
        line: 2,
        column: Some(61),
    };
    let res = goto_def(&pos, &gs);
    let expected_path = konfig_path
        .clone()
        .join("base")
        .join("pkg")
        .join("kusion_models")
        .join("kube")
        .join("templates")
        .join("resource.k");
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 0, 0, 0, 0),
    );
}

#[test]
fn konfig_completion_test_main() {
    let konfig_path = konfig_path();
    let main_path = konfig_path
        .clone()
        .join("appops")
        .join("nginx-example")
        .join("dev")
        .join("main.k");
    let main_path_str = main_path.to_str().unwrap().to_string();
    let (program, schema_map, gs) = compile_with_params(Params {
        file: Some(main_path_str.clone()),
        module_cache: None,
        scope_cache: None,
        vfs: Some(KCLVfs::default()),
        gs_cache: Some(KCLGlobalStateCache::default()),
    })
    .1
    .unwrap();

    // pkg's definition(schema) completion
    let pos = KCLPos {
        filename: main_path_str.clone().adjust_canonicalization(),
        line: 6,
        column: Some(27),
    };
    let tool = toolchain::default();
    let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
    let got_labels: Vec<String> = match got {
        CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
        CompletionResponse::List(_) => panic!("test failed"),
    };

    let expected_labels: Vec<String> = ["Job", "Server"]
        .iter()
        .map(|attr| attr.to_string())
        .collect();
    assert_eq!(got_labels, expected_labels);

    // schema attr completion
    let pos = KCLPos {
        filename: main_path_str.clone().adjust_canonicalization(),
        line: 7,
        column: Some(4),
    };
    let tool = toolchain::default();
    let got = completion(None, &program, &pos, &gs, &tool, None, &schema_map).unwrap();
    let mut got_labels: Vec<String> = match got {
        CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
        CompletionResponse::List(_) => panic!("test failed"),
    };
    let mut attr = [
        "annotations",
        "configMaps",
        "database",
        "enableMonitoring",
        "frontend",
        "image",
        "ingresses",
        "initContainers",
        "labels",
        "mainContainer",
        "name",
        "needNamespace",
        "podMetadata",
        "renderType",
        "replicas",
        "res_tpl",
        "schedulingStrategy",
        "secrets",
        "selector",
        "serviceAccount",
        "services",
        "sidecarContainers",
        "storage",
        "useBuiltInLabels",
        "useBuiltInSelector",
        "volumes",
        "workloadType",
    ];
    got_labels.sort();
    attr.sort();
    assert_eq!(got_labels, attr);

    // import path completion
    let pos = KCLPos {
        filename: main_path_str.clone().adjust_canonicalization(),
        line: 1,
        column: Some(35),
    };
    let tool = toolchain::default();
    let got = completion(Some('.'), &program, &pos, &gs, &tool, None, &schema_map).unwrap();
    let mut got_labels: Vec<String> = match got {
        CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
        CompletionResponse::List(_) => panic!("test failed"),
    };
    let mut pkgs = [
        "common",
        "configmap",
        "container",
        "ingress",
        "job",
        "rbac",
        "resource",
        "secret",
        "server",
        "service",
        "serviceaccount",
        "sidecar",
        "storage",
        "strategy",
        "volume",
    ];
    got_labels.sort();
    pkgs.sort();
    assert_eq!(got_labels, pkgs);
}

#[test]
fn konfig_hover_test_main() {
    let konfig_path = konfig_path();
    let main_path = konfig_path
        .clone()
        .join("appops")
        .join("nginx-example")
        .join("dev")
        .join("main.k");

    let main_path_str = main_path.to_str().unwrap().to_string();
    let (_program, _, gs) = compile_with_params(Params {
        file: Some(main_path_str.clone()),
        module_cache: None,
        scope_cache: None,
        vfs: Some(KCLVfs::default()),
        gs_cache: Some(KCLGlobalStateCache::default()),
    })
    .1
    .unwrap();

    // schema def hover
    let pos = KCLPos {
        filename: main_path_str.clone().adjust_canonicalization(),
        line: 6,
        column: Some(32),
    };
    let got = hover(&pos, &gs).unwrap();
    match got.contents {
        HoverContents::Array(arr) => {
            let expect: Vec<MarkedString> = vec![
                MarkedString::String("base.pkg.kusion_models.kube.frontend".to_string()),
                MarkedString::LanguageString(lsp_types::LanguageString {
                    language: "KCL".to_string(),
                    value: "schema Server:\n    name?: str\n    workloadType: \"Deployment\" | \"StatefulSet\" = \"Deployment\"\n    renderType?: \"Server\" | \"KubeVelaApplication\" = \"Server\"\n    replicas: int = option(\"replicas\") or 1\n    image: str = option(\"image\")\n    schedulingStrategy: SchedulingStrategy = strategy.SchedulingStrategy {}\n    mainContainer: Main\n    sidecarContainers?: [Sidecar]\n    initContainers?: [Sidecar]\n    useBuiltInLabels?: bool = True\n    labels?: {str:str}\n    annotations?: {str:str}\n    useBuiltInSelector?: bool = True\n    selector?: {str:str}\n    podMetadata?: ObjectMeta\n    volumes?: [Volume]\n    needNamespace?: bool = True\n    enableMonitoring?: bool = False\n    configMaps?: [ConfigMap]\n    secrets?: [Secret]\n    services?: [Service]\n    ingresses?: [Ingress]\n    serviceAccount?: ServiceAccount\n    storage?: ObjectStorage\n    database?: DataBase".to_string()
                }),
                MarkedString::String("Server is abstaction of Deployment and StatefulSet.".to_string()),
                MarkedString::LanguageString(lsp_types::LanguageString {
                    language: "KCL".to_string(),
                    value: "import base.pkg.kusion_models.kube.frontend\nimport base.pkg.kusion_models.kube.frontend.container\nimport base.pkg.kusion_models.kube.templates.resource as res_tpl\n\nappConfiguration: frontend.Server {\n    mainContainer = container.Main {\n        name = \"php-redis\"\n        env = [\n            {\n                name = \"GET_HOSTS_FROM\"\n                value = \"dns\"\n            }\n        ]\n        ports = [{containerPort = 80}]\n    }\n    selector = {\n        tier = \"frontend\"\n    }\n    podMetadata.labels: {\n        tier = \"frontend\"\n    }\n    schedulingStrategy.resource = res_tpl.tiny\n}\n".to_string()
                }),
            ];
            assert_eq!(expect, arr);
        }
        _ => unreachable!("test error"),
    }

    // schema attr def hover
    let pos = KCLPos {
        filename: main_path_str.clone().adjust_canonicalization(),
        line: 7,
        column: Some(15),
    };
    let got = hover(&pos, &gs).unwrap();
    match got.contents {
        HoverContents::Array(arr) => {
            let expect: Vec<MarkedString> = vec![
                MarkedString::LanguageString(lsp_types::LanguageString {
                    language: "KCL".to_string(),
                    value:
                        "schedulingStrategy: SchedulingStrategy = strategy.SchedulingStrategy {}"
                            .to_string(),
                }),
                MarkedString::String(
                    "SchedulingStrategy represents scheduling strategy.".to_string(),
                ),
            ];
            assert_eq!(expect, arr);
        }
        _ => unreachable!("test error"),
    }

    // variable hover
    let pos = KCLPos {
        filename: main_path_str.clone().adjust_canonicalization(),
        line: 6,
        column: Some(3),
    };
    let got = hover(&pos, &gs).unwrap();
    match got.contents {
        HoverContents::Scalar(s) => {
            assert_eq!(
                s,
                MarkedString::LanguageString(lsp_types::LanguageString {
                    language: "KCL".to_string(),
                    value: "appConfiguration: Server".to_string()
                })
            );
        }
        _ => unreachable!("test error"),
    }
}

#[test]
fn lsp_version_test() {
    let args = vec!["kcl-language-server".to_string(), "version".to_string()];
    let matches = crate::app::app()
        .arg_required_else_help(false)
        .try_get_matches_from(args);
    match matches {
        Ok(arg_match) => match arg_match.subcommand() {
            Some(("version", _)) => {}
            _ => panic!("test failed"),
        },
        Err(_) => panic!("test failed"),
    }
}

#[test]
fn lsp_run_test() {
    let args = vec!["kcl-language-server".to_string()];
    let matches = crate::app::app()
        .arg_required_else_help(false)
        .try_get_matches_from(args);
    match matches {
        Ok(arg_match) => match arg_match.subcommand() {
            None => {}
            _ => panic!("test failed"),
        },
        Err(_) => panic!("test failed"),
    }
}

#[test]
fn lsp_invalid_subcommand_test() {
    let args = vec!["kcl-language-server".to_string(), "invalid".to_string()];
    let matches = crate::app::app()
        .arg_required_else_help(false)
        .try_get_matches_from(args);
    match matches {
        Ok(_) => panic!("test failed"),
        Err(e) => match e.kind() {
            clap::error::ErrorKind::InvalidSubcommand => {}
            _ => panic!("test failed"),
        },
    }
}

#[test]
fn find_refs_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_data")
        .join("find_refs_test");
    let mut path = root.clone();
    path.push("main.k");

    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let initialize_params = InitializeParams {
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: Url::from_file_path(root.clone()).unwrap(),
            name: "test".to_string(),
        }]),
        ..Default::default()
    };
    let server = Project {}.server(initialize_params);

    // Wait for async build word_index_map
    wait_async!();

    let url = Url::from_file_path(path).unwrap();

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: url.clone(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/references".to_string(),
        ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: url.clone() },
                position: Position::new(0, 1),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: ReferenceContext {
                include_declaration: true,
            },
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);

    assert_eq!(
        res.result.unwrap(),
        to_json(vec![
            Location {
                uri: url.clone(),
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 1),
                },
            },
            Location {
                uri: url.clone(),
                range: Range {
                    start: Position::new(1, 4),
                    end: Position::new(1, 5),
                },
            },
            Location {
                uri: url.clone(),
                range: Range {
                    start: Position::new(2, 4),
                    end: Position::new(2, 5),
                },
            },
            Location {
                uri: url.clone(),
                range: Range {
                    start: Position::new(12, 14),
                    end: Position::new(12, 15),
                },
            },
        ])
        .unwrap()
    );
}

#[test]
fn find_refs_with_file_change_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_data")
        .join("find_refs_test");
    let mut path = root.clone();
    path.push("main.k");

    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let initialize_params = InitializeParams {
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: Url::from_file_path(root.clone()).unwrap(),
            name: "test".to_string(),
        }]),
        ..Default::default()
    };
    let server = Project {}.server(initialize_params);

    // Wait for async build word_index_map
    wait_async!();

    let url = Url::from_file_path(path).unwrap();

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: url.clone(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    // Mock change file content
    server.notification::<lsp_types::notification::DidChangeTextDocument>(
        lsp_types::DidChangeTextDocumentParams {
            text_document: lsp_types::VersionedTextDocumentIdentifier {
                uri: url.clone(),
                version: 1,
            },
            content_changes: vec![lsp_types::TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: r#"a = "demo"

schema Name:
    name: str

schema Person:
    n: Name

p2 = Person {
    n: Name{
        name: a
    }
}"#
                .to_string(),
            }],
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));
    // Mock trigger find references
    let r: Request = Request::new(
        id.into(),
        "textDocument/references".to_string(),
        ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: url.clone() },
                position: Position::new(0, 1),
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: ReferenceContext {
                include_declaration: true,
            },
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);
    assert_eq!(
        res.result.unwrap(),
        to_json(vec![
            Location {
                uri: url.clone(),
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 1),
                },
            },
            Location {
                uri: url.clone(),
                range: Range {
                    start: Position::new(10, 14),
                    end: Position::new(10, 15),
                },
            },
        ])
        .unwrap()
    );
}

#[test]
fn rename_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_data")
        .join("rename_test");
    let path = root.clone();
    let main_path = root.clone();
    let path = path.join("pkg").join("vars.k");
    let main_path = main_path.join("main.k");

    let path = path.to_str().unwrap();
    let src = std::fs::read_to_string(path).unwrap();
    let initialize_params = InitializeParams {
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: Url::from_file_path(root.clone()).unwrap(),
            name: "test".to_string(),
        }]),
        ..Default::default()
    };
    let server = Project {}.server(initialize_params);

    wait_async!();

    let url = Url::from_file_path(path).unwrap();
    let main_url = Url::from_file_path(main_path).unwrap();

    // Mock open file
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: url.clone(),
                language_id: "KCL".to_string(),
                version: 0,
                text: src,
            },
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let new_name = String::from("Person2");
    let r: Request = Request::new(
        id.into(),
        "textDocument/rename".to_string(),
        RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: url.clone() },
                position: Position::new(0, 7),
            },
            new_name: new_name.clone(),
            work_done_progress_params: Default::default(),
        },
    );

    // Send request and wait for it's response
    let res = server.send_and_receive(r);
    let expect = WorkspaceEdit {
        changes: Some(HashMap::from_iter(vec![
            (
                url.clone(),
                vec![
                    TextEdit {
                        range: Range {
                            start: Position::new(0, 7),
                            end: Position::new(0, 13),
                        },
                        new_text: new_name.clone(),
                    },
                    TextEdit {
                        range: Range {
                            start: Position::new(4, 7),
                            end: Position::new(4, 13),
                        },
                        new_text: new_name.clone(),
                    },
                    TextEdit {
                        range: Range {
                            start: Position::new(9, 8),
                            end: Position::new(9, 14),
                        },
                        new_text: new_name.clone(),
                    },
                ],
            ),
            (
                main_url.clone(),
                vec![TextEdit {
                    range: Range {
                        start: Position::new(2, 11),
                        end: Position::new(2, 17),
                    },
                    new_text: new_name.clone(),
                }],
            ),
        ])),
        ..Default::default()
    };
    assert_eq!(res.result.unwrap(), to_json(expect).unwrap());
}

#[test]
fn kcl_workspace_init_kclwork_test() {
    let tool: crate::state::KCLToolChain = Arc::new(RwLock::new(toolchain::default()));
    let tool = Arc::clone(&tool);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_data")
        .join("workspace")
        .join("init");

    let mut work = root.clone();
    work.push("work");

    let (workspaces, failed) =
        kclvm_driver::lookup_compile_workspaces(&*tool.read(), work.to_str().unwrap(), true);

    let mut expected = HashSet::new();

    expected.insert(WorkSpaceKind::Folder(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("test_data")
            .join("workspace")
            .join("init")
            .join("work")
            .join("a"),
    ));

    expected.insert(WorkSpaceKind::Folder(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("test_data")
            .join("workspace")
            .join("init")
            .join("work")
            .join("b"),
    ));

    expected.insert(WorkSpaceKind::File(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("test_data")
            .join("workspace")
            .join("init")
            .join("work")
            .join("c.k"),
    ));

    assert_eq!(expected, workspaces.keys().cloned().collect());

    assert!(failed.is_some());
    assert!(failed.unwrap().is_empty());
}

#[test]
fn kcl_workspace_init_kclmod_test() {
    let tool: crate::state::KCLToolChain = Arc::new(RwLock::new(toolchain::default()));
    let tool = Arc::clone(&tool);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_data")
        .join("workspace")
        .join("init");

    let mut work = root.clone();
    work.push("mod");

    let mut main = work.clone();
    main.push("main.k");

    let (workspaces, failed) =
        kclvm_driver::lookup_compile_workspaces(&*tool.read(), work.to_str().unwrap(), true);

    let mut expected = HashSet::new();

    expected.insert(WorkSpaceKind::ModFile(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("test_data")
            .join("workspace")
            .join("init")
            .join("mod")
            .join("kcl.mod"),
    ));

    assert_eq!(expected, workspaces.keys().cloned().collect());
    assert_eq!(
        vec![main.to_str().unwrap().to_string()],
        workspaces.values().next().unwrap().0
    );
    assert!(failed.is_none());
}

#[test]
fn kcl_workspace_init_settings_file_test() {
    let tool: crate::state::KCLToolChain = Arc::new(RwLock::new(toolchain::default()));
    let tool = Arc::clone(&tool);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_data")
        .join("workspace")
        .join("init");

    let mut work = root.clone();
    work.push("setting");

    let mut a = work.clone();
    a.push("a.k");

    let (workspaces, failed) =
        kclvm_driver::lookup_compile_workspaces(&*tool.read(), work.to_str().unwrap(), true);

    let mut expected = HashSet::new();

    expected.insert(WorkSpaceKind::SettingFile(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("test_data")
            .join("workspace")
            .join("init")
            .join("setting")
            .join("kcl.yaml"),
    ));

    assert_eq!(expected, workspaces.keys().cloned().collect());
    assert!(failed.is_none());
    assert_eq!(
        vec![a.file_name().unwrap().to_str().unwrap().to_string()],
        workspaces.values().next().unwrap().0
    );
}

#[test]
fn kcl_workspace_init_folder_test() {
    let tool: crate::state::KCLToolChain = Arc::new(RwLock::new(toolchain::default()));
    let tool = Arc::clone(&tool);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_data")
        .join("workspace")
        .join("init");

    let mut work = root.clone();
    work.push("folder");

    let (workspaces, failed) =
        kclvm_driver::lookup_compile_workspaces(&*tool.read(), work.to_str().unwrap(), true);

    let mut expected = HashSet::new();

    expected.insert(WorkSpaceKind::Folder(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("test_data")
            .join("workspace")
            .join("init")
            .join("folder"),
    ));

    assert_eq!(expected, workspaces.keys().cloned().collect());

    assert!(failed.is_none());
}

#[test]
fn init_workspace_sema_token_test() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("test_data")
        .join("workspace")
        .join("init")
        .join("folder");

    let mut a_path = root.clone();
    a_path.push("a.k");

    let mut c_path = root.clone();
    c_path.push("sub");
    c_path.push("c.k");

    let a_path = a_path.to_str().unwrap();
    let c_path = c_path.to_str().unwrap();
    let a_src = std::fs::read_to_string(a_path).unwrap();
    let c_src = std::fs::read_to_string(c_path).unwrap();
    let initialize_params = InitializeParams {
        workspace_folders: Some(vec![WorkspaceFolder {
            uri: Url::from_file_path(root.clone()).unwrap(),
            name: "test".to_string(),
        }]),
        ..Default::default()
    };
    let server = Project {}.server(initialize_params);

    let a_url = Url::from_file_path(a_path).unwrap();
    let c_url = Url::from_file_path(c_path).unwrap();

    // Mock open file in init workspace
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: a_url.clone(),
                language_id: "KCL".to_string(),
                version: 0,
                text: a_src,
            },
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/semanticTokens/full".to_string(),
        SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri: a_url },
            work_done_progress_params: WorkDoneProgressParams {
                work_done_token: None,
            },
            partial_result_params: PartialResultParams {
                partial_result_token: None,
            },
        },
    );

    let res = server.send_and_receive(r);
    assert!(res.result.is_some());

    // Mock open file not in init workspace
    server.notification::<lsp_types::notification::DidOpenTextDocument>(
        lsp_types::DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: c_url.clone(),
                language_id: "KCL".to_string(),
                version: 0,
                text: c_src,
            },
        },
    );

    let id = server.next_request_id.get();
    server.next_request_id.set(id.wrapping_add(1));

    let r: Request = Request::new(
        id.into(),
        "textDocument/semanticTokens/full".to_string(),
        SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri: c_url },
            work_done_progress_params: WorkDoneProgressParams {
                work_done_token: None,
            },
            partial_result_params: PartialResultParams {
                partial_result_token: None,
            },
        },
    );

    let res = server.send_and_receive(r);
    assert!(res.result.is_some());
}

#[test]
fn pkg_mod_test() {
    let (_file, _program, diags, _gs, _) =
        compile_test_file("src/test_data/workspace/pkg_mod_test/test/main.k");
    assert_eq!(diags.iter().filter(|diag| diag.is_error()).count(), 0);
}

#[test]
fn aug_assign_without_define() {
    let (_file, _program, diags, _gs, _) =
        compile_test_file("src/test_data/error_code/aug_assign/aug_assign.k");
    assert_eq!(diags.len(), 1);
}
