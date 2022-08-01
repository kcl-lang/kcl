use crate::{Pendant, Position};
use core::panic;
use kclvm_span::{FilePathMapping, SourceMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::{path::Path, rc::Rc};
use style::styled_buffer::StyledBuffer;
use style::Shader;

// TODO(zongz): Put this in the lib.rs and combina with macros.
pub struct HeaderPendant {
    logo: Option<String>,
    diag_level: String,
    diag_code: Option<String>,
}

impl HeaderPendant {
    pub fn new(diag_level: String, diag_code: Option<String>) -> Self {
        Self {
            logo: None,
            diag_level,
            diag_code,
        }
    }

    pub fn set_logo(&mut self, logo: String) {
        self.logo = Some(logo);
    }
}

// TODO(zongz): These are not part of CompilerBase.
// Generated them by macro in the
impl Pendant for HeaderPendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer) {
        let line_num = sb.num_lines();
        let col = 0;

        // format logo
        if let Some(logo) = &self.logo {
            sb.puts(line_num, col, &logo, shader.logo_style());
        }

        // format header -> error[E0101] or warning[W1010]

        // get label text, label text length, style for different level.
        let (label_text, label_len, style) = match self.diag_level.as_str() {
            "error" => ("error", "error".len(), shader.need_fix_style()),
            "warning" => ("warning", "warning".len(), shader.need_attention_style()),
            _ => {
                panic!("Internal bug")
            }
        };

        sb.puts(line_num, col, label_text, style);
        let mut offset = label_len;

        // for e.g. "error[E1010]"
        sb.putc(line_num, col + offset, '[', shader.helpful_style());
        offset = offset + 1;
        sb.puts(line_num, col + offset, "E0000", shader.helpful_style());

        offset = offset + "E0000".len();
        sb.putc(line_num, col + offset, ']', shader.helpful_style());

        offset = offset + 1;
        sb.putc(line_num, col + offset, ':', shader.helpful_style());
    }
}

pub struct LabelPendant {
    diagnostic_type: String,
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

        sb.putl(&format!("{} |", self.code_pos.line), shader.url_style());

        if let Some(sm) = &self.source_map {
            if let Some(source_file) = sm.source_file_by_filename(&self.code_pos.filename) {
                if let Some(line) = source_file.get_line(self.code_pos.line as usize - 1) {
                    sb.putl(&line.to_string(), shader.url_style());
                }
            }
        } else {
            let sm = SourceMap::new(FilePathMapping::empty());
            if let Ok(source_file) = sm.load_file(Path::new(&self.code_pos.filename)) {
                if let Some(line) = source_file.get_line(self.code_pos.line as usize - 1) {
                    sb.putl(&line.to_string(), shader.url_style());
                }
            }
        }
        sb.putl(&format!("^"), shader.need_fix_style());
    }
}
