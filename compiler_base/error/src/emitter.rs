//! 'emitter.rs' defines the diagnostic emitter,
//! which is responsible for displaying the rendered diagnostic.
//!
//! The crate provides `Emitter` trait to define the interface that diagnostic emitter should implement.
//! and also provides a built-in emitters:
//!
//!  + `EmitterWriter` is responsible for emitting diagnostic to the writer who implements trait [`Write`] and [`Send`].
//!  + TODO(zongz): `EmitterAPI` is responsible for serializing diagnostics and emitting them to the API.
//!
//！Besides, it's easy to define your customized `Emitter` by implementing `Emitter` trait.
//! For more information about how to define your customized `Emitter`, see the doc above `Emitter` trait.

use crate::{
    diagnostic::{Component, Diagnostic},
    errors::ComponentError,
    DiagnosticStyle,
};
use anyhow::Result;
use rustc_errors::{
    styled_buffer::{StyledBuffer, StyledString},
    Style,
};
use std::fmt::Debug;
use std::io::{self, Write};
use termcolor::{Ansi, Buffer, BufferWriter, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// trait `Emitter` for emitting diagnostic.
///
/// `T: Clone + PartialEq + Eq + Style` is responsible for the theme style when diaplaying diagnostic.
/// Builtin `DiagnosticStyle` provided in 'compiler_base/error/diagnostic/style.rs'.
///
/// To customize your own `Emitter`, you could do the following steps:
///
/// # Examples
///
/// 1. Define your Emitter:
///
/// ```ignore rust
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
/// ```ignore rust
///
/// // Create a diagnostic for emitting.
/// let mut diagnostic = Diagnostic::<DiagnosticStyle>::new();
///
/// // Create a string component wrapped by `Box<>`.
/// let msg = Box::new(": this is an error!".to_string());
///
/// // Add it to `Diagnostic`.
/// diagnostic.append_component(msg);
///
/// // Create the emitter and emit it.
/// let mut emitter = DummyEmitter {};
/// emitter.emit_diagnostic(&diagnostic);
/// ```
pub trait Emitter<T>
where
    T: Clone + PartialEq + Eq + Style + Debug,
{
    /// Format struct `Diagnostic` into `String` and render `String` into `StyledString`,
    /// and save `StyledString` in `StyledBuffer`.
    fn format_diagnostic(
        &mut self,
        diag: &Diagnostic<T>,
    ) -> Result<StyledBuffer<T>, ComponentError>;

    /// Emit a structured diagnostic.
    fn emit_diagnostic(&mut self, diag: &Diagnostic<T>) -> Result<()>;

    /// Checks if we can use colors in the current output stream.
    /// `false` by default.
    fn supports_color(&self) -> bool {
        false
    }
}

/// `EmitterWriter` implements trait `Emitter` based on `termcolor1.0`
/// for rendering diagnostic as strings and displaying them to the terminal.
///
/// `termcolor1.0` supports displaying colorful string to terminal.
///
/// # Examples
///
/// ```rust
/// # use crate::compiler_base_error::Emitter;
/// # use compiler_base_error::EmitterWriter;
/// # use compiler_base_error::{components::Label, Diagnostic};
/// # use compiler_base_error::DiagnosticStyle;
///
/// // 1. Create a EmitterWriter
/// let mut term_emitter = EmitterWriter::default();
///
/// // 2. Create a diagnostic for emitting.
/// let mut diagnostic = Diagnostic::<DiagnosticStyle>::new();
///
/// // 3. Create components wrapped by `Box<>`.
/// let err_label = Box::new(Label::Error("E3033".to_string()));
/// let msg = Box::new(": this is an error!".to_string());
///
/// // 4. Add components to `Diagnostic`.
/// diagnostic.append_component(err_label);
/// diagnostic.append_component(msg);
///
/// // 5. Emit the diagnostic.
/// term_emitter.emit_diagnostic(&diagnostic);
/// ```
pub struct EmitterWriter<'a> {
    dst: Destination<'a>,
}

impl<'a> EmitterWriter<'a> {
    /// Return a [`Destination`] with custom writer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compiler_base_error::Destination;
    /// use compiler_base_error::EmitterWriter;
    /// use termcolor::ColorChoice;
    /// // 1. Create a `Destination` and close the color.
    /// let dest = Destination::from_stderr(ColorChoice::Never);
    ///
    /// // 2. Create the EmiterWriter by `Destination` with writer stderr.
    /// let emitter_writer = EmitterWriter::new_with_writer(dest);
    /// ```
    pub fn new_with_writer(dst: Destination<'a>) -> Self {
        Self { dst }
    }
}

impl<'a> Default for EmitterWriter<'a> {
    /// Return a [`Destination`] with writer stderr.
    fn default() -> Self {
        Self {
            dst: Destination::from_stderr(ColorChoice::Auto),
        }
    }
}

