mod test_styled_buffer {
    use compiler_base_style::{option_box_style, diagnostic_style::DiagnosticStyle, Style};

    use crate::styled_buffer::{StyledBuffer, StyledString};


    fn construct_new_styledbuffer() -> StyledBuffer {
        StyledBuffer::new()
    }

    fn putc_hello_world(sb: &mut StyledBuffer) {
        sb.putc(0, 0, 'H',option_box_style!(DiagnosticStyle::NoStyle));
        sb.putc(0, 1, 'e',option_box_style!(DiagnosticStyle::NoStyle));
        sb.putc(0, 2, 'l',option_box_style!(DiagnosticStyle::NoStyle));
        sb.putc(0, 3, 'l',option_box_style!(DiagnosticStyle::NoStyle));
        sb.putc(0, 4, 'o',option_box_style!(DiagnosticStyle::NoStyle));
        sb.putc(0, 5, 'W',option_box_style!(DiagnosticStyle::NeedFix));
        sb.putc(0, 6, 'o',option_box_style!(DiagnosticStyle::NeedFix));
        sb.putc(0, 7, 'r',option_box_style!(DiagnosticStyle::NeedFix));
        sb.putc(0, 8, 'l',option_box_style!(DiagnosticStyle::NeedFix));
        sb.putc(0, 9, 'd',option_box_style!(DiagnosticStyle::NeedFix));
    }

    fn puts_hello_world(sb: &mut StyledBuffer) {
        sb.puts(0, 0, "Hello",option_box_style!(DiagnosticStyle::NoStyle));
        sb.puts(0, 5, "World",option_box_style!(DiagnosticStyle::NeedFix));
    }

    fn putl_hello_world(sb: &mut StyledBuffer) {
        sb.putl("Hello",option_box_style!(DiagnosticStyle::NoStyle));
        sb.putl("World",option_box_style!(DiagnosticStyle::NeedFix));
    }

    fn appendl_hello_world(sb: &mut StyledBuffer) {
        sb.appendl("Hello",option_box_style!(DiagnosticStyle::NoStyle));
        sb.appendl("World",option_box_style!(DiagnosticStyle::NeedFix));
    }

    fn require_hello_world(styled_strings: Vec<Vec<StyledString>>) {
        assert_eq!(styled_strings.len(), 1);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);

        
        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "Hello");
        assert!(DiagnosticStyle::NoStyle.style_eq(styled_strings.get(0).unwrap().get(0).unwrap().style.as_ref().unwrap()));
        assert_eq!(styled_strings.get(0).unwrap().get(1).unwrap().text, "World");
        assert!(DiagnosticStyle::NeedFix.style_eq(styled_strings.get(0).unwrap().get(1).unwrap().style.as_ref().unwrap()));
    }

    #[test]
    fn test_putc() {
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);

        let styled_strings = sb.render();

        require_hello_world(styled_strings);

        sb.putc(0, 0, 'H',option_box_style!(DiagnosticStyle::NoStyle));
        sb.putc(0, 1, 'E',option_box_style!(DiagnosticStyle::NoStyle));
        sb.putc(0, 2, 'L',option_box_style!(DiagnosticStyle::NoStyle));
        sb.putc(0, 3, 'L',option_box_style!(DiagnosticStyle::NoStyle));
        sb.putc(0, 4, 'O',option_box_style!(DiagnosticStyle::NoStyle));
        let styled_strings = sb.render();
        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "HELLO");
        assert!(DiagnosticStyle::NoStyle.style_eq(styled_strings.get(0).unwrap().get(0).unwrap().style.as_ref().unwrap()),
        "style error: expected style : {:?}", DiagnosticStyle::NoStyle);
    }

    #[test]
    fn test_putc_new_line() {
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);

        sb.putc(2, 0, 'A',option_box_style!(DiagnosticStyle::Important));
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 3);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);
        assert_eq!(styled_strings.get(1).unwrap().len(), 0);
        assert_eq!(styled_strings.get(2).unwrap().len(), 1);
        assert_eq!(styled_strings.get(2).unwrap().get(0).unwrap().text, "A");
        assert!(DiagnosticStyle::Important.style_eq(styled_strings.get(2).unwrap().get(0).unwrap().style.as_ref().unwrap()),
        "style error: expected style : {:?}", DiagnosticStyle::Important);
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

        sb.puts(2, 0, "A",option_box_style!(DiagnosticStyle::Important));
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 3);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);
        assert_eq!(styled_strings.get(1).unwrap().len(), 0);
        assert_eq!(styled_strings.get(2).unwrap().len(), 1);
        assert_eq!(styled_strings.get(2).unwrap().get(0).unwrap().text, "A");
        assert!(DiagnosticStyle::Important.style_eq(styled_strings.get(2).unwrap().get(0).unwrap().style.as_ref().unwrap()),
        "style error: expected style : {:?}", DiagnosticStyle::Important);
    }

    #[test]
    fn test_putl() {
        let mut sb = construct_new_styledbuffer();
        putl_hello_world(&mut sb);
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 2);
        assert_eq!(styled_strings.get(0).unwrap().len(), 1);

        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "Hello");
        assert!(DiagnosticStyle::NoStyle.style_eq(styled_strings.get(0).unwrap().get(0).unwrap().style.as_ref().unwrap()),
        "style error: expected style : {:?}", DiagnosticStyle::NoStyle);

        assert_eq!(styled_strings.get(1).unwrap().get(0).unwrap().text, "World");
        assert!(DiagnosticStyle::NeedFix.style_eq(styled_strings.get(1).unwrap().get(0).unwrap().style.as_ref().unwrap()),
        "style error: expected style : {:?}", DiagnosticStyle::NeedFix);
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
        sb.appendl("World",option_box_style!(DiagnosticStyle::NeedFix));
        sb.prepend(0, "Hello",option_box_style!(DiagnosticStyle::NoStyle));
        let styled_strings = sb.render();
        require_hello_world(styled_strings);
    }

    #[test]
    fn test_num_lines() {
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 1);
        sb.appendl("World",option_box_style!(DiagnosticStyle::NeedFix));
        assert_eq!(sb.num_lines(), 1);
        putl_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 3);
        puts_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 3);
    }
}


