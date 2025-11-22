use compiler_base_span::{Span, DUMMY_SP};
use std::{
    fmt,
    hash::{Hash, Hasher},
};

use crate::session_globals::Interner;
use crate::with_session_globals;

// The proc macro code for this is in `kclvm_macros/src/symbols.rs`.
symbols! {
    // After modifying this list adjust `is_special`, `is_used_keyword`/`is_unused_keyword`,
    // this should be rarely necessary though if the keywords are kept in alphabetic order.
    Keywords {
        // Special reserved identifiers used internally for elided lifetimes,
        // unnamed method parameters, crate root module, error recovery etc.
        Empty:           "",
        As:              "as",
        Import:          "import",
        Rule:            "rule",
        Schema:          "schema",
        Mixin:           "mixin",
        Protocol:        "protocol",
        Check:           "check",
        For:             "for",
        Assert:          "assert",
        If:              "if",
        Elif:            "elif",
        Else:            "else",
        Or:              "or",
        And:             "and",
        Not:             "not",
        In:              "in",
        Is:              "is",
        Lambda:          "lambda",
        All:             "all",
        Any:             "any",
        Filter:          "filter",
        Map:             "map",
        Type:            "type",
        True:            "True",
        False:           "False",
        None:            "None",
        Undefined:       "Undefined",
    }
    // Pre-interned symbols that can be referred to with `kclvm_span::sym::*`.
    Symbols {
        bool,
        float,
        int,
        str,
    }
}

/// Ident denotes a identifier with a symbol name and span
///
/// ```
/// use kclvm_span::*;
/// use compiler_base_span::span::new_byte_pos;
///
/// create_session_globals_then(||{
///     let ident = Ident::new(
///         Symbol::intern("identifier"),
///         Span::new(new_byte_pos(0), new_byte_pos(10)),
///     );
/// })
/// ```
#[derive(Debug, Copy, Clone, Eq)]
pub struct Ident {
    pub name: Symbol,
    pub span: Span,
}

impl std::str::FromStr for Ident {
    type Err = String;
    /// Maps a string to an identifier with a dummy span.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ident::with_dummy_span(Symbol::intern(s)))
    }
}

impl Ident {
    #[inline]
    /// Constructs a new identifier from a symbol and a span.
    pub const fn new(name: Symbol, span: Span) -> Ident {
        Ident { name, span }
    }

    /// Constructs a new identifier with a dummy span.
    #[inline]
    pub const fn with_dummy_span(name: Symbol) -> Ident {
        Ident::new(name, DUMMY_SP)
    }

    /// Maps a string and a span to an identifier.
    pub fn from_str_and_span(string: &str, span: Span) -> Ident {
        Ident::new(Symbol::intern(string), span)
    }

    pub fn without_first_quote(self) -> Ident {
        Ident::new(
            Symbol::intern(self.as_str().trim_start_matches('\'')),
            self.span,
        )
    }

    /// Access the underlying string. This is a slowish operation because it
    /// requires locking the symbol interner.
    ///
    /// Note that the lifetime of the return value is a lie. See
    /// `Symbol::as_str()` for details.
    pub fn as_str(&self) -> String {
        self.name.as_str()
    }
}

impl PartialEq for Ident {
    fn eq(&self, rhs: &Self) -> bool {
        self.name == rhs.name
    }
}

impl Hash for Ident {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.span.hash(state);
    }
}

/// An interned string.
///
/// Internally, a `Symbol` is implemented as an index, and all operations
/// (including hashing, equality, and ordering) operate on that index.
///
/// ```
/// use kclvm_span::*;
/// create_session_globals_then(||{
///     let sym = Symbol::intern("name");
/// });
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Symbol(pub(crate) SymbolIndex);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SymbolIndex {
    pub(crate) idx: u32,
}

impl Symbol {
    pub(crate) const fn new(n: u32) -> Self {
        Symbol(SymbolIndex { idx: n })
    }

    /// Maps a string to its interned representation.
    pub fn intern(string: &str) -> Self {
        with_session_globals(|session_globals| session_globals.symbol_interner.intern(string))
    }

    /// Access the underlying string. This is a slowish operation because it
    /// requires locking the symbol interner.
    ///
    /// Note that the lifetime of the return value is a lie. It's not the same
    /// as `&self`, but actually tied to the lifetime of the underlying
    /// interner. Interners are long-lived, and there are very few of them, and
    /// this function is typically used for short-lived things, so in practice
    /// it works out ok.
    pub fn as_str(&self) -> String {
        with_session_globals(|session_globals| session_globals.symbol_interner.get(*self))
    }

    pub fn as_u32(self) -> u32 {
        self.0.idx
    }

    /// This method is supposed to be used in error messages, so it's expected to be
    /// identical to printing the original identifier token written in source code
    /// (`token_to_string`, `Ident::to_string`), except that symbols don't keep the rawness flag
    /// or edition, so we have to guess the rawness using the global edition.
    pub fn to_ident_string(self) -> String {
        format!("{:?}", Ident::with_dummy_span(self))
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_str(), f)
    }
}

impl From<Symbol> for String {
    fn from(val: Symbol) -> Self {
        val.as_str()
    }
}

// This module has a very short name because it's used a lot.
/// This module contains all the defined keyword `Symbol`s.
///
/// Given that `kw` is imported, use them like `kw::keyword_name`.
/// For example `kw::Loop` or `kw::Break`.
pub mod kw {
    pub use super::kw_generated::*;
}

// This module has a very short name because it's used a lot.
/// This module contains all the defined non-keyword `Symbol`s.
///
/// Given that `sym` is imported, use them like `sym::symbol_name`.
/// For example `sym::rustfmt` or `sym::u8`.
pub mod sym {
    use super::Symbol;
    use std::convert::TryInto;

    #[doc(inline)]
    pub use super::sym_generated::*;

    /// Get the symbol for an integer.
    ///
    /// The first few non-negative integers each have a static symbol and therefore
    /// are fast.
    pub fn integer<N: TryInto<usize> + Copy + ToString>(n: N) -> Symbol {
        if let Result::Ok(idx) = n.try_into() {
            if idx < 10 {
                return Symbol::new(super::SYMBOL_DIGITS_BASE + idx as u32);
            }
        }
        Symbol::intern(&n.to_string())
    }
}

pub mod reserved {

    pub use super::reserved_word;

    pub fn is_reserved_word(word: &str) -> bool {
        reserved_word::reserved_words.contains(&word)
    }
}

/// Special symbols related to KCL keywords.
impl Symbol {
    /// Returns `true` if the symbol is `true` or `false`.
    pub fn is_bool_lit(self) -> bool {
        self == kw::True || self == kw::False
    }
}
