use rustc_span;

pub type BytePos = rustc_span::BytePos;
pub type Span = rustc_span::Span;
pub type SpanData = rustc_span::SpanData;
pub const DUMMY_SP: Span = rustc_span::DUMMY_SP;

pub fn new_byte_pos(arg: u32) -> rustc_span::BytePos {
    rustc_span::BytePos(arg)
}
