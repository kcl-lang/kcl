mod test_shader{
    mod test_diagnostic_shader {
        use crate::{ShaderFactory, Style};

        #[test]
        fn test_logo_style(){
            let shader = ShaderFactory::Diagnostic.get_shader();
            assert_eq!(shader.logo_style(), Style::Logo);
        }

        #[test]
        fn test_need_fix_style(){
            let shader = ShaderFactory::Diagnostic.get_shader();
            assert_eq!(shader.need_fix_style(), Style::NeedFix);
        }

        #[test]
        fn test_need_attention_style(){
            let shader = ShaderFactory::Diagnostic.get_shader();
            assert_eq!(shader.need_attention_style(), Style::NeedAttention);
        }

        #[test]
        fn test_helpful_style(){
            let shader = ShaderFactory::Diagnostic.get_shader();
            assert_eq!(shader.helpful_style(), Style::Helpful);
        }

        #[test]
        fn test_important_style(){
            let shader = ShaderFactory::Diagnostic.get_shader();
            assert_eq!(shader.important_style(), Style::Important);
        }

        #[test]
        fn test_normal_msg_style(){
            let shader = ShaderFactory::Diagnostic.get_shader();
            assert_eq!(shader.normal_msg_style(), Style::Normal);
        }

        #[test]
        fn test_url_style(){
            let shader = ShaderFactory::Diagnostic.get_shader();
            assert_eq!(shader.url_style(), Style::Url);
        }

        #[test]
        fn test_no_style(){
            let shader = ShaderFactory::Diagnostic.get_shader();
            assert_eq!(shader.no_style(), Style::NoStyle);
        }
    }
    
    mod test_default_shader {
        use crate::{ShaderFactory, Style};

        #[test]
        fn test_logo_style(){
            let shader = ShaderFactory::Default.get_shader();
            assert_eq!(shader.logo_style(), Style::NoStyle);
        }

        #[test]
        fn test_need_fix_style(){
            let shader = ShaderFactory::Default.get_shader();
            assert_eq!(shader.need_fix_style(), Style::NoStyle);
        }

        #[test]
        fn test_need_attention_style(){
            let shader = ShaderFactory::Default.get_shader();
            assert_eq!(shader.need_attention_style(), Style::NoStyle);
        }

        #[test]
        fn test_helpful_style(){
            let shader = ShaderFactory::Default.get_shader();
            assert_eq!(shader.helpful_style(), Style::NoStyle);
        }

        #[test]
        fn test_important_style(){
            let shader = ShaderFactory::Default.get_shader();
            assert_eq!(shader.important_style(), Style::NoStyle);
        }

        #[test]
        fn test_normal_msg_style(){
            let shader = ShaderFactory::Default.get_shader();
            assert_eq!(shader.normal_msg_style(), Style::NoStyle);
        }

        #[test]
        fn test_url_style(){
            let shader = ShaderFactory::Default.get_shader();
            assert_eq!(shader.url_style(), Style::NoStyle);
        }

        #[test]
        fn test_no_style(){
            let shader = ShaderFactory::Default.get_shader();
            assert_eq!(shader.no_style(), Style::NoStyle);
        }
    }
}

mod test_style{
    use crate::Style;

    #[test]
    fn test_render_style(){
        let color_spec = Style::NeedFix.render_style();
        Style::NeedFix.check_is_expected_colorspec(&color_spec);
        let color_spec = Style::NeedAttention.render_style();
        Style::NeedAttention.check_is_expected_colorspec(&color_spec);
        let color_spec = Style::Helpful.render_style();
        Style::Helpful.check_is_expected_colorspec(&color_spec);
        let color_spec = Style::Important.render_style();
        Style::Important.check_is_expected_colorspec(&color_spec);
        let color_spec = Style::Logo.render_style();
        Style::Logo.check_is_expected_colorspec(&color_spec);
        let color_spec = Style::NoStyle.render_style();
        Style::NoStyle.check_is_expected_colorspec(&color_spec);
        let color_spec = Style::Normal.render_style();
        Style::Normal.check_is_expected_colorspec(&color_spec);
        let color_spec = Style::Url.render_style();
        Style::Url.check_is_expected_colorspec(&color_spec);
    }
}

mod test_styled_buffer{
    use crate::styled_buffer::{StyledBuffer, StyledString};
    use crate::Style;

    fn construct_new_styledbuffer() -> StyledBuffer {
        StyledBuffer::new()
    }
    