/// Emit destinations provide four ways to emit.
///
/// - [`Destination::Terminal`]: Emit by [`StandardStream`]
/// - [`Destination::Buffered`]: Emit by [`BufferWriter`], you can save content in [`Buffer`] first, and then emit the [`Buffer`] to [`BufferWriter`] on flush.
/// - [`Destination::UnColoredRaw`]: Emit by a custom writer that does not support colors.
/// - [`Destination::ColoredRaw`]: Emit by a custom writer that supports colors.
///
/// Note: All custom writers must implement two traits [`Write`] and [`Send`].
///
/// # Examples
/// 1. If you want to use writer stdout or stderr, you can use the method `from_stderr` and `from_stdout`.
///
/// ```rust
/// use compiler_base_error::Destination;
/// use termcolor::ColorChoice;
/// // stdout
/// let dest_stdout = Destination::from_stdout(ColorChoice::Never);
/// // stderr
/// let dest_stderr = Destination::from_stderr(ColorChoice::Never);
/// ```
///
/// 2. If you want to use custom writer
/// ```rust
/// use compiler_base_error::Destination;
/// use termcolor::Ansi;
/// use std::io::Write;
/// use std::io;
///
/// // 1. Define a custom writer.
/// struct MyWriter {
///     content: String,
/// }
///
/// // 2. Implement trait `Write`.
/// impl Write for MyWriter {
///     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
///         if let Ok(s) = std::str::from_utf8(buf) {
///             self.content.push_str(s)
///         } else {
///             self.content = "Nothing".to_string();
///         }
///         Ok(buf.len())
///     }
///
///     fn flush(&mut self) -> io::Result<()> {
///         Ok(())
///     }
/// }
/// // 3. Implement trait `Send`.
/// unsafe impl Send for MyWriter {}
///
/// // 4. Define a destiation.
/// let mut my_writer = MyWriter{ content: String::new() };
/// Destination::UnColoredRaw(&mut my_writer);
///
/// // 5. If your custom writer supports color.
/// Destination::ColoredRaw(Ansi::new(&mut my_writer));
/// ```
pub enum Destination<'a> {
    /// Emit to stderr/stdout by stream.
    Terminal(StandardStream),

    /// Save by the 'Buffer', and then Emit to stderr/stdout by the 'Buffer' through the 'BufferWriter'.
    Buffered(BufferWriter, Buffer),

    /// Emit to a destiation without color.
    UnColoredRaw(&'a mut (dyn Write + Send)),

    /// Emit to a customize destiation with color.
    ColoredRaw(Ansi<&'a mut (dyn Write + Send)>),
}

impl<'a> Destination<'a> {
    /// New a stderr destination.
    /// [`ColorChoice`] is used to determine whether the output content has been colored.
    pub fn from_stderr(choice: ColorChoice) -> Self {
        // On Windows we'll be performing global synchronization on the entire
        // system for emitting errors, so there's no need to buffer
        // anything.
        //
        // On non-Windows we rely on the atomicity of `write` to ensure errors
        // don't get all jumbled up.
        if cfg!(windows) {
            Self::Terminal(StandardStream::stderr(choice))
        } else {
            let buffer_writer = BufferWriter::stderr(choice);
            let buffer = buffer_writer.buffer();
            Destination::Buffered(buffer_writer, buffer)
        }
    }

    /// New a stdout destination.
    /// [`ColorChoice`] is used to determine whether the output content has been colored.
    pub fn from_stdout(choice: ColorChoice) -> Self {
        // On Windows we'll be performing global synchronization on the entire
        // system for emitting errors, so there's no need to buffer
        // anything.
        //
        // On non-Windows we rely on the atomicity of `write` to ensure errors
        // don't get all jumbled up.
        if cfg!(windows) {
            Self::Terminal(StandardStream::stdout(choice))
        } else {
            let buffer_writer = BufferWriter::stdout(ColorChoice::Auto);
            let buffer = buffer_writer.buffer();
            Destination::Buffered(buffer_writer, buffer)
        }
    }

    /// Returns true if and only if the underlying [`Destination`] supports colors.
    pub fn supports_color(&self) -> bool {
        match *self {
            Self::Terminal(ref stream) => stream.supports_color(),
            Self::Buffered(ref buffer, _) => buffer.buffer().supports_color(),
            Self::UnColoredRaw(_) => false,
            Self::ColoredRaw(_) => true,
        }
    }

    /// Set color for the [`Destination`] by [`ColorSpec`].
    /// Subsequent writes to this writer will use these settings until either `reset()` is called or new color settings are set.
    /// If there was a problem resetting the color settings, then an error is returned.
    pub fn set_color(&mut self, color: &ColorSpec) -> io::Result<()> {
        match *self {
            Self::Terminal(ref mut t) => t.set_color(color),
            Self::Buffered(_, ref mut t) => t.set_color(color),
            Self::ColoredRaw(ref mut t) => t.set_color(color),
            Self::UnColoredRaw(_) => Ok(()),
        }
    }

    /// Reset the current color settings for [`Destination`] to their original settings.
    /// If there was a problem resetting the color settings, then an error is returned.
    pub fn reset(&mut self) -> io::Result<()> {
        match *self {
            Self::Terminal(ref mut t) => t.reset(),
            Self::Buffered(_, ref mut t) => t.reset(),
            Self::ColoredRaw(ref mut t) => t.reset(),
            Self::UnColoredRaw(_) => Ok(()),
        }
    }
}

impl<'a> Write for Destination<'a> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match *self {
            Self::Terminal(ref mut t) => t.write(bytes),
            Self::Buffered(_, ref mut buf) => buf.write(bytes),
            Self::UnColoredRaw(ref mut w) => w.write(bytes),
            Self::ColoredRaw(ref mut t) => t.write(bytes),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            Self::Terminal(ref mut t) => t.flush(),
            Self::Buffered(_, ref mut buf) => buf.flush(),
            Self::UnColoredRaw(ref mut w) => w.flush(),
            Self::ColoredRaw(ref mut w) => w.flush(),
        }
    }
}

