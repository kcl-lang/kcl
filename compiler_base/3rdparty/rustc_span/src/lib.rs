//! Source positions and related helper functions.
//!
//! Important concepts in this module include:
//!
//! - the *span*, represented by [`SpanData`] and related types;
//! - source code as represented by a [`SourceMap`]; and
//! - interned strings, represented by [`Symbol`]s, with some common symbols available statically in the [`sym`] module.
//!
//! Unlike most compilers, the span contains not only the position in the source code, but also various other metadata,
//! such as the edition and macro hygiene. This metadata is stored in [`SyntaxContext`] and [`ExpnData`].
//!
//! ## Note
//!
//! This API is completely unstable and subject to change.

mod caching_source_map_view;
pub mod fatal_error;
pub mod source_map;
pub use self::caching_source_map_view::CachingSourceMapView;
use rustc_data_structures::sync::Lrc;
pub use source_map::SourceMap;

mod span_encoding;
pub use span_encoding::{Span, DUMMY_SP};

mod analyze_source_file;

use std::borrow::Cow;
use std::cmp::{self, Ordering};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Range, Sub};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

use md5::Digest;
use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;

use tracing::debug;

// FIXME: We should use this enum or something like it to get rid of the
// use of magic `/rust/1.x/...` paths across the board.
#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd)]
pub enum RealFileName {
    LocalPath(PathBuf),
    /// For remapped paths (namely paths into libstd that have been mapped
    /// to the appropriate spot on the local host's file system, and local file
    /// system paths that have been remapped with `FilePathMapping`),
    Remapped {
        /// `local_path` is the (host-dependent) local path to the file. This is
        /// None if the file was imported from another crate
        local_path: Option<PathBuf>,
        /// `virtual_name` is the stable path rustc will store internally within
        /// build artifacts.
        virtual_name: PathBuf,
    },
}

impl Hash for RealFileName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // To prevent #70924 from happening again we should only hash the
        // remapped (virtualized) path if that exists. This is because
        // virtualized paths to sysroot crates (/rust/$hash or /rust/$version)
        // remain stable even if the corresponding local_path changes
        self.remapped_path_if_available().hash(state)
    }
}

impl RealFileName {
    /// Returns the path suitable for reading from the file system on the local host,
    /// if this information exists.
    /// Avoid embedding this in build artifacts; see `remapped_path_if_available()` for that.
    pub fn local_path(&self) -> Option<&Path> {
        match self {
            RealFileName::LocalPath(p) => Some(p),
            RealFileName::Remapped {
                local_path: p,
                virtual_name: _,
            } => p.as_ref().map(PathBuf::as_path),
        }
    }

    /// Returns the path suitable for reading from the file system on the local host,
    /// if this information exists.
    /// Avoid embedding this in build artifacts; see `remapped_path_if_available()` for that.
    pub fn into_local_path(self) -> Option<PathBuf> {
        match self {
            RealFileName::LocalPath(p) => Some(p),
            RealFileName::Remapped {
                local_path: p,
                virtual_name: _,
            } => p,
        }
    }

    /// Returns the path suitable for embedding into build artifacts. This would still
    /// be a local path if it has not been remapped. A remapped path will not correspond
    /// to a valid file system path: see `local_path_if_available()` for something that
    /// is more likely to return paths into the local host file system.
    pub fn remapped_path_if_available(&self) -> &Path {
        match self {
            RealFileName::LocalPath(p)
            | RealFileName::Remapped {
                local_path: _,
                virtual_name: p,
            } => &p,
        }
    }

    /// Returns the path suitable for reading from the file system on the local host,
    /// if this information exists. Otherwise returns the remapped name.
    /// Avoid embedding this in build artifacts; see `remapped_path_if_available()` for that.
    pub fn local_path_if_available(&self) -> &Path {
        match self {
            RealFileName::LocalPath(path)
            | RealFileName::Remapped {
                local_path: None,
                virtual_name: path,
            }
            | RealFileName::Remapped {
                local_path: Some(path),
                virtual_name: _,
            } => path,
        }
    }

    pub fn to_string_lossy(&self, display_pref: FileNameDisplayPreference) -> Cow<'_, str> {
        match display_pref {
            FileNameDisplayPreference::Local => self.local_path_if_available().to_string_lossy(),
            FileNameDisplayPreference::Remapped => {
                self.remapped_path_if_available().to_string_lossy()
            }
        }
    }
}