    fn putc_hello_world(sb: &mut StyledBuffer){
        sb.putc(0, 0, 'H', Style::NoStyle);
        sb.putc(0, 1, 'e', Style::NoStyle);
        sb.putc(0, 2, 'l', Style::NoStyle);
        sb.putc(0, 3, 'l', Style::NoStyle);
        sb.putc(0, 4, 'o', Style::NoStyle);

        sb.putc(0, 5, 'W', Style::NeedFix);
        sb.putc(0, 6, 'o', Style::NeedFix);
        sb.putc(0, 7, 'r', Style::NeedFix);
        sb.putc(0, 8, 'l', Style::NeedFix);
        sb.putc(0, 9, 'd', Style::NeedFix);
    }

    fn puts_hello_world(sb: &mut StyledBuffer){
        sb.puts(0, 0, "Hello", Style::NoStyle);
        sb.puts(0, 5, "World", Style::NeedFix);
    }

    fn putl_hello_world(sb: &mut StyledBuffer){
        sb.putl("Hello", Style::NoStyle);
        sb.putl("World", Style::NeedFix);
    }

    fn appendl_hello_world(sb: &mut StyledBuffer){
        sb.appendl("Hello", Style::NoStyle);
        sb.appendl("World", Style::NeedFix);
    }

    fn require_hello_world(styled_strings: Vec<Vec<StyledString>>){
        assert_eq!(styled_strings.len(), 1);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);

        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "Hello");
        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().style, Style::NoStyle);

        assert_eq!(styled_strings.get(0).unwrap().get(1).unwrap().text, "World");
        assert_eq!(styled_strings.get(0).unwrap().get(1).unwrap().style, Style::NeedFix);
    }

    #[test]
    fn test_putc() {
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);

        let styled_strings = sb.render();

        require_hello_world(styled_strings);

        sb.putc(0, 0, 'H', Style::NoStyle);
        sb.putc(0, 1, 'E', Style::NoStyle);
        sb.putc(0, 2, 'L', Style::NoStyle);
        sb.putc(0, 3, 'L', Style::NoStyle);
        sb.putc(0, 4, 'O', Style::NoStyle);
        let styled_strings = sb.render();
        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "HELLO");
        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().style, Style::NoStyle);
    }

    #[test]
    fn test_putc_new_line(){
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);

        sb.putc(2, 0, 'A', Style::Important);
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 3);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);
        assert_eq!(styled_strings.get(1).unwrap().len(), 0);
        assert_eq!(styled_strings.get(2).unwrap().len(), 1);
        assert_eq!(styled_strings.get(2).unwrap().get(0).unwrap().text, "A");
        assert_eq!(styled_strings.get(2).unwrap().get(0).unwrap().style, Style::Important);
    }
    
    #[test]
    fn test_puts() {
        let mut sb = construct_new_styledbuffer();
        puts_hello_world(&mut sb);
        let styled_strings = sb.render();
        require_hello_world(styled_strings);
    }

    #[test]
    fn test_puts_new_line(){
        let mut sb = construct_new_styledbuffer();
        puts_hello_world(&mut sb);

        sb.puts(2, 0, "A", Style::Important);
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 3);
        assert_eq!(styled_strings.get(0).unwrap().len(), 2);
        assert_eq!(styled_strings.get(1).unwrap().len(), 0);
        assert_eq!(styled_strings.get(2).unwrap().len(), 1);
        assert_eq!(styled_strings.get(2).unwrap().get(0).unwrap().text, "A");
        assert_eq!(styled_strings.get(2).unwrap().get(0).unwrap().style, Style::Important);
    }

    #[test]
    fn test_putl() {
        let mut sb = construct_new_styledbuffer();
        putl_hello_world(&mut sb);
        let styled_strings = sb.render();
        assert_eq!(styled_strings.len(), 2);
        assert_eq!(styled_strings.get(0).unwrap().len(), 1);

        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().text, "Hello");
        assert_eq!(styled_strings.get(0).unwrap().get(0).unwrap().style, Style::NoStyle);

        assert_eq!(styled_strings.get(1).unwrap().get(0).unwrap().text, "World");
        assert_eq!(styled_strings.get(1).unwrap().get(0).unwrap().style, Style::NeedFix);
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
        sb.appendl("World", Style::NeedFix);
        sb.prepend(0, "Hello", Style::NoStyle);
        let styled_strings = sb.render();
        require_hello_world(styled_strings);
    }
    
    #[test]
    fn test_num_lines() {
        let mut sb = construct_new_styledbuffer();
        putc_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 1);
        sb.appendl("World", Style::NeedFix);
        assert_eq!(sb.num_lines(), 1);
        putl_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 3);
        puts_hello_world(&mut sb);
        assert_eq!(sb.num_lines(), 3);
    }
}


