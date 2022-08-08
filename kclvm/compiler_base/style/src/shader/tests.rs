// mod test_shader {
//     mod test_diagnostic_shader {
//         use crate::{ShaderFactory, Style};

//         #[test]
//         fn test_logo_style() {
//             let shader = ShaderFactory::Diagnostic.get_shader();
//             assert_eq!(shader.logo_style(), Style::Logo);
//         }

//         #[test]
//         fn test_need_fix_style() {
//             let shader = ShaderFactory::Diagnostic.get_shader();
//             assert_eq!(shader.need_fix_style(), Style::NeedFix);
//         }

//         #[test]
//         fn test_need_attention_style() {
//             let shader = ShaderFactory::Diagnostic.get_shader();
//             assert_eq!(shader.need_attention_style(), Style::NeedAttention);
//         }

//         #[test]
//         fn test_helpful_style() {
//             let shader = ShaderFactory::Diagnostic.get_shader();
//             assert_eq!(shader.helpful_style(), Style::Helpful);
//         }

//         #[test]
//         fn test_important_style() {
//             let shader = ShaderFactory::Diagnostic.get_shader();
//             assert_eq!(shader.important_style(), Style::Important);
//         }

//         #[test]
//         fn test_normal_msg_style() {
//             let shader = ShaderFactory::Diagnostic.get_shader();
//             assert_eq!(shader.normal_msg_style(), Style::Normal);
//         }

//         #[test]
//         fn test_url_style() {
//             let shader = ShaderFactory::Diagnostic.get_shader();
//             assert_eq!(shader.url_style(), Style::Url);
//         }

//         #[test]
//         fn test_no_style() {
//             let shader = ShaderFactory::Diagnostic.get_shader();
//             assert_eq!(shader.no_style(), Style::NoStyle);
//         }
//     }

//     mod test_default_shader {
//         use crate::{ShaderFactory, Style};

//         #[test]
//         fn test_logo_style() {
//             let shader = ShaderFactory::Default.get_shader();
//             assert_eq!(shader.logo_style(), Style::NoStyle);
//         }

//         #[test]
//         fn test_need_fix_style() {
//             let shader = ShaderFactory::Default.get_shader();
//             assert_eq!(shader.need_fix_style(), Style::NoStyle);
//         }

//         #[test]
//         fn test_need_attention_style() {
//             let shader = ShaderFactory::Default.get_shader();
//             assert_eq!(shader.need_attention_style(), Style::NoStyle);
//         }

//         #[test]
//         fn test_helpful_style() {
//             let shader = ShaderFactory::Default.get_shader();
//             assert_eq!(shader.helpful_style(), Style::NoStyle);
//         }

//         #[test]
//         fn test_important_style() {
//             let shader = ShaderFactory::Default.get_shader();
//             assert_eq!(shader.important_style(), Style::NoStyle);
//         }

//         #[test]
//         fn test_normal_msg_style() {
//             let shader = ShaderFactory::Default.get_shader();
//             assert_eq!(shader.normal_msg_style(), Style::NoStyle);
//         }

//         #[test]
//         fn test_url_style() {
//             let shader = ShaderFactory::Default.get_shader();
//             assert_eq!(shader.url_style(), Style::NoStyle);
//         }

//         #[test]
//         fn test_no_style() {
//             let shader = ShaderFactory::Default.get_shader();
//             assert_eq!(shader.no_style(), Style::NoStyle);
//         }
//     }
// }
