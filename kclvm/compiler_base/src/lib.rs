use compiler_base_diagnostic::{
    emitter::{Emitter, EmitterWriter},
    DiagnosticBuilder,
};
use std::{panic, sync::Arc};

pub struct ErrHandler {
    emitter: Box<dyn Emitter>,
    source_map: Arc<SourceMap>,
}

impl ErrHandler {
    pub fn new() -> Self {
        Self {
            emitter: Box::new(EmitterWriter::default()),
        }
    }

    pub fn after_emit(&self) {
        panic::set_hook(Box::new(|_| {}));
        panic!()
    }

    pub fn emit_err(&mut self, err: impl DiagnosticBuilder) {
        self.emitter.emit_diagnostic(&err.into_diagnostic());
        self.after_emit();
    }
}