/// Differentiates between real files and common virtual files.
#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd, Hash)]
pub enum FileName {
    Real(RealFileName),
    /// Call to `quote!`.
    QuoteExpansion(u64),
    /// Command line.
    Anon(u64),
    /// Hack in `src/librustc_ast/parse.rs`.
    // FIXME(jseyfried)
    MacroExpansion(u64),
    ProcMacroSourceCode(u64),
    /// Strings provided as `--cfg [cfgspec]` stored in a `crate_cfg`.
    CfgSpec(u64),
    /// Strings provided as crate attributes in the CLI.
    CliCrateAttr(u64),
    /// Custom sources for explicit parser calls from plugins and drivers.
    Custom(String),
    DocTest(PathBuf, isize),
    /// Post-substitution inline assembly from LLVM.
    InlineAsm(u64),
}

impl From<PathBuf> for FileName {
    fn from(p: PathBuf) -> Self {
        assert!(!p.to_string_lossy().ends_with('>'));
        FileName::Real(RealFileName::LocalPath(p))
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum FileNameDisplayPreference {
    Remapped,
    Local,
}

pub struct FileNameDisplay<'a> {
    inner: &'a FileName,
    display_pref: FileNameDisplayPreference,
}

impl fmt::Display for FileNameDisplay<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FileName::*;
        match *self.inner {
            Real(ref name) => {
                write!(fmt, "{}", name.to_string_lossy(self.display_pref))
            }
            QuoteExpansion(_) => write!(fmt, "<quote expansion>"),
            MacroExpansion(_) => write!(fmt, "<macro expansion>"),
            Anon(_) => write!(fmt, "<anon>"),
            ProcMacroSourceCode(_) => write!(fmt, "<proc-macro source code>"),
            CfgSpec(_) => write!(fmt, "<cfgspec>"),
            CliCrateAttr(_) => write!(fmt, "<crate attribute>"),
            Custom(ref s) => write!(fmt, "<{}>", s),
            DocTest(ref path, _) => write!(fmt, "{}", path.display()),
            InlineAsm(_) => write!(fmt, "<inline asm>"),
        }
    }
}

impl FileNameDisplay<'_> {
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        match self.inner {
            FileName::Real(ref inner) => inner.to_string_lossy(self.display_pref),
            _ => Cow::from(format!("{}", self)),
        }
    }
}

impl FileName {
    pub fn is_real(&self) -> bool {
        use FileName::*;
        match *self {
            Real(_) => true,
            Anon(_)
            | MacroExpansion(_)
            | ProcMacroSourceCode(_)
            | CfgSpec(_)
            | CliCrateAttr(_)
            | Custom(_)
            | QuoteExpansion(_)
            | DocTest(_, _)
            | InlineAsm(_) => false,
        }
    }

    pub fn prefer_remapped(&self) -> FileNameDisplay<'_> {
        FileNameDisplay {
            inner: self,
            display_pref: FileNameDisplayPreference::Remapped,
        }
    }

    // This may include transient local filesystem information.
    // Must not be embedded in build outputs.
    pub fn prefer_local(&self) -> FileNameDisplay<'_> {
        FileNameDisplay {
            inner: self,
            display_pref: FileNameDisplayPreference::Local,
        }
    }

    pub fn display(&self, display_pref: FileNameDisplayPreference) -> FileNameDisplay<'_> {
        FileNameDisplay {
            inner: self,
            display_pref,
        }
    }
}

/// Represents a span.
///
/// Spans represent a region of code, used for error reporting. Positions in spans
/// are *absolute* positions from the beginning of the [`SourceMap`], not positions
/// relative to [`SourceFile`]s. Methods on the `SourceMap` can be used to relate spans back
/// to the original source.
///
/// You must be careful if the span crosses more than one file, since you will not be
/// able to use many of the functions on spans in source_map and you cannot assume
/// that the length of the span is equal to `span.hi - span.lo`; there may be space in the
/// [`BytePos`] range between files.
///
/// `SpanData` is public because `Span` uses a thread-local interner and can't be
/// sent to other threads, but some pieces of performance infra run in a separate thread.
/// Using `Span` is generally preferred.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct SpanData {
    pub lo: BytePos,
    pub hi: BytePos,
}

// Order spans by position in the file.
impl Ord for SpanData {
    fn cmp(&self, other: &Self) -> Ordering {
        let SpanData { lo: s_lo, hi: s_hi } = self;
        let SpanData { lo: o_lo, hi: o_hi } = other;

        (s_lo, s_hi).cmp(&(o_lo, o_hi))
    }
}

