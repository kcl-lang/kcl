//! Text rendering and related helper functions.
//!
//! Reuse 'styled_buffer.rs' in 'rustc_errors',
//! and 'styled_buffer.rs' has been modified to fit the feature of 'Compiler-Base'.
//!
//! - add method `appendl()` and `pushs()` to `StyledBuffer`.
//!
//! - replaced the `enum Style` with generics `T: Clone + PartialEq + Eq + Style` to support extending more styles.
//!   `StyledBuffer` still should be valid when facing the user-defined style, rather than just supporting a built-in `enum Style`.
//!
//! - add some test cases for 'StyledBuffer'.
use termcolor::ColorSpec;
pub mod lock;
pub mod styled_buffer;

/// 'Style' is a trait used to specify the user customize 'XXXStyle' can be accepted by 'StyleBuffer'.
///
/// It provides the following method `render_style_to_color_spec()`.
/// render_style_to_color_spec(&self) : render style to terminal color/font configuration.
pub trait Style {
    /// render style to terminal color/font configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    /// pub enum DummyStyle {
    ///     Dummy,
    ///     NoStyle,
    /// }
    ///
    /// impl Style for DummyStyle {
    ///     fn render_style_to_color_spec(&self) -> ColorSpec {
    ///         let mut spec = ColorSpec::new();
    ///         match self{
    ///             // For `DummyStyle::Dummy`, the font is intense and the font color is red.
    ///             DummyStyle::Dummy => {
    ///                 spec.set_fg(Some(Color::Red)).set_intense(true);
    ///             }
    ///         }
    ///         spec
    ///     }
    /// }
    /// ```
    fn render_style_to_color_spec(&self) -> ColorSpec;
}

#[cfg(test)]
mod test_styled_buffer {
    use crate::{
        styled_buffer::{StyledBuffer, StyledString},
        Style,
    };
    use termcolor::{Color, ColorSpec};

