mod diagnostic;
mod emitter;

use std::sync::Arc;

use anyhow::{Context, Result};
use compiler_base_span::SourceMap;
use diagnostic::{diagnostic_message::TemplateLoader, style::DiagnosticStyle, Diagnostic};
use emitter::{Emitter, TerminalEmitter};

pub struct ErrorHandler {
    source_map: Arc<SourceMap>,
    template_loader: Arc<TemplateLoader>,
    emitter: Box<dyn Emitter<DiagnosticStyle>>,
    diagnostics: Vec<Diagnostic<DiagnosticStyle>>,
}

impl ErrorHandler {
    pub fn new(source_map: Arc<SourceMap>, template_dir: &str) -> Result<Self> {
        let template_loader = TemplateLoader::new_with_template_dir(template_dir)
            .with_context(|| format!("Failed to init `TemplateLoader` from '{}'", template_dir))?;
        Ok(Self {
            source_map,
            template_loader: Arc::new(template_loader),
            emitter: Box::new(TerminalEmitter::default()),
            diagnostics: vec![],
        })
    }

    pub fn add_diagnostic(&mut self, diag_builder: impl DiagnosticBuilder) {
        self.diagnostics.push(diag_builder.into_diagnostic(
            Arc::clone(&self.source_map),
            Arc::clone(&self.template_loader),
        ));
    }

    pub fn emit_err(&mut self, diag_builder: impl DiagnosticBuilder) {
        self.emitter.emit_diagnostic(&diag_builder.into_diagnostic(
            Arc::clone(&self.source_map),
            Arc::clone(&self.template_loader),
        ));
    }

    pub fn emit_all_errs(&mut self) {
        for diag in &self.diagnostics {
            self.emitter.emit_diagnostic(&diag)
        }
    }
}

pub trait DiagnosticBuilder {
    fn into_diagnostic(
        self,
        sm: Arc<SourceMap>,
        template_loader: Arc<TemplateLoader>, // 那就应该只把template_loader做成对外的接口, 相反ErrorMessage不太需要给开发者用
    ) -> Diagnostic<DiagnosticStyle>;
}

struct InvalidSyntaxError;

impl DiagnosticBuilder for InvalidSyntaxError {
    fn into_diagnostic(
        self,
        sm: Arc<SourceMap>,
        template_loader: Arc<TemplateLoader>,
    ) -> Diagnostic<DiagnosticStyle> {
        todo!()
    }
}
