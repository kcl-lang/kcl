use std::{cell::RefCell, collections::HashMap};

use crate::symbol::Symbol;

/// Per-session global variables: this struct is stored in thread-local storage
/// in such a way that it is accessible without any kind of handle to all
/// threads within the compilation session, but is not accessible outside the
/// session.
///
/// The `kclvm_span::Symbol` uses `SessionGlobals` to implement a fast global
/// string cache.
#[derive(Debug)]
pub struct SessionGlobals {
    pub symbol_interner: Interner,
}

impl SessionGlobals {
    pub fn new() -> SessionGlobals {
        SessionGlobals {
            symbol_interner: Interner::fresh(),
        }
    }
}

impl Default for SessionGlobals {
    fn default() -> Self {
        Self::new()
    }
}

/// Create thread local global session globals
#[inline]
pub fn create_session_globals_then<R>(f: impl FnOnce() -> R) -> R {
    assert!(
        !SESSION_GLOBALS.is_set(),
        "SESSION_GLOBALS should never be overwritten! \
         Use another thread if you need another SessionGlobals"
    );
    let session_globals = SessionGlobals::new();
    SESSION_GLOBALS.set(&session_globals, f)
}

#[inline]
pub fn with_session_globals<R, F>(f: F) -> R
where
    F: FnOnce(&SessionGlobals) -> R,
{
    SESSION_GLOBALS.with(f)
}

// If this ever becomes non thread-local, `decode_syntax_context`
// and `decode_expn_id` will need to be updated to handle concurrent
// deserialization.
scoped_tls::scoped_thread_local!(static SESSION_GLOBALS: SessionGlobals);

#[derive(Debug)]
pub struct Interner(RefCell<InternerInner>);

// This type is private to prevent accidentally constructing more than one
// `Interner` on the same thread, which makes it easy to mixup `Symbol`s
// between `Interner`s.
#[derive(Default, Debug)]
struct InternerInner {
    names: HashMap<String, Symbol>,
    strings: Vec<String>,
}

impl Default for Interner {
    fn default() -> Self {
        Interner(RefCell::new(InternerInner::default()))
    }
}

impl Interner {
    pub fn prefill(init: &[&'static str]) -> Self {
        Interner(RefCell::new(InternerInner {
            strings: init.iter().map(|s| s.to_string()).collect(),
            names: init
                .iter()
                .map(|s| s.to_string())
                .zip((0..).map(Symbol::new))
                .collect(),
        }))
    }

    #[inline]
    pub fn intern(&self, string: &str) -> Symbol {
        let mut inner = self.0.borrow_mut();
        if let Some(&name) = inner.names.get(string) {
            return name;
        }

        let name = Symbol::new(inner.strings.len() as u32);

        inner.strings.push(string.to_string());

        inner.names.insert(string.to_string(), name);
        name
    }

    // Get the symbol as a string. `Symbol::as_str()` should be used in
    // preference to this function.
    pub fn get(&self, symbol: Symbol) -> String {
        self.0.borrow().strings[symbol.0.idx as usize].clone()
    }
}