    // DummyStyle for testing 'StyledBuffer'.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum DummyStyle {
        Dummy,
        NoStyle,
    }

    impl Style for DummyStyle {
        fn render_style_to_color_spec(&self) -> ColorSpec {
            let mut spec = ColorSpec::new();
            match self {
                DummyStyle::Dummy => {
                    spec.set_fg(Some(Color::Red)).set_intense(true);
                }
                DummyStyle::NoStyle => {
                    spec.set_fg(Some(Color::Green)).set_intense(false);
                }
            }
            spec
        }
    }

    fn construct_new_styledbuffer() -> StyledBuffer<DummyStyle> {
        StyledBuffer::new()
    }

    fn putc_hello_world(sb: &mut StyledBuffer<DummyStyle>) {
        sb.putc(0, 0, 'H', Some(DummyStyle::NoStyle));
        sb.putc(0, 1, 'e', Some(DummyStyle::NoStyle));
        sb.putc(0, 2, 'l', Some(DummyStyle::NoStyle));
        sb.putc(0, 3, 'l', Some(DummyStyle::NoStyle));
        sb.putc(0, 4, 'o', Some(DummyStyle::NoStyle));
        sb.putc(0, 5, 'W', Some(DummyStyle::Dummy));
        sb.putc(0, 6, 'o', Some(DummyStyle::Dummy));
        sb.putc(0, 7, 'r', Some(DummyStyle::Dummy));
        sb.putc(0, 8, 'l', Some(DummyStyle::Dummy));
        sb.putc(0, 9, 'd', Some(DummyStyle::Dummy));
    }

    fn puts_hello_world(sb: &mut StyledBuffer<DummyStyle>) {
        sb.puts(0, 0, "Hello", Some(DummyStyle::NoStyle));
        sb.puts(0, 5, "World", Some(DummyStyle::Dummy));
    }

    fn pushs_hello_world(sb: &mut StyledBuffer<DummyStyle>) {
        sb.pushs("Hello", Some(DummyStyle::NoStyle));
        sb.pushs("World", Some(DummyStyle::Dummy));
    }

    fn appendl_hello_world(sb: &mut StyledBuffer<DummyStyle>) {
        sb.appendl("Hello", Some(DummyStyle::NoStyle));
        sb.appendl("World", Some(DummyStyle::Dummy));
    }

    fn require_hello_world(styled_strings: Vec<Vec<StyledString<DummyStyle>>>) {
        assert_eq!(styled_strings.len(), 1);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);

        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "Hello");
        assert!(
            DummyStyle::NoStyle
                == *styled_strings
                    .get(0)
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap()
        );
        assert_eq!(styled_strings.get(0).unwrap().get(1).unwrap().text, "World");
        assert!(
            DummyStyle::Dummy
                == *styled_strings
                    .get(0)
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap()
        );
    }

    #[test]
    fn test_putc() {
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);

        let styled_strings = sb.render();

        require_hello_world(styled_strings);

        sb.putc(0, 0, 'H', Some(DummyStyle::NoStyle));
        sb.putc(0, 1, 'E', Some(DummyStyle::NoStyle));
        sb.putc(0, 2, 'L', Some(DummyStyle::NoStyle));
        sb.putc(0, 3, 'L', Some(DummyStyle::NoStyle));
        sb.putc(0, 4, 'O', Some(DummyStyle::NoStyle));
        let styled_strings = sb.render();
        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "HELLO");
        assert!(
            DummyStyle::NoStyle
                == *styled_strings
                    .get(0)
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap(),
            "style error: expected style : {:?}",
            DummyStyle::NoStyle
        );
    }

    #[test]
    fn test_putc_new_line() {
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);

        sb.putc(2, 0, 'A', Some(DummyStyle::Dummy));
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 3);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);
        assert_eq!(styled_strings.get(1).unwrap().len(), 0);
        assert_eq!(styled_strings.get(2).unwrap().len(), 1);
        assert_eq!(styled_strings.get(2).unwrap().get(0).unwrap().text, "A");
        assert!(
            DummyStyle::Dummy
                == *styled_strings
                    .get(2)
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap(),
            "style error: expected style : {:?}",
            DummyStyle::Dummy
        );
    }

    #[test]
    fn test_puts() {
        let mut sb = construct_new_styledbuffer();
        puts_hello_world(&mut sb);
        let styled_strings = sb.render();
        require_hello_world(styled_strings);
    }

    #[test]
    fn test_puts_new_line() {
        let mut sb = construct_new_styledbuffer();
        puts_hello_world(&mut sb);

        sb.puts(2, 0, "A", Some(DummyStyle::Dummy));
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 3);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);
        assert_eq!(styled_strings.get(1).unwrap().len(), 0);
        assert_eq!(styled_strings.get(2).unwrap().len(), 1);
        assert_eq!(styled_strings.get(2).unwrap().get(0).unwrap().text, "A");
        assert!(
            DummyStyle::Dummy
                == *styled_strings
                    .get(2)
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap(),
            "style error: expected style : {:?}",
            DummyStyle::Dummy
        );
    }

    #[test]
    fn test_pushs() {
        let mut sb = construct_new_styledbuffer();
        pushs_hello_world(&mut sb);
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 2);
        assert_eq!(styled_strings.get(0).unwrap().len(), 1);

        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "Hello");
        assert!(
            DummyStyle::NoStyle
                == *styled_strings
                    .get(0)
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap(),
            "style error: expected style : {:?}",
            DummyStyle::NoStyle
        );

        assert_eq!(styled_strings.get(1).unwrap().get(0).unwrap().text, "World");
        assert!(
            DummyStyle::Dummy
                == *styled_strings
                    .get(1)
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap(),
            "style error: expected style : {:?}",
            DummyStyle::Dummy
        );
    }

    #[test]
    fn test_appendl() {
        let mut sb = construct_new_styledbuffer();
        appendl_hello_world(&mut sb);
        let styled_strings = sb.render();
        require_hello_world(styled_strings);
    }

    #[test]
    fn test_prepend() {
        let mut sb = construct_new_styledbuffer();
        sb.appendl("World", Some(DummyStyle::Dummy));
        sb.prepend(0, "Hello", Some(DummyStyle::NoStyle));
        let styled_strings = sb.render();
        require_hello_world(styled_strings);
    }

    #[test]
    fn test_num_lines() {
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 1);
        sb.appendl("World", Some(DummyStyle::Dummy));
        assert_eq!(sb.num_lines(), 1);
        pushs_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 3);
        puts_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 3);
    }
}
