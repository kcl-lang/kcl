// Spans are encoded using 1-bit tag and 2 different encoding formats (one for each tag value).
// One format is used for keeping span data inline,
// another contains index into an out-of-line span interner.
// The encoding format for inline spans were obtained by optimizing over crates in rustc/libstd.
// See https://internals.rust-lang.org/t/rfc-compiler-refactoring-spans/1357/28

use crate::{BytePos, SpanData};

/// A compressed span.
///
/// Whereas [`SpanData`] is 12 bytes, which is a bit too big to stick everywhere, `Span`
/// is a form that only takes up 8 bytes, with less space for the length and
/// context. The vast majority (99.9%+) of `SpanData` instances will fit within
/// those 8 bytes; any `SpanData` whose fields don't fit into a `Span` are
/// stored in a separate interner table, and the `Span` will index into that
/// table. Interning is rare enough that the cost is low, but common enough
/// that the code is exercised regularly.
///
/// An earlier version of this code used only 4 bytes for `Span`, but that was
/// slower because only 80--90% of spans could be stored inline (even less in
/// very large crates) and so the interner was used a lot more.
///
/// Inline (compressed) format:
/// - `span.base_or_index == span_data.lo`
/// - `span.len_or_tag == len == span_data.hi - span_data.lo` (must be `<= MAX_LEN`)
/// - `span.ctxt == span_data.ctxt` (must be `<= MAX_CTXT`)
///
/// Interned format:
/// - `span.base_or_index == index` (indexes into the interner table)
/// - `span.len_or_tag == LEN_TAG` (high bit set, all other bits are zero)
/// - `span.ctxt == 0`
///
/// The inline form uses 0 for the tag value (rather than 1) so that we don't
/// need to mask out the tag bit when getting the length, and so that the
/// dummy span can be all zeroes.
///
/// Notes about the choice of field sizes:
/// - `base` is 32 bits in both `Span` and `SpanData`, which means that `base`
///   values never cause interning. The number of bits needed for `base`
///   depends on the crate size. 32 bits allows up to 4 GiB of code in a crate.
/// - `len` is 15 bits in `Span` (a u16, minus 1 bit for the tag) and 32 bits
///   in `SpanData`, which means that large `len` values will cause interning.
///   The number of bits needed for `len` does not depend on the crate size.
///   The most common numbers of bits for `len` are from 0 to 7, with a peak usually
///   at 3 or 4, and then it drops off quickly from 8 onwards. 15 bits is enough
///   for 99.99%+ of cases, but larger values (sometimes 20+ bits) might occur
///   dozens of times in a typical crate.
/// - `ctxt` is 16 bits in `Span` and 32 bits in `SpanData`, which means that
///   large `ctxt` values will cause interning. The number of bits needed for
///   `ctxt` values depend partly on the crate size and partly on the form of
///   the code. No crates in `rustc-perf` need more than 15 bits for `ctxt`,
///   but larger crates might need more than 16 bits.
///
/// In order to reliably use parented spans in incremental compilation,
/// the dependency to the parent definition's span. This is performed
/// using the callback `SPAN_TRACK` to access the query engine.
///
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Span {
    base_or_index: u32,
    len_or_tag: u16,
}

/// Dummy span, both position and length are zero, syntax context is zero as well.
pub const DUMMY_SP: Span = Span {
    base_or_index: 0,
    len_or_tag: 0,
};

impl Span {
    #[inline]
    pub fn new(mut lo: BytePos, mut hi: BytePos) -> Self {
        if lo > hi {
            std::mem::swap(&mut lo, &mut hi);
        }

        let (base, len) = (lo.0, hi.0 - lo.0);

        Span {
            base_or_index: base,
            len_or_tag: len as u16,
        }
    }

    #[inline]
    pub fn data(self) -> SpanData {
        self.data_untracked()
    }

    /// Internal function to translate between an encoded span and the expanded representation.
    /// This function must not be used outside the incremental engine.
    #[inline]
    pub fn data_untracked(self) -> SpanData {
        SpanData {
            lo: BytePos(self.base_or_index),
            hi: BytePos(self.base_or_index + self.len_or_tag as u32),
        }
    }
}
