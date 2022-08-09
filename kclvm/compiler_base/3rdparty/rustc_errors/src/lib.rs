//! Text rendering and related helper functions.
//!
//! Reuse 'styled_buffer.rs' in 'rustc_errors',
//! and 'styled_buffer.rs' has been modified to fit the feature of 'Compiler-Base'.
//!
//! - add method 'appendl()' and 'putl()' to 'StyledBuffer'.
//! - replaced the 'enum Style' with 'trait Style' to support extending more styles.
//! - add some test cases for 'StyledBuffer'.

use std::any::Any;
use termcolor::ColorSpec;

pub mod styled_buffer;

/// 'Style' is a trait used to specify the 'enum Style' supported by 'StyleBuffer'.
///
/// It provides the following 4 methods.
///
/// as_any(&self) : return 'self' to support for downcasting concrete style types.
/// box_clone(&self) : copy 'self' and return via 'Box' wrapper.
/// style_eq(&self, other: &Box<dyn Style>) : downcast 'other' to concrete style type and compare equivalence with 'self'.
/// fn render_style(&self) : render style to terminal color/font configuration.
///
/// # Examples
///
/// ```rust
/// use rustc_errors::Style;
/// use termcolor::{ColorSpec, Color};
/// use core::any::Any;
///
/// #[derive(Copy, Clone, Debug, PartialEq, Hash)]
/// pub enum DummyStyle{
///     Dummy
/// }
///
/// impl Style for DummyStyle{
///    // return 'self' to support for downcasting concrete style types.
///    fn as_any(&self) -> &dyn Any{
///        self
///    }
///
///    // copy 'self' and return via 'Box' wrapper.
///    fn box_clone(&self) -> Box<dyn Style> {
///         Box::new((*self).clone())
///    }
///
///    fn render_style(&self) -> ColorSpec {
///         let mut spec = ColorSpec::new();
///         match self{
///             DummyStyle::Dummy => {
///                 spec.set_fg(Some(Color::Red)).set_intense(true);
///             }
///         }
///         spec
///    }
///
///    fn style_eq(&self, other: &Box<dyn Style>) -> bool {
///         let other_style: &DummyStyle = match other.as_any().downcast_ref::<DummyStyle>() {
///             Some(sty) => sty,
///             None => panic!("Err")
///         };
///         *self == *other_style
///    }
/// }
/// ```
pub trait Style {
    fn as_any(&self) -> &dyn Any;
    fn box_clone(&self) -> Box<dyn Style>;
    fn style_eq(&self, other: &Box<dyn Style>) -> bool;
    fn render_style(&self) -> ColorSpec;
}