impl PartialOrd for SpanData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl SpanData {
    #[inline]
    pub fn span(&self) -> Span {
        Span::new(self.lo, self.hi)
    }
    #[inline]
    pub fn with_lo(&self, lo: BytePos) -> Span {
        Span::new(lo, self.hi)
    }
    #[inline]
    pub fn with_hi(&self, hi: BytePos) -> Span {
        Span::new(self.lo, hi)
    }
    /// Returns `true` if this is a dummy span with any hygienic context.
    #[inline]
    pub fn is_dummy(self) -> bool {
        self.lo.0 == 0 && self.hi.0 == 0
    }
    /// Returns `true` if `self` fully encloses `other`.
    pub fn contains(self, other: Self) -> bool {
        self.lo <= other.lo && other.hi <= self.hi
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.data(), &rhs.data())
    }
}
impl Ord for Span {
    fn cmp(&self, rhs: &Self) -> Ordering {
        Ord::cmp(&self.data(), &rhs.data())
    }
}

impl Span {
    #[inline]
    pub fn lo(self) -> BytePos {
        self.data().lo
    }
    #[inline]
    pub fn with_lo(self, lo: BytePos) -> Span {
        self.data().with_lo(lo)
    }
    #[inline]
    pub fn hi(self) -> BytePos {
        self.data().hi
    }
    #[inline]
    pub fn with_hi(self, hi: BytePos) -> Span {
        self.data().with_hi(hi)
    }

    /// Returns `true` if this is a dummy span with any hygienic context.
    #[inline]
    pub fn is_dummy(self) -> bool {
        self.data_untracked().is_dummy()
    }

    /// Returns a new span representing an empty span at the beginning of this span.
    #[inline]
    pub fn shrink_to_lo(self) -> Span {
        let span = self.data_untracked();
        span.with_hi(span.lo)
    }
    /// Returns a new span representing an empty span at the end of this span.
    #[inline]
    pub fn shrink_to_hi(self) -> Span {
        let span = self.data_untracked();
        span.with_lo(span.hi)
    }

    #[inline]
    /// Returns `true` if `hi == lo`.
    pub fn is_empty(self) -> bool {
        let span = self.data_untracked();
        span.hi == span.lo
    }

    /// Returns `self` if `self` is not the dummy span, and `other` otherwise.
    pub fn substitute_dummy(self, other: Span) -> Span {
        if self.is_dummy() {
            other
        } else {
            self
        }
    }

    /// Returns `true` if `self` fully encloses `other`.
    pub fn contains(self, other: Span) -> bool {
        let span = self.data();
        let other = other.data();
        span.contains(other)
    }

    /// Returns `true` if `self` touches `other`.
    pub fn overlaps(self, other: Span) -> bool {
        let span = self.data();
        let other = other.data();
        span.lo < other.hi && other.lo < span.hi
    }

    /// Returns `true` if the spans are equal with regards to the source text.
    ///
    /// Use this instead of `==` when either span could be generated code,
    /// and you only care that they point to the same bytes of source text.
    pub fn source_equal(self, other: Span) -> bool {
        let span = self.data();
        let other = other.data();
        span.lo == other.lo && span.hi == other.hi
    }

    /// Returns `Some(span)`, where the start is trimmed by the end of `other`.
    pub fn trim_start(self, other: Span) -> Option<Span> {
        let span = self.data();
        let other = other.data();
        if span.hi > other.hi {
            Some(span.with_lo(cmp::max(span.lo, other.hi)))
        } else {
            None
        }
    }

    /// Returns a `Span` that would enclose both `self` and `end`.
    ///
    /// ```text
    ///     ____             ___
    ///     self lorem ipsum end
    ///     ^^^^^^^^^^^^^^^^^^^^
    /// ```
    pub fn to(self, end: Span) -> Span {
        let span_data = self.data();
        let end_data = end.data();
        Span::new(
            cmp::min(span_data.lo, end_data.lo),
            cmp::max(span_data.hi, end_data.hi),
        )
    }

    /// Returns a `Span` between the end of `self` to the beginning of `end`.
    ///
    /// ```text
    ///     ____             ___
    ///     self lorem ipsum end
    ///         ^^^^^^^^^^^^^
    /// ```
    pub fn between(self, end: Span) -> Span {
        let span = self.data();
        let end = end.data();
        Span::new(span.hi, end.lo)
    }

