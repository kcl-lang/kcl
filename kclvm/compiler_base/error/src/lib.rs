pub mod diagnostic;
mod emitter;

pub use diagnostic::diagnostic_message::TemplateLoader;
pub use emitter::Emitter;
pub use emitter::TerminalEmitter;


mod test{
    use crate::diagnostic::diagnostic_message::TemplateLoader;

    #[test]
    fn test(){
        TemplateLoader::new_with_template_dir("template_dir");
    }
}
