//! 'emitter.rs' defines the diagnostic emitter,
//! which is responsible for displaying the rendered diagnostic.
use crate::diagnostic::{Component, Diagnostic};
use compiler_base_macros::bug;
use rustc_errors::{
    styled_buffer::{StyledBuffer, StyledString},
    Style,
};
use std::io::{self, Write};
use termcolor::{Buffer, BufferWriter, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Emitter trait for emitting diagnostic.
///
/// `T: Clone + PartialEq + Eq + Style` is responsible for the theme style when diaplaying diagnostic.
/// Builtin `DiagnosticStyle` provided in 'compiler_base/error/diagnostic/style.rs'.
///
/// To customize your own `Emitter`, you could do the following steps:
///
/// 1. Define your Emitter:
///
/// ```no_run rust
///
/// // create a new `Emitter`
/// struct DummyEmitter {
///     support_color: bool
/// }
///
/// // `Dummy_Emitter` can use `DiagnosticStyle` or other style user-defined.
/// impl Emitter<DiagnosticStyle> for DummyEmitter {
///     fn supports_color(&self) -> bool {
///         // Does `Dummy_Emitter` support color ?
///         self.support_color
///     }
///
///     fn emit_diagnostic(&mut self, diag: &Diagnostic<DiagnosticStyle>) {
///         // Format `Diagnostic` into `String`.
///         let styled_string = self.format_diagnostic(diag);
///         todo!("displaying the 'styled_string'");
///     }
///
///     fn format_diagnostic(&mut self, diag: &Diagnostic<DiagnosticStyle>) -> StyledBuffer<DiagnosticStyle> {
///         // Format `Diagnostic` into `String`.
///         // This part can format `Diagnostic` into a `String`, but it does not automatically diaplay,
///         // and the `String` can be sent to an external port such as RPC.
///         let mut sb = StyledBuffer::<DiagnosticStyle>::new();
///         diag.format(&mut sb);
///         sb
///     }
/// }
///
/// ```
///
/// 2. Use your Emitter with diagnostic:
///
/// ```no_run rust
/// // create a diagnostic for emitting.
/// let mut diagnostic = Diagnostic::<DiagnosticStyle>::new();
/// // create a string component wrapped by `Box<>`.
/// let msg = Box::new(": this is an error!".to_string());
/// // add it to `Diagnostic`.
/// diagnostic.append_component(msg);
///
/// // create the emitter and emit it.
/// let mut emitter = DummyEmitter {};
/// emitter.emit_diagnostic(&diagnostic);
/// ```
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
    /// `false` by default.
    fn supports_color(&self) -> bool {
        false
    }
}

/// `EmitterWriter` is a default concrete struct of trait `Emitter` based on `termcolor1.0`.
/// `termcolor1.0` supports displaying colorful string to terminal.
///
/// # Examples
///
/// ```rust
/// # use crate::compiler_base_error::Emitter;
/// # use compiler_base_error::EmitterWriter;
/// # use compiler_base_error::diagnostic::{components::Label, Diagnostic};
/// # use compiler_base_error::diagnostic::style::DiagnosticStyle;
///
/// // 1. Create a EmitterWriter
/// let mut emitter_writer = EmitterWriter::default();
///
/// // 2. create a diagnostic for emitting.
/// let mut diagnostic = Diagnostic::<DiagnosticStyle>::new();
///
/// // 3. create components wrapped by `Box<>`.
/// let err_label = Box::new(Label::Error("E3033".to_string()));
/// let msg = Box::new(": this is an error!".to_string());
///
/// // 4. add components to `Diagnostic`.
/// diagnostic.append_component(err_label);
/// diagnostic.append_component(msg);
///
/// // 5. emit the diagnostic.
/// emitter_writer.emit_diagnostic(&diagnostic);
/// ```
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

/// Emit destinations
enum Destination {
    /// The `StandardStream` works similarly to `std::io::Stdout`,
    /// it is augmented with methods for coloring by the `WriteColor` trait.
    Terminal(StandardStream),