    /// Returns a `Span` from the beginning of `self` until the beginning of `end`.
    ///
    /// ```text
    ///     ____             ___
    ///     self lorem ipsum end
    ///     ^^^^^^^^^^^^^^^^^
    /// ```
    pub fn until(self, end: Span) -> Span {
        // Most of this function's body is copied from `to`.
        // We can't just do `self.to(end.shrink_to_lo())`,
        // because to also does some magic where it uses min/max so
        // it can handle overlapping spans. Some advanced mis-use of
        // `until` with different ctxts makes this visible.
        let span_data = self.data();
        let end_data = end.data();
        Span::new(span_data.lo, end_data.lo)
    }

    pub fn from_inner(self, inner: InnerSpan) -> Span {
        let span = self.data();
        Span::new(
            span.lo + BytePos::from_usize(inner.start),
            span.lo + BytePos::from_usize(inner.end),
        )
    }
}

impl Default for Span {
    fn default() -> Self {
        DUMMY_SP
    }
}

impl fmt::Debug for SpanData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&Span::new(self.lo, self.hi), f)
    }
}

/// Identifies an offset of a multi-byte character in a `SourceFile`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct MultiByteChar {
    /// The absolute offset of the character in the `SourceMap`.
    pub pos: BytePos,
    /// The number of bytes, `>= 2`.
    pub bytes: u8,
}

/// Identifies an offset of a non-narrow character in a `SourceFile`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NonNarrowChar {
    /// Represents a zero-width character.
    ZeroWidth(BytePos),
    /// Represents a wide (full-width) character.
    Wide(BytePos),
    /// Represents a tab character, represented visually with a width of 4 characters.
    Tab(BytePos),
}

impl NonNarrowChar {
    fn new(pos: BytePos, width: usize) -> Self {
        match width {
            0 => NonNarrowChar::ZeroWidth(pos),
            2 => NonNarrowChar::Wide(pos),
            4 => NonNarrowChar::Tab(pos),
            _ => panic!("width {} given for non-narrow character", width),
        }
    }

    /// Returns the absolute offset of the character in the `SourceMap`.
    pub fn pos(&self) -> BytePos {
        match *self {
            NonNarrowChar::ZeroWidth(p) | NonNarrowChar::Wide(p) | NonNarrowChar::Tab(p) => p,
        }
    }

    /// Returns the width of the character, 0 (zero-width) or 2 (wide).
    pub fn width(&self) -> usize {
        match *self {
            NonNarrowChar::ZeroWidth(_) => 0,
            NonNarrowChar::Wide(_) => 2,
            NonNarrowChar::Tab(_) => 4,
        }
    }
}

impl Add<BytePos> for NonNarrowChar {
    type Output = Self;

    fn add(self, rhs: BytePos) -> Self {
        match self {
            NonNarrowChar::ZeroWidth(pos) => NonNarrowChar::ZeroWidth(pos + rhs),
            NonNarrowChar::Wide(pos) => NonNarrowChar::Wide(pos + rhs),
            NonNarrowChar::Tab(pos) => NonNarrowChar::Tab(pos + rhs),
        }
    }
}

impl Sub<BytePos> for NonNarrowChar {
    type Output = Self;

    fn sub(self, rhs: BytePos) -> Self {
        match self {
            NonNarrowChar::ZeroWidth(pos) => NonNarrowChar::ZeroWidth(pos - rhs),
            NonNarrowChar::Wide(pos) => NonNarrowChar::Wide(pos - rhs),
            NonNarrowChar::Tab(pos) => NonNarrowChar::Tab(pos - rhs),
        }
    }
}

/// Identifies an offset of a character that was normalized away from `SourceFile`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct NormalizedPos {
    /// The absolute offset of the character in the `SourceMap`.
    pub pos: BytePos,
    /// The difference between original and normalized string at position.
    pub diff: u32,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ExternalSource {
    /// No external source has to be loaded, since the `SourceFile` represents a local crate.
    Unneeded,
    Foreign {
        kind: ExternalSourceKind,
        /// This SourceFile's byte-offset within the source_map of its original crate.
        original_start_pos: BytePos,
        /// The end of this SourceFile within the source_map of its original crate.
        original_end_pos: BytePos,
    },
}

/// The state of the lazy external source loading mechanism of a `SourceFile`.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ExternalSourceKind {
    /// The external source has been loaded already.
    Present(Rc<String>),
    /// No attempt has been made to load the external source.
    AbsentOk,
    /// A failed attempt has been made to load the external source.
    AbsentErr,
    Unneeded,
}

