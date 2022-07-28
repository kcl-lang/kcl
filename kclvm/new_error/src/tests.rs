use std::rc::Rc;
use new_err_macro::DiagnosticBuilder;
use kclvm_error::Position;

use crate::{
    diagnostic::Diagnostic,
    emitter::{Emitter, EmitterWriter},
    error::{DiagnosticBuilder, ThisIsAnErr},
    pendant::{CodeCtxPendant, HeaderPendant, LabelPendant, Pendant},
    sentence::{Message, Sentence},
    shader::{ColorShader, Level, Shader},
    styled_buffer::StyledBuffer,
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
fn test_diagnostic_builder() {
    let err = ThisIsAnErr {
        pos: get_code_position(),
    };

    let mut emitter = EmitterWriter::default();
    emitter.emit_diagnostic(&err.into_diagnostic());
}

// 这里可以出了title 都放后面，title只能有一个。
// 这里的顺序怎么写，外面后面就怎么输出。
#[derive(DiagnosticBuilder)]
#[title(kind = "error", msg = "oh no! this is an error!", code = "E0124")]
#[note(label = "error", msg = "oh no! this is an error!")]
pub struct ThisIsAnErr1 {
    // 这里都是放在title 和 后面中间的部分，这里的顺序怎么写，外面就怎么输出。
    // 在position下面目前只能有一个注解，用来标识代码挂件的一句话
    // 或者可以用挂件的堆叠表示嵌套 TOFEAT(zongz)
    // #[position]
    // #[help]
    // #[note(msg = "Hello World")]
    // $ Code ctx^ help:note: Hello World.
    // 一阶段所有的msg写在注解参数里
    #[position(msg = "oh no! this is an error!")]
    pub pos: Position,
}

#[test]
fn test_macro() {
    let err = ThisIsAnErr1 {
        pos: get_code_position(),
    };

    let mut emitter = EmitterWriter::default();
    emitter.emit_err(err);
}