    /// `BufferWriter` can create buffers and write buffers to stdout or stderr.
    /// It does not implement `io::Write or WriteColor` itself.
    ///
    /// `Buffer` implements `io::Write and io::WriteColor`.
    Buffered(BufferWriter),
}

// Convert terminal to writable terminal
// On Unix systems, a buffer will be create for `BufferWriter` and return `&mut BufferWriter` and buffer.
// On Windows, it will still return the `&mut StandardStream`.
enum WritableDst<'a> {
    Terminal(&'a mut StandardStream),
    Buffered(&'a mut BufferWriter, Buffer),
}

impl Destination {
    fn from_stderr() -> Self {
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

    fn writable(&mut self) -> WritableDst<'_> {
        match *self {
            // On Windows, it will still return the `StandardStream`.
            Destination::Terminal(ref mut t) => WritableDst::Terminal(t),
            // On Unix systems, a buffer will be create for `BufferWriter`.
            Destination::Buffered(ref mut t) => {
                let buf = t.buffer();
                WritableDst::Buffered(t, buf)
            }
        }
    }

    fn supports_color(&self) -> bool {
        match *self {
            Self::Terminal(ref stream) => stream.supports_color(),
            Self::Buffered(ref buffer) => buffer.buffer().supports_color(),
        }
    }
}

impl<'a> WritableDst<'a> {
    fn set_color(&mut self, color: &ColorSpec) -> io::Result<()> {
        match *self {
            WritableDst::Terminal(ref mut t) => t.set_color(color),
            WritableDst::Buffered(_, ref mut t) => t.set_color(color),
        }
    }

    fn reset(&mut self) -> io::Result<()> {
        match *self {
            WritableDst::Terminal(ref mut t) => t.reset(),
            WritableDst::Buffered(_, ref mut t) => t.reset(),
        }
    }
}

impl<'a> Write for Destination {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match *self {
            Destination::Terminal(ref mut t) => t.write(bytes),
            Destination::Buffered(ref mut t) => t.buffer().write(bytes),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            Destination::Terminal(ref mut t) => t.flush(),
            Destination::Buffered(ref mut t) => t.buffer().flush(),
        }
    }
}

impl<'a> Write for WritableDst<'a> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match *self {
            WritableDst::Terminal(ref mut t) => t.write(bytes),
            WritableDst::Buffered(_, ref mut buf) => buf.write(bytes),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            WritableDst::Terminal(ref mut t) => t.flush(),
            WritableDst::Buffered(ref mut t, ref mut buf) => match buf.flush() {
                Ok(_) => t.print(buf),
                Err(err) => Err(err),
            },
        }
    }
}

impl<T> Emitter<T> for EmitterWriter
where
    T: Clone + PartialEq + Eq + Style,
{
    /// Checks if we can use colors in the current output stream.
    /// Depends on `termcolor1.0` which supports color.
    fn supports_color(&self) -> bool {
        self.dst.supports_color()
    }

    /// Emit a structured diagnostic.
    /// It will call `format_diagnostic` first to format the `Diagnostic` into `StyledString`.
    ///
    /// It will `panic` if something wrong during emitting.
    fn emit_diagnostic(&mut self, diag: &Diagnostic<T>) {
        let buffer = self.format_diagnostic(diag);
        if let Err(e) = emit_to_destination(&buffer.render(), &mut self.dst, self.short_message) {
            bug!("failed to emit diagnositc: {}", e)
        }
    }

    /// Format struct `Diagnostic` into `String` and render `String` into `StyledString`,
    /// and save `StyledString` in `StyledBuffer`.
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

    // Convert terminal to writable terminal.
    // On Unix systems, a buffer will be create for `BufferWriter`.
    // On Windows, it will still return the `StandardStream`.
    let mut dst = dst.writable();
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
    let _buffer_lock = lock::acquire_global_lock("compiler_base_errors");
    for (pos, line) in rendered_buffer.iter().enumerate() {
        for part in line {
            let color_spec = match &part.style {
                Some(style) => style.render_style_to_color_spec(),
                None => ColorSpec::new(),
            };
            dst.set_color(&color_spec)?;
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