impl ExternalSource {
    pub fn get_source(&self) -> Option<&Rc<String>> {
        match self {
            ExternalSource::Foreign {
                kind: ExternalSourceKind::Present(ref src),
                ..
            } => Some(src),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct OffsetOverflowError;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceFileHashAlgorithm {
    Md5,
    Sha1,
    Sha256,
}

impl FromStr for SourceFileHashAlgorithm {
    type Err = ();

    fn from_str(s: &str) -> Result<SourceFileHashAlgorithm, ()> {
        match s {
            "md5" => Ok(SourceFileHashAlgorithm::Md5),
            "sha1" => Ok(SourceFileHashAlgorithm::Sha1),
            "sha256" => Ok(SourceFileHashAlgorithm::Sha256),
            _ => Err(()),
        }
    }
}

/// The hash of the on-disk source file used for debug info.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SourceFileHash {
    pub kind: SourceFileHashAlgorithm,
    value: [u8; 32],
}

impl SourceFileHash {
    pub fn new(kind: SourceFileHashAlgorithm, src: &str) -> SourceFileHash {
        let mut hash = SourceFileHash {
            kind,
            value: Default::default(),
        };
        let len = hash.hash_len();
        let value = &mut hash.value[..len];
        let data = src.as_bytes();
        match kind {
            SourceFileHashAlgorithm::Md5 => {
                value.copy_from_slice(&Md5::digest(data));
            }
            SourceFileHashAlgorithm::Sha1 => {
                value.copy_from_slice(&Sha1::digest(data));
            }
            SourceFileHashAlgorithm::Sha256 => {
                value.copy_from_slice(&Sha256::digest(data));
            }
        }
        hash
    }

    /// Check if the stored hash matches the hash of the string.
    pub fn matches(&self, src: &str) -> bool {
        Self::new(self.kind, src) == *self
    }

    /// The bytes of the hash.
    pub fn hash_bytes(&self) -> &[u8] {
        let len = self.hash_len();
        &self.value[..len]
    }

    fn hash_len(&self) -> usize {
        match self.kind {
            SourceFileHashAlgorithm::Md5 => 16,
            SourceFileHashAlgorithm::Sha1 => 20,
            SourceFileHashAlgorithm::Sha256 => 32,
        }
    }
}

/// A single source in the [`SourceMap`].
#[derive(Clone)]
pub struct SourceFile {
    /// The name of the file that the source came from. Source that doesn't
    /// originate from files has names between angle brackets by convention
    /// (e.g., `<anon>`).
    pub name: FileName,
    /// The complete source code.
    pub src: Option<Rc<String>>,
    /// The source code's hash.
    pub src_hash: SourceFileHash,
    /// The start position of this source in the `SourceMap`.
    pub start_pos: BytePos,
    /// The end position of this source in the `SourceMap`.
    pub end_pos: BytePos,
    /// Locations of lines beginnings in the source code.
    pub lines: Vec<BytePos>,
    /// Locations of multi-byte characters in the source code.
    pub multibyte_chars: Vec<MultiByteChar>,
    /// Width of characters that are not narrow in the source code.
    pub non_narrow_chars: Vec<NonNarrowChar>,
    /// Locations of characters removed during normalization.
    pub normalized_pos: Vec<NormalizedPos>,
    /// A hash of the filename, used for speeding up hashing in incremental compilation.
    pub name_hash: u64,
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "SourceFile({:?})", self.name)
    }
}

impl SourceFile {
    pub fn new(
        name: FileName,
        mut src: String,
        start_pos: BytePos,
        hash_kind: SourceFileHashAlgorithm,
    ) -> Self {
        // Compute the file hash before any normalization.
        let src_hash = SourceFileHash::new(hash_kind, &src);
        let normalized_pos = normalize_src(&mut src, start_pos);

        let name_hash = {
            let mut hasher = DefaultHasher::new();
            name.hash(&mut hasher);
            hasher.finish()
        };
        let end_pos = start_pos.to_usize() + src.len();
        assert!(end_pos <= u32::MAX as usize);

        let (lines, multibyte_chars, non_narrow_chars) =
            analyze_source_file::analyze_source_file(&src, start_pos);

        SourceFile {
            name,
            src: Some(Rc::new(src)),
            src_hash,
            start_pos,
            end_pos: Pos::from_usize(end_pos),
            lines,
            multibyte_chars,
            non_narrow_chars,
            normalized_pos,
            name_hash,
        }
    }

    /// Returns the `BytePos` of the beginning of the current line.
    pub fn line_begin_pos(&self, pos: BytePos) -> BytePos {
        let line_index = self.lookup_line(pos).unwrap();
        self.lines[line_index]
    }