impl<'a> Drop for Destination<'a> {
    fn drop(&mut self) {
        if let Destination::Buffered(ref mut dst, ref mut buf) = self {
            drop(dst.print(buf));
        }
    }
}

impl<'a, T> Emitter<T> for EmitterWriter<'a>
where
    T: Clone + PartialEq + Eq + Style + Debug,
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
    fn emit_diagnostic(&mut self, diag: &Diagnostic<T>) -> Result<()> {
        let buffer = self.format_diagnostic(diag)?;
        emit_to_destination(&buffer.render(), &mut self.dst)?;
        Ok(())
    }

    /// Format struct `Diagnostic` into `String` and render `String` into `StyledString`,
    /// and save `StyledString` in `StyledBuffer`.
    fn format_diagnostic(
        &mut self,
        diag: &Diagnostic<T>,
    ) -> Result<StyledBuffer<T>, ComponentError> {
        let mut sb = StyledBuffer::<T>::new();
        let mut errs = vec![];
        diag.format(&mut sb, &mut errs);
        if !errs.is_empty() {
            return Err(ComponentError::ComponentFormatErrors(errs));
        }
        Ok(sb)
    }
}

fn emit_to_destination<T>(
    rendered_buffer: &[Vec<StyledString<T>>],
    dst: &mut Destination,
) -> io::Result<()>
where
    T: Clone + PartialEq + Eq + Style + Debug,
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
    //
    // This part of the code refers to the implementation of [`rustc_error`].
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
        if pos != rendered_buffer.len() - 1 {
            writeln!(dst)?;
        }
    }
    dst.flush()?;
    Ok(())
}

/// Emit the [`Diagnostic`] with [`DiagnosticStyle`] to uncolored text strng.
///
/// Examples
///
/// ```rust
/// use compiler_base_error::{Diagnostic, components::Label};
/// use compiler_base_error::emit_diagnostic_to_uncolored_text;
/// use compiler_base_error::DiagnosticStyle;
/// // 1. Define your diagnostic.
/// let mut diag = Diagnostic::<DiagnosticStyle>::new();
///
/// // 2. Add a component for the diagnostic, otherwise it will emit an empty string.
/// diag.append_component(Box::new(Label::Note));
///
/// // 3. Emit it.
/// let text = emit_diagnostic_to_uncolored_text(&diag).unwrap();
/// assert_eq!(text, "note");
/// ```
pub fn emit_diagnostic_to_uncolored_text(diag: &Diagnostic<DiagnosticStyle>) -> Result<String> {
    let mut emit_tes = EmitResultText::new();
    {
        let mut emit_writter =
            EmitterWriter::new_with_writer(Destination::UnColoredRaw(&mut emit_tes));
        emit_writter.emit_diagnostic(diag)?;
    }
    Ok(emit_tes.test_res)
}

/// Used to save the result of emit into a [`String`],
/// because trait [`Write`] and [`Send`] cannot be directly implemented by [`String`].
pub(crate) struct EmitResultText {
    test_res: String,
}

impl Write for EmitResultText {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            self.test_res.push_str(s)
        } else {
            self.test_res = String::new();
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

unsafe impl Send for EmitResultText {}

impl EmitResultText {
    /// New a [`EmitResultText`] with an empty [`String`]。
    pub(crate) fn new() -> Self {
        Self {
            test_res: String::new(),
        }
    }
}
