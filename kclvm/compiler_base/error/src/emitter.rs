use crate::diagnostic::Diagnostic;

use compiler_base_span::SourceMap;
use rustc_errors::{
    styled_buffer::{StyledBuffer, StyledString},
    Style,
};
use std::io::{self, Write};
use std::sync::Arc;
use termcolor::{BufferWriter, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Emitter trait for emitting errors.
pub trait Emitter<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    fn format_diagnostic(&mut self, diag: &Diagnostic<T>) -> StyledBuffer<T>;
    /// Emit a structured diagnostic.
    fn emit_diagnostic(&mut self, diag: &Diagnostic<T>);
    /// Checks if we can use colors in the current output stream.
    fn supports_color(&self) -> bool {
        false
    }
}

/// Emitter writer.
pub struct EmitterWriter {
    dst: Destination,
    short_message: bool,
    source_map: Option<Arc<SourceMap>>,
}

impl Default for EmitterWriter {
    fn default() -> Self {
        Self {
            dst: Destination::from_stderr(),
            short_message: false,
            source_map: None,
        }
    }
}

impl EmitterWriter {
    pub fn from_stderr(source_map: Arc<SourceMap>) -> Self {
        Self {
            dst: Destination::from_stderr(),
            short_message: false,
            source_map: Some(source_map),
        }
    }
}

/// Emit destinations
pub enum Destination {
    Terminal(StandardStream),
    Buffered(BufferWriter),
    // The bool denotes whether we should be emitting ansi color codes or not
    Raw(Box<(dyn Write + Send)>, bool),
}

impl Destination {
    #[allow(dead_code)]
    pub fn from_raw(dst: Box<dyn Write + Send>, colored: bool) -> Self {
        Destination::Raw(dst, colored)
    }

    pub fn from_stderr() -> Self {
        // On Windows we'll be performing global synchronization on the entire
        // system for emitting rustc errors, so there's no need to buffer
        // anything.
        //
        // On non-Windows we rely on the atomicity of `write` to ensure errors
        // don't get all jumbled up.
        if !cfg!(windows) {
            Destination::Terminal(StandardStream::stderr(ColorChoice::Auto))
        } else {
            Destination::Buffered(BufferWriter::stderr(ColorChoice::Auto))
        }
    }

    fn supports_color(&self) -> bool {
        match *self {
            Self::Terminal(ref stream) => stream.supports_color(),
            Self::Buffered(ref buffer) => buffer.buffer().supports_color(),
            Self::Raw(_, supports_color) => supports_color,
        }
    }

    fn set_color(&mut self, color: &ColorSpec) -> io::Result<()> {
        match *self {
            Destination::Terminal(ref mut t) => t.set_color(color),
            Destination::Buffered(ref mut t) => t.buffer().set_color(color),
            Destination::Raw(_, _) => Ok(()),
        }
    }

    fn reset(&mut self) -> io::Result<()> {
        match *self {
            Destination::Terminal(ref mut t) => t.reset(),
            Destination::Buffered(ref mut t) => t.buffer().reset(),
            Destination::Raw(..) => Ok(()),
        }
    }
}

impl<'a> Write for Destination {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match *self {
            Destination::Terminal(ref mut t) => t.write(bytes),
            Destination::Buffered(ref mut t) => t.buffer().write(bytes),
            Destination::Raw(ref mut t, _) => t.write(bytes),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            Destination::Terminal(ref mut t) => t.flush(),
            Destination::Buffered(ref mut t) => t.buffer().flush(),
            Destination::Raw(ref mut t, _) => t.flush(),
        }
    }
}

impl<T> Emitter<T> for EmitterWriter
where
    T: Clone + PartialEq + Eq + Style,
{
    fn supports_color(&self) -> bool {
        self.dst.supports_color()
    }

    fn emit_diagnostic(&mut self, diag: &Diagnostic<T>) {
        let buffer = self.format_diagnostic(diag);
        if let Err(e) = emit_to_destination(&buffer.render(), &mut self.dst, self.short_message) {
            panic!("failed to emit error: {}", e)
        }
    }

    fn format_diagnostic(&mut self, diag: &Diagnostic<T>) -> StyledBuffer<T> {
        let mut sb = StyledBuffer::<T>::new();
        for component in &diag.components {
            component.format(&mut sb)
        }
        sb
    }
}

fn emit_to_destination<T>(
    rendered_buffer: &[Vec<StyledString<T>>],
    dst: &mut Destination,
    short_message: bool,
) -> io::Result<()>
where
    T: Clone + PartialEq + Eq + Style,
{
    for (pos, line) in rendered_buffer.iter().enumerate() {
        for part in line {
            dst.set_color(&part.style.as_ref().unwrap().render_style_to_color_spec())?;
            write!(dst, "{}", part.text)?;
            dst.reset()?;
        }
        if !short_message || pos != rendered_buffer.len() - 1 {
            writeln!(dst)?;
        }
    }
    dst.flush()?;
    Ok(())
}
