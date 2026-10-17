//! A host's panic hook installed before KCL's first call must keep receiving
//! the host's own panics, and none of KCL's, while evaluations run
//! concurrently. This file holds a single test so that its process installs
//! the host hook before anything else installs KCL's.
#![allow(clippy::arc_with_non_send_sync)]

use kcl_parser::ParseSession;
use kcl_runner::{ExecProgramArgs, exec_program};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const FIXTURE: &str = "src/test_issues/github.com/kcl-lang/kcl/2223/main.k";

static HOST_PANICS: AtomicUsize = AtomicUsize::new(0);

#[test]
fn host_hook_survives_concurrent_evaluations() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        HOST_PANICS.fetch_add(1, Ordering::SeqCst);
        default_hook(info);
    }));

    for _ in 0..20 {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                std::thread::spawn(|| {
                    let args = ExecProgramArgs {
                        k_filename_list: vec![FIXTURE.to_string()],
                        ..Default::default()
                    };
                    exec_program(Arc::new(ParseSession::default()), &args).unwrap()
                })
            })
            .collect();
        for handle in handles {
            assert!(!handle.join().unwrap().err_message.is_empty());
        }
    }
    assert_eq!(HOST_PANICS.load(Ordering::SeqCst), 0);

    std::thread::spawn(|| panic!("host panic"))
        .join()
        .unwrap_err();
    assert_eq!(HOST_PANICS.load(Ordering::SeqCst), 1);
}
