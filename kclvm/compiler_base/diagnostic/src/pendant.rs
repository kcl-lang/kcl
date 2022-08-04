use crate::{Pendant, Position};
use kclvm_span::{FilePathMapping, SourceMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::{path::Path, rc::Rc};
use compiler_base_style::styled_buffer::StyledBuffer;
use compiler_base_style::Shader;

// TODO(zongz): Put this in the lib.rs and combina with macros.
pub struct HeaderPendant {
    logo: Option<String>,
    diag_label: String,
    diag_code: Option<String>,
}

impl HeaderPendant {
    pub fn new(diag_label: String, diag_code: Option<String>) -> Self {
        Self {
            logo: None,
            diag_label,
            diag_code,
        }
    }

    pub fn set_logo(&mut self, logo: String) {
        self.logo = Some(logo);
    }

    pub fn get_logo(&self) -> String {
        self.logo.clone().unwrap()
    }
}

// TODO(zongz): These are not part of CompilerBase.
// Generated them by macro in the
impl Pendant for HeaderPendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer) {
        let line_num = sb.num_lines();
        let col = 0;
        let mut offset = 0;

        if let Some(logo) = &self.logo {
            sb.puts(line_num, col, &logo, shader.logo_style());
            offset = offset + logo.len();
        }

        let label_text = self.diag_label.as_str();
        let label_len = label_text.len();
        let style = match label_text {
            "error" => shader.need_fix_style(),
            "warning" | "help" | "note" => shader.need_attention_style(),
            _ => shader.normal_msg_style(),
        };
        sb.puts(line_num, col + offset, label_text, style);
        offset = offset + label_len;

        // for e.g. "error[E1010]"
        if let Some(c) = &self.diag_code {
            sb.putc(line_num, col + offset, '[', shader.helpful_style());
            offset = offset + 1;

            sb.puts(line_num, col + offset, c.as_str(), shader.helpful_style());
            offset = offset + c.len();

            sb.putc(line_num, col + offset, ']', shader.helpful_style());
            offset = offset + 1;
        }

        sb.putc(line_num, col + offset, ':', shader.normal_msg_style());
    }
}

pub struct CodeCtxPendant {
    code_pos: Position,
    source_map: Option<Arc<SourceMap>>,
}

impl CodeCtxPendant {
    /// Share source_map with the outside through input parameter 'source_map: Option<Arc<SourceMap>>'.
    pub fn new_with_source_map(code_pos: Position, source_map: Option<Arc<SourceMap>>) -> Self {
        Self {
            code_pos,
            source_map,
        }
    }

    /// Create a new source_map by code_pos.filename.
    pub fn new(code_pos: Position) -> Self {
        let source_map = Arc::new(CodeCtxPendant::init_source_map(&code_pos.filename));

        Self {
            code_pos,
            source_map: Some(source_map),
        }
    }

    pub fn init_source_map(filename: &String) -> SourceMap {
        let src = std::fs::read_to_string(filename.clone()).unwrap();
        let sm = kclvm_span::SourceMap::new(FilePathMapping::empty());
        sm.new_source_file(PathBuf::from(filename.clone()).into(), src.to_string());
        sm
    }
}

impl Pendant for CodeCtxPendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer) {
        sb.putl(&self.code_pos.info(), shader.url_style());

        let line = self.code_pos.line.to_string();
        let indent = line.len() + 1;

        sb.putl(&format!("{:<indent$}|", ""), shader.normal_msg_style());
        sb.putl(&format!("{:<indent$}", &line), shader.url_style());
        sb.appendl("|", shader.normal_msg_style());

        if let Some(sm) = &self.source_map {
            if let Some(source_file) = sm.source_file_by_filename(&self.code_pos.filename) {
                if let Some(line) = source_file.get_line(self.code_pos.line as usize - 1) {
                    sb.appendl(&line.to_string(), shader.normal_msg_style());
                }
            }
        } else {
            let sm = SourceMap::new(FilePathMapping::empty());
            if let Ok(source_file) = sm.load_file(Path::new(&self.code_pos.filename)) {
                if let Some(line) = source_file.get_line(self.code_pos.line as usize - 1) {
                    sb.appendl(&line.to_string(), shader.normal_msg_style());
                }
            }
        }
        sb.putl(&format!("{:<indent$}|", ""), shader.normal_msg_style());

        let col = self.code_pos.column;
        if let Some(col) = col {
            let col = col as usize;
            sb.appendl(&format!("{:>col$}^ ", col), shader.need_fix_style());
        }
    }
}

pub struct NoPendant;

impl NoPendant {
    pub fn new() -> Self {
        Self {}
    }
}

impl Pendant for NoPendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer) {
        sb.putl("- ", shader.normal_msg_style());
    }
}