impl PartialEq for Box<dyn Style> {
    fn eq(&self, other: &Self) -> bool {
        self.style_eq(other)
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Clone for Box<dyn Style> {
    fn clone(&self) -> Box<dyn Style> {
        self.box_clone()
    }
}

#[cfg(test)]
mod test_styled_buffer {
    use crate::{
        styled_buffer::{StyledBuffer, StyledString},
        Style,
    };
    use std::any::Any;
    use termcolor::{Color, ColorSpec};

    // DummyStyle for testing 'StyledBuffer'.
    #[derive(Copy, Clone, Debug, PartialEq, Hash)]
    pub enum DummyStyle {
        Dummy,
        NoStyle,
    }

    impl Style for DummyStyle {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn box_clone(&self) -> Box<dyn Style> {
            Box::new((*self).clone())
        }

        fn render_style(&self) -> ColorSpec {
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

        fn style_eq(&self, other: &Box<dyn Style>) -> bool {
            let other_style: &DummyStyle = match other.as_any().downcast_ref::<DummyStyle>() {
                Some(sty) => sty,
                None => panic!("Err"),
            };
            *self == *other_style
        }
    }

    fn construct_new_styledbuffer() -> StyledBuffer {
        StyledBuffer::new()
    }

    fn putc_hello_world(sb: &mut StyledBuffer) {
        sb.putc(0, 0, 'H', Some(Box::new(DummyStyle::NoStyle)));
        sb.putc(0, 1, 'e', Some(Box::new(DummyStyle::NoStyle)));
        sb.putc(0, 2, 'l', Some(Box::new(DummyStyle::NoStyle)));
        sb.putc(0, 3, 'l', Some(Box::new(DummyStyle::NoStyle)));
        sb.putc(0, 4, 'o', Some(Box::new(DummyStyle::NoStyle)));
        sb.putc(0, 5, 'W', Some(Box::new(DummyStyle::Dummy)));
        sb.putc(0, 6, 'o', Some(Box::new(DummyStyle::Dummy)));
        sb.putc(0, 7, 'r', Some(Box::new(DummyStyle::Dummy)));
        sb.putc(0, 8, 'l', Some(Box::new(DummyStyle::Dummy)));
        sb.putc(0, 9, 'd', Some(Box::new(DummyStyle::Dummy)));
    }

    fn puts_hello_world(sb: &mut StyledBuffer) {
        sb.puts(0, 0, "Hello", Some(Box::new(DummyStyle::NoStyle)));
        sb.puts(0, 5, "World", Some(Box::new(DummyStyle::Dummy)));
    }

    fn putl_hello_world(sb: &mut StyledBuffer) {
        sb.putl("Hello", Some(Box::new(DummyStyle::NoStyle)));
        sb.putl("World", Some(Box::new(DummyStyle::Dummy)));
    }

    fn appendl_hello_world(sb: &mut StyledBuffer) {
        sb.appendl("Hello", Some(Box::new(DummyStyle::NoStyle)));
        sb.appendl("World", Some(Box::new(DummyStyle::Dummy)));
    }

    fn require_hello_world(styled_strings: Vec<Vec<StyledString>>) {
        assert_eq!(styled_strings.len(), 1);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);

        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "Hello");
        assert!(DummyStyle::NoStyle.style_eq(
            styled_strings
                .get(0)
                .unwrap()
                .get(0)
                .unwrap()
                .style
                .as_ref()
                .unwrap()
        ));
        assert_eq!(styled_strings.get(0).unwrap().get(1).unwrap().text, "World");
        assert!(DummyStyle::Dummy.style_eq(
            styled_strings
                .get(0)
                .unwrap()
                .get(1)
                .unwrap()
                .style
                .as_ref()
                .unwrap()
        ));
    }

    #[test]
    fn test_putc() {
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);

        let styled_strings = sb.render();

        require_hello_world(styled_strings);

        sb.putc(0, 0, 'H', Some(Box::new(DummyStyle::NoStyle)));
        sb.putc(0, 1, 'E', Some(Box::new(DummyStyle::NoStyle)));
        sb.putc(0, 2, 'L', Some(Box::new(DummyStyle::NoStyle)));
        sb.putc(0, 3, 'L', Some(Box::new(DummyStyle::NoStyle)));
        sb.putc(0, 4, 'O', Some(Box::new(DummyStyle::NoStyle)));
        let styled_strings = sb.render();
        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "HELLO");
        assert!(
            DummyStyle::NoStyle.style_eq(
                styled_strings
                    .get(0)
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap()
            ),
            "style error: expected style : {:?}",
            DummyStyle::NoStyle
        );
    }

    #[test]
    fn test_putc_new_line() {
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);

        sb.putc(2, 0, 'A', Some(Box::new(DummyStyle::Dummy)));
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 3);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);
        assert_eq!(styled_strings.get(1).unwrap().len(), 0);
        assert_eq!(styled_strings.get(2).unwrap().len(), 1);
        assert_eq!(styled_strings.get(2).unwrap().get(0).unwrap().text, "A");
        assert!(
            DummyStyle::Dummy.style_eq(
                styled_strings
                    .get(2)
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap()
            ),
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

        sb.puts(2, 0, "A", Some(Box::new(DummyStyle::Dummy)));
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 3);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);
        assert_eq!(styled_strings.get(1).unwrap().len(), 0);
        assert_eq!(styled_strings.get(2).unwrap().len(), 1);
        assert_eq!(styled_strings.get(2).unwrap().get(0).unwrap().text, "A");
        assert!(
            DummyStyle::Dummy.style_eq(
                styled_strings
                    .get(2)
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap()
            ),
            "style error: expected style : {:?}",
            DummyStyle::Dummy
        );
    }

    #[test]
    fn test_putl() {
        let mut sb = construct_new_styledbuffer();
        putl_hello_world(&mut sb);
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 2);
        assert_eq!(styled_strings.get(0).unwrap().len(), 1);

        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "Hello");
        assert!(
            DummyStyle::NoStyle.style_eq(
                styled_strings
                    .get(0)
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap()
            ),
            "style error: expected style : {:?}",
            DummyStyle::NoStyle
        );

        assert_eq!(styled_strings.get(1).unwrap().get(0).unwrap().text, "World");
        assert!(
            DummyStyle::Dummy.style_eq(
                styled_strings
                    .get(1)
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .style
                    .as_ref()
                    .unwrap()
            ),
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
        sb.appendl("World", Some(Box::new(DummyStyle::Dummy)));
        sb.prepend(0, "Hello", Some(Box::new(DummyStyle::NoStyle)));
        let styled_strings = sb.render();
        require_hello_world(styled_strings);
    }

    #[test]
    fn test_num_lines() {
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 1);
        sb.appendl("World", Some(Box::new(DummyStyle::Dummy)));
        assert_eq!(sb.num_lines(), 1);
        putl_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 3);
        puts_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 3);
    }
}