    /// Gets a line from the list of pre-computed line-beginnings.
    /// The line number here is 0-based.
    pub fn get_line(&self, line_number: usize) -> Option<Cow<'_, str>> {
        fn get_until_newline(src: &str, begin: usize) -> &str {
            // We can't use `lines.get(line_number+1)` because we might
            // be parsing when we call this function and thus the current
            // line is the last one we have line info for.
            let slice = &src[begin..];
            match slice.find('\n') {
                Some(e) => &slice[..e],
                None => slice,
            }
        }

        let begin = {
            let line = self.lines.get(line_number)?;
            let begin: BytePos = *line - self.start_pos;
            begin.to_usize()
        };

        if let Some(ref src) = self.src {
            Some(Cow::from(get_until_newline(src, begin)))
        } else {
            None
        }
    }

    pub fn is_real_file(&self) -> bool {
        self.name.is_real()
    }

    pub fn is_imported(&self) -> bool {
        self.src.is_none()
    }

    pub fn count_lines(&self) -> usize {
        self.lines.len()
    }

    /// Finds the line containing the given position. The return value is the
    /// index into the `lines` array of this `SourceFile`, not the 1-based line
    /// number. If the source_file is empty or the position is located before the
    /// first line, `None` is returned.
    pub fn lookup_line(&self, pos: BytePos) -> Option<usize> {
        match self.lines.binary_search(&pos) {
            Ok(idx) => Some(idx),
            Err(0) => None,
            Err(idx) => Some(idx - 1),
        }
    }

    pub fn line_bounds(&self, line_index: usize) -> Range<BytePos> {
        if self.is_empty() {
            return self.start_pos..self.end_pos;
        }

        assert!(line_index < self.lines.len());
        if line_index == (self.lines.len() - 1) {
            self.lines[line_index]..self.end_pos
        } else {
            self.lines[line_index]..self.lines[line_index + 1]
        }
    }

    /// Returns whether or not the file contains the given `SourceMap` byte
    /// position. The position one past the end of the file is considered to be
    /// contained by the file. This implies that files for which `is_empty`
    /// returns true still contain one byte position according to this function.
    #[inline]
    pub fn contains(&self, byte_pos: BytePos) -> bool {
        byte_pos >= self.start_pos && byte_pos <= self.end_pos
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start_pos == self.end_pos
    }

    /// Calculates the original byte position relative to the start of the file
    /// based on the given byte position.
    pub fn original_relative_byte_pos(&self, pos: BytePos) -> BytePos {
        // Diff before any records is 0. Otherwise use the previously recorded
        // diff as that applies to the following characters until a new diff
        // is recorded.
        let diff = match self.normalized_pos.binary_search_by(|np| np.pos.cmp(&pos)) {
            Ok(i) => self.normalized_pos[i].diff,
            Err(i) if i == 0 => 0,
            Err(i) => self.normalized_pos[i - 1].diff,
        };

        BytePos::from_u32(pos.0 - self.start_pos.0 + diff)
    }

    /// Converts an absolute `BytePos` to a `CharPos` relative to the `SourceFile`.
    pub fn bytepos_to_file_charpos(&self, bpos: BytePos) -> CharPos {
        // The number of extra bytes due to multibyte chars in the `SourceFile`.
        let mut total_extra_bytes = 0;

        for mbc in self.multibyte_chars.iter() {
            debug!("{}-byte char at {:?}", mbc.bytes, mbc.pos);
            if mbc.pos < bpos {
                // Every character is at least one byte, so we only
                // count the actual extra bytes.
                total_extra_bytes += mbc.bytes as u32 - 1;
                // We should never see a byte position in the middle of a
                // character.
                assert!(bpos.to_u32() >= mbc.pos.to_u32() + mbc.bytes as u32);
            } else {
                break;
            }
        }

        assert!(self.start_pos.to_u32() + total_extra_bytes <= bpos.to_u32());
        CharPos(bpos.to_usize() - self.start_pos.to_usize() - total_extra_bytes as usize)
    }

    /// Looks up the file's (1-based) line number and (0-based `CharPos`) column offset, for a
    /// given `BytePos`.
    pub fn lookup_file_pos(&self, pos: BytePos) -> (usize, CharPos) {
        let chpos = self.bytepos_to_file_charpos(pos);
        match self.lookup_line(pos) {
            Some(a) => {
                let line = a + 1; // Line numbers start at 1
                let linebpos = self.lines[a];
                let linechpos = self.bytepos_to_file_charpos(linebpos);
                let col = chpos - linechpos;
                debug!(
                    "byte pos {:?} is on the line at byte pos {:?}",
                    pos, linebpos
                );
                debug!(
                    "char pos {:?} is on the line at char pos {:?}",
                    chpos, linechpos
                );
                debug!("byte is on line: {}", line);
                assert!(chpos >= linechpos);
                (line, col)
            }
            None => (0, chpos),
        }
    }

    /// Looks up the file's (1-based) line number, (0-based `CharPos`) column offset, and (0-based)
    /// column offset when displayed, for a given `BytePos`.
    pub fn lookup_file_pos_with_col_display(&self, pos: BytePos) -> (usize, CharPos, usize) {
        let (line, col_or_chpos) = self.lookup_file_pos(pos);
        if line > 0 {
            let col = col_or_chpos;
            let linebpos = self.lines[line - 1];
            let col_display = {
                let start_width_idx = self
                    .non_narrow_chars
                    .binary_search_by_key(&linebpos, |x| x.pos())
                    .unwrap_or_else(|x| x);
                let end_width_idx = self
                    .non_narrow_chars
                    .binary_search_by_key(&pos, |x| x.pos())
                    .unwrap_or_else(|x| x);
                let special_chars = end_width_idx - start_width_idx;
                let non_narrow: usize = self.non_narrow_chars[start_width_idx..end_width_idx]
                    .iter()
                    .map(|x| x.width())
                    .sum();
                col.0 - special_chars + non_narrow
            };
            (line, col, col_display)
        } else {
            let chpos = col_or_chpos;
            let col_display = {
                let end_width_idx = self
                    .non_narrow_chars
                    .binary_search_by_key(&pos, |x| x.pos())
                    .unwrap_or_else(|x| x);
                let non_narrow: usize = self.non_narrow_chars[0..end_width_idx]
                    .iter()
                    .map(|x| x.width())
                    .sum();
                chpos.0 - end_width_idx + non_narrow
            };
            (0, chpos, col_display)
        }
    }
}

