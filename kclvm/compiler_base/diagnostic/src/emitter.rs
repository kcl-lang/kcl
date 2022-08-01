use std::{
    io::{self, Write},
    rc::Rc,
};
use style::{
    styled_buffer::{StyledBuffer, StyledString},
    Shader, ShaderFactory,
};
use termcolor::{BufferWriter, ColorChoice, StandardStream, WriteColor};

use crate::{Diagnostic, DiagnosticBuilder};

/// Emitter trait for emitting errors.
pub trait Emitter {
    fn format_diagnostic(&mut self, diag: &Diagnostic) -> StyledBuffer;
    /// Emit a structured diagnostic.
    fn emit_diagnostic(&mut self, diag: &Diagnostic);
    /// Checks if we can use colors in the current output stream.
    fn supports_color(&self) -> bool {
        false
    }

    fn emit_err(&mut self, err: impl DiagnosticBuilder);
}

/// Emitter writer.
pub struct EmitterWriter {
    shader: Rc<dyn Shader>,
    dst: Destination,
    short_message: bool,
}

impl Default for EmitterWriter {
    fn default() -> Self {
        EmitterWriter::from_stderr()
    }
}

impl EmitterWriter {
    pub fn from_stderr() -> Self {
        Self {
            shader: ShaderFactory::Diagnostic.get_shader(),
            dst: Destination::from_stderr(),
            short_message: false,
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

impl Emitter for EmitterWriter {
    fn supports_color(&self) -> bool {
        self.dst.supports_color()
    }

    fn emit_diagnostic(&mut self, diag: &Diagnostic) {
        let buffer = self.format_diagnostic(diag);
        if let Err(e) = emit_to_destination(&buffer.render(), &mut self.dst, self.short_message) {
            panic!("failed to emit error: {}", e)
        }
    }

    fn emit_err(&mut self, err: impl DiagnosticBuilder) {
        let buffer = self.format_diagnostic(&err.into_diagnostic());
        if let Err(e) = emit_to_destination(&buffer.render(), &mut self.dst, self.short_message) {
            panic!("failed to emit error: {}", e)
        }
    }

    fn format_diagnostic(&mut self, diag: &Diagnostic) -> StyledBuffer {
        let mut sb = StyledBuffer::new();
        for sentence in diag.messages.iter() {
            sentence.format(Rc::clone(&self.shader), &mut sb)
        }
        sb
    }
}

fn emit_to_destination(
    rendered_buffer: &[Vec<StyledString>],
    dst: &mut Destination,
    short_message: bool,
) -> io::Result<()> {
    for (pos, line) in rendered_buffer.iter().enumerate() {
        for part in line {
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
