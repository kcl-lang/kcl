use rustc_span;

pub type BytePos = rustc_span::BytePos;
pub type Span = rustc_span::Span;
pub type SpanData = rustc_span::SpanData;
pub const DUMMY_SP: Span = rustc_span::DUMMY_SP;

/// New a `BytePos`
///
/// # Examples
///
/// ```rust
/// # use compiler_base_span::span::new_byte_pos;
/// let byte_pos = new_byte_pos(10);
/// ```
#[inline]
pub fn new_byte_pos(arg: u32) -> rustc_span::BytePos {
    rustc_span::BytePos(arg)
}
