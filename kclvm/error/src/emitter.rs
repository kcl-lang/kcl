use crate::{
    diagnostic::{Diagnostic, Style},
    DiagnosticId, Level,
};

use kclvm_span::{FilePathMapping, SourceMap};
use std::sync::Arc;
use std::{
    io::{self, Write},
    path::Path,
};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Emitter trait for emitting errors.
pub trait Emitter {
    fn format_diagnostic(&mut self, diag: &Diagnostic) -> Vec<String>;
    /// Emit a structured diagnostic.
    fn emit_diagnostic(&mut self, diag: &Diagnostic);
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

    fn apply_style(&mut self, lvl: Level, style: Style) -> io::Result<()> {
        let mut spec = ColorSpec::new();
        match style {
            Style::Empty | Style::LineAndColumn => {
                spec.set_bold(true);
                spec = lvl.color();
            }
            Style::Line => {
                spec.set_bold(true);
                spec.set_intense(true);
                if cfg!(windows) {
                    spec.set_fg(Some(Color::Cyan));
                } else {
                    spec.set_fg(Some(Color::Blue));
                }
            }
        }
        self.set_color(&spec)
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

impl Emitter for EmitterWriter {
    fn supports_color(&self) -> bool {
        self.dst.supports_color()
    }

    fn emit_diagnostic(&mut self, diag: &Diagnostic) {
        let buffer = self.format_diagnostic(diag);
        if let Err(e) = emit_to_destination(&buffer, &diag.level, &mut self.dst, self.short_message)
        {
            panic!("failed to emit error: {}", e)
        }
    }

    fn format_diagnostic(&mut self, diag: &Diagnostic) -> Vec<String> {
        let mut buffer: Vec<String> = vec![];
        let mut diag_str = "KCL ".to_string();
        diag_str += diag.level.to_str();
        if let Some(code) = &diag.code {
            let code_str = match code {
                DiagnosticId::Error(kind) => kind.name(),
                DiagnosticId::Warning(warn_msg) => warn_msg.to_string(),
            };
            diag_str += &format!(" [{}]", code_str);
        }
        buffer.push(diag_str);
        for (i, msg) in diag.messages.iter().enumerate() {
            buffer.push("  ".repeat(i) + &msg.pos.info());
            // To prevent read empty source content.
            if msg.pos.line > 0 {
                let mut line_source = format!("{} |", msg.pos.line);
                let line_hint_len = line_source.len();
                if let Some(sm) = &self.source_map {
                    if let Some(source_file) = sm.source_file_by_filename(&msg.pos.filename) {
                        if let Some(line) = source_file.get_line(msg.pos.line as usize - 1) {
                            line_source += &line.to_string();
                        }
                    }
                } else {
                    let sm = SourceMap::new(FilePathMapping::empty());
                    if let Ok(source_file) = sm.load_file(Path::new(&msg.pos.filename)) {
                        if let Some(line) = source_file.get_line(msg.pos.line as usize - 1) {
                            line_source += &line.to_string();
                        }
                    }
                }
                buffer.push("  ".repeat(i) + &line_source);
                if let Style::LineAndColumn = msg.style {
                    if let Some(column) = msg.pos.column {
                        let column_source = format!("{} ^", column);
                        let prefix_space = line_hint_len + column as usize - column_source.len();
                        let column_source = " ".repeat(prefix_space) + &column_source;
                        buffer.push("  ".repeat(i) + &column_source);
                    }
                }
            }
            buffer.push("  ".repeat(i) + &msg.message.clone());
            if !self.short_message {
                if let Some(note) = &msg.note {
                    buffer.push("  ".repeat(i) + &format!("Note: {}", note));
                }
            }
            buffer.push("".to_string());
        }
        buffer
    }
}

fn emit_to_destination(
    rendered_buffer: &[String],
    lvl: &Level,
    dst: &mut Destination,
    short_message: bool,
) -> io::Result<()> {
    // In order to prevent error message interleaving, where multiple error lines get intermixed
    // when multiple compiler processes error simultaneously, we emit errors with additional
    // steps.
    //
    // On Unix systems, we write into a buffered terminal rather than directly to a terminal. When
    // the .flush() is called we take the buffer created from the buffered writes and write it at
    // one shot.  Because the Unix systems use ANSI for the colors, which is a text-based styling
    // scheme, this buffered approach works and maintains the styling.
    //
    // On Windows, styling happens through calls to a terminal API. This prevents us from using the
    // same buffering approach.  Instead, we use a global Windows mutex, which we acquire long
    // enough to output the full error message, then we release.
    for (pos, line) in rendered_buffer.iter().enumerate() {
        if line.starts_with("KCL") {
            dst.apply_style(*lvl, Style::LineAndColumn)?;
        }
        write!(dst, "{}", line)?;
        dst.reset()?;
        if !short_message || pos != rendered_buffer.len() - 1 {
            writeln!(dst)?;
        }
    }
    dst.flush()?;
    Ok(())
}