/// Normalizes the source code and records the normalizations.
fn normalize_src(src: &mut String, start_pos: BytePos) -> Vec<NormalizedPos> {
    let mut normalized_pos = vec![];
    remove_bom(src, &mut normalized_pos);
    normalize_newlines(src, &mut normalized_pos);

    // Offset all the positions by start_pos to match the final file positions.
    for np in &mut normalized_pos {
        np.pos.0 += start_pos.0;
    }

    normalized_pos
}

/// Removes UTF-8 BOM, if any.
fn remove_bom(src: &mut String, normalized_pos: &mut Vec<NormalizedPos>) {
    if src.starts_with('\u{feff}') {
        src.drain(..3);
        normalized_pos.push(NormalizedPos {
            pos: BytePos(0),
            diff: 3,
        });
    }
}

/// Replaces `\r\n` with `\n` in-place in `src`.
///
/// Returns error if there's a lone `\r` in the string.
fn normalize_newlines(src: &mut String, normalized_pos: &mut Vec<NormalizedPos>) {
    if !src.as_bytes().contains(&b'\r') {
        return;
    }

    // We replace `\r\n` with `\n` in-place, which doesn't break utf-8 encoding.
    // While we *can* call `as_mut_vec` and do surgery on the live string
    // directly, let's rather steal the contents of `src`. This makes the code
    // safe even if a panic occurs.

    let mut buf = std::mem::replace(src, String::new()).into_bytes();
    let mut gap_len = 0;
    let mut tail = buf.as_mut_slice();
    let mut cursor = 0;
    let original_gap = normalized_pos.last().map_or(0, |l| l.diff);
    loop {
        let idx = match find_crlf(&tail[gap_len..]) {
            None => tail.len(),
            Some(idx) => idx + gap_len,
        };
        tail.copy_within(gap_len..idx, 0);
        tail = &mut tail[idx - gap_len..];
        if tail.len() == gap_len {
            break;
        }
        cursor += idx - gap_len;
        gap_len += 1;
        normalized_pos.push(NormalizedPos {
            pos: BytePos::from_usize(cursor + 1),
            diff: original_gap + gap_len as u32,
        });
    }

    // Account for removed `\r`.
    // After `set_len`, `buf` is guaranteed to contain utf-8 again.
    let new_len = buf.len() - gap_len;
    unsafe {
        buf.set_len(new_len);
        *src = String::from_utf8_unchecked(buf);
    }

    fn find_crlf(src: &[u8]) -> Option<usize> {
        let mut search_idx = 0;
        while let Some(idx) = find_cr(&src[search_idx..]) {
            if src[search_idx..].get(idx + 1) != Some(&b'\n') {
                search_idx += idx + 1;
                continue;
            }
            return Some(search_idx + idx);
        }
        None
    }

    fn find_cr(src: &[u8]) -> Option<usize> {
        src.iter().position(|&b| b == b'\r')
    }
}

