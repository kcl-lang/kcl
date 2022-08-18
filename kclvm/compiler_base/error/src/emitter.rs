//! 'emitter.rs' defines the diagnostic emitter,
//! which is responsible for displaying the rendered diagnostic.
use crate::diagnostic::{Component, Diagnostic};
use compiler_base_macros::bug;
use rustc_errors::{
    styled_buffer::{StyledBuffer, StyledString},
    Style,
};
use std::io::{self, Write};
use termcolor::{BufferWriter, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Emitter trait for emitting diagnostic.
///
/// `T: Clone + PartialEq + Eq + Style` is responsible for the theme style when diaplaying diagnostic.
/// Builtin `DiagnosticStyle` provided in 'compiler_base/error/diagnostic/style.rs'.
pub trait Emitter<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    /// Format struct `Diagnostic` into `String` and render `String` into `StyledString`,
    /// and save `StyledString` in `StyledBuffer`.
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
}

impl Default for EmitterWriter {
    fn default() -> Self {
        Self {
            dst: Destination::from_stderr(),
            short_message: false,
        }
    }
}

impl EmitterWriter {
    pub fn from_stderr() -> Self {
        Self {
            dst: Destination::from_stderr(),
            short_message: false,
        }
    }
}

/// Emit destinations
pub enum Destination {
    /// The `StandardStream` works similarly to `std::io::Stdout`,
    /// it is augmented with methods for coloring by the `WriteColor` trait.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io::Write;
    /// # use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
    /// // Diaplay a red color string "test text".
    /// let mut standard_stream = StandardStream::stdout(ColorChoice::Always);
    /// standard_stream.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    /// writeln!(&mut standard_stream, "test text")?;
    /// ```
    Terminal(StandardStream),

    /// `BufferWriter` can create buffers and write buffers to stdout or stderr.
    /// It does not implement `io::Write or WriteColor` itself.
    ///
    /// `Buffer` implements `io::Write and io::WriteColor`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io::Write;
    /// # use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};
    /// // `buffer` can be created.
    /// let mut writter = BufferWriter::stderr(ColorChoice::Always);
    /// let mut buffer = writter.buffer();
    ///
    /// // write in `buffer`
    /// buffer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    /// writeln!(&mut buffer, "test text 1")?;
    /// writeln!(&mut buffer, "test text 2")?;
    /// writeln!(&mut buffer, "test text 3")?;
    ///
    /// // print `buffer`
    /// writter.print(&buffer)?;
    /// ```
    Buffered(BufferWriter),

    // 'Raw' is an interface used to expand the emitter destination
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
        if cfg!(windows) {
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
            bug!("failed to emit diagnositc: {}", e)
        }
    }

    fn format_diagnostic(&mut self, diag: &Diagnostic<T>) -> StyledBuffer<T> {
        let mut sb = StyledBuffer::<T>::new();
        diag.format(&mut sb);
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
    use rustc_errors::lock;
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
    let _buffer_lock = lock::acquire_global_lock("rustc_errors");
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
