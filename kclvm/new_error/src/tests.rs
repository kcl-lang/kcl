use std::{path::PathBuf, rc::Rc, sync::Arc};

use kclvm_error::{ErrorKind, Position};
use kclvm_span::FilePathMapping;

use crate::{
    diagnostic::Diagnostic,
    emitter::{Emitter, EmitterWriter},
    pendant::{CodeCtxPendant, HeaderPendant, LabelPendant, Pendant},
    sentence::{Message, Sentence},
    shader::{ColorShader, Level, Shader},
    styled_buffer::StyledBuffer, error::{ThisIsAnErr, DiagnosticBuilder},
};

fn get_code_position() -> Position {
    let mut pos = Position::default();
    pos.filename =
        "/Users/shijun/Workspace/kusion/KCLVM_ERROR_SYS/KCLVM/test/grammar/schema/simple/main.k"
            .to_string();
    pos.line = 2;
    pos.column = Some(6);
    pos
}

#[test]
fn test_pendant() {
    let pos = get_code_position();

    let mut sb = StyledBuffer::new();
    let shader: Rc<dyn Shader> = Rc::new(ColorShader::new());

    let hp = HeaderPendant::new(Level::Error, "E1000".to_string());
    hp.format(Rc::clone(&shader), &mut sb);

    let ccp = CodeCtxPendant::new(pos);
    ccp.format(Rc::clone(&shader), &mut sb);

    let lp = LabelPendant::new("note".to_string());
    lp.format(Rc::clone(&shader), &mut sb);

    let sss = sb.render();
    for ss in sss {
        println!("test pendant - {:?}", ss);
    }

    let sent1 =
        Sentence::new_sentence_str(Box::new(hp), Message::Str("This is an error".to_string()));
    let sent2 =
        Sentence::new_sentence_str(Box::new(ccp), Message::Str("This is an error".to_string()));
    let sent3 =
        Sentence::new_sentence_str(Box::new(lp), Message::Str("This is an error".to_string()));

    let mut emitter = EmitterWriter::default();
    let mut diag = Diagnostic::new();
    diag.add_sentence(sent1);
    diag.add_sentence(sent2);
    diag.add_sentence(sent3);

    emitter.emit_diagnostic(&diag)
}


#[test]
fn test_diagnostic_builder(){

    let err = ThisIsAnErr{
        pos: get_code_position()
    };

    let mut emitter = EmitterWriter::default();
    
    emitter.emit_diagnostic(&err.into_diagnostic());
}