// _____________________________________________________________________________
// Pos, BytePos, CharPos
//

pub trait Pos {
    fn from_usize(n: usize) -> Self;
    fn to_usize(&self) -> usize;
    fn from_u32(n: u32) -> Self;
    fn to_u32(&self) -> u32;
}

macro_rules! impl_pos {
    (
        $(
            $(#[$attr:meta])*
            $vis:vis struct $ident:ident($inner_vis:vis $inner_ty:ty);
        )*
    ) => {
        $(
            $(#[$attr])*
            $vis struct $ident($inner_vis $inner_ty);

            impl Pos for $ident {
                #[inline(always)]
                fn from_usize(n: usize) -> $ident {
                    $ident(n as $inner_ty)
                }

                #[inline(always)]
                fn to_usize(&self) -> usize {
                    self.0 as usize
                }

                #[inline(always)]
                fn from_u32(n: u32) -> $ident {
                    $ident(n as $inner_ty)
                }

                #[inline(always)]
                fn to_u32(&self) -> u32 {
                    self.0 as u32
                }
            }

            impl Add for $ident {
                type Output = $ident;

                #[inline(always)]
                fn add(self, rhs: $ident) -> $ident {
                    $ident(self.0 + rhs.0)
                }
            }

            impl Sub for $ident {
                type Output = $ident;

                #[inline(always)]
                fn sub(self, rhs: $ident) -> $ident {
                    $ident(self.0 - rhs.0)
                }
            }

            impl fmt::Display for $ident {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{}", self.0)
                }
            }
        )*
    };
}

impl_pos! {
    /// A byte offset.
    ///
    /// Keep this small (currently 32-bits), as AST contains a lot of them.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
    pub struct BytePos(pub u32);

    /// A character offset.
    ///
    /// Because of multibyte UTF-8 characters, a byte offset
    /// is not equivalent to a character offset. The [`SourceMap`] will convert [`BytePos`]
    /// values to `CharPos` values as necessary.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
    pub struct CharPos(pub usize);
}

impl BytePos {}

// _____________________________________________________________________________
// Loc, SourceFileAndLine, SourceFileAndBytePos
//

/// A source code location used for error reporting.
#[derive(Debug, Clone)]
pub struct Loc {
    /// Information about the original source.
    pub file: Lrc<SourceFile>,
    /// The (1-based) line number.
    pub line: usize,
    /// The (0-based) column offset.
    pub col: CharPos,
    /// The (0-based) column offset when displayed.
    pub col_display: usize,
}

// Used to be structural records.
#[derive(Debug)]
pub struct SourceFileAndLine {
    pub sf: Lrc<SourceFile>,
    /// Index of line, starting from 0.
    pub line: usize,
}
#[derive(Debug)]
pub struct SourceFileAndBytePos {
    pub sf: Lrc<SourceFile>,
    pub pos: BytePos,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LineInfo {
    /// Index of line, starting from 0.
    pub line_index: usize,

    /// Column in line where span begins, starting from 0.
    pub start_col: CharPos,

    /// Column in line where span ends, starting from 0, exclusive.
    pub end_col: CharPos,
}

pub struct FileLines {
    pub file: Lrc<SourceFile>,
    pub lines: Vec<LineInfo>,
}

// _____________________________________________________________________________
// SpanLinesError, SpanSnippetError, DistinctSources, MalformedSourceMapPositions
//

pub type FileLinesResult = Result<FileLines, SpanLinesError>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SpanLinesError {
    DistinctSources(DistinctSources),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SpanSnippetError {
    IllFormedSpan(Span),
    DistinctSources(DistinctSources),
    MalformedForSourcemap(MalformedSourceMapPositions),
    SourceNotAvailable { filename: FileName },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DistinctSources {
    pub begin: (FileName, BytePos),
    pub end: (FileName, BytePos),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MalformedSourceMapPositions {
    pub name: FileName,
    pub source_len: usize,
    pub begin_pos: BytePos,
    pub end_pos: BytePos,
}

/// Range inside of a `Span` used for diagnostics when we only have access to relative positions.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct InnerSpan {
    pub start: usize,
    pub end: usize,
}

impl InnerSpan {
    pub fn new(start: usize, end: usize) -> InnerSpan {
        InnerSpan { start, end }
    }
}
