use std::{path::Path, rc::Rc};
use std::sync::Arc;
use crate::shader::{Shader, Level};
use crate::styled_buffer::StyledBuffer;
use kclvm_error::Position;
use kclvm_span::{FilePathMapping, SourceMap};

// TODO(zongz): The 'impl Pendant' can also be replaced by macros.
pub trait Pendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer);
}

// TODO(zongz): Put this in the lib.rs and combina with macros.
pub struct HeaderPendant {
    logo: Option<String>,
    diag_level: Level,
    diag_code: String,
}

impl HeaderPendant {
    pub fn new(diag_level: Level, diag_code: String) -> Self {
        Self {
            logo: None,
            diag_level,
            diag_code,
        }
    }

    pub fn set_logo(&mut self, logo: String){
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
        if let Some(logo) = &self.logo{
            sb.puts(line_num, col, &logo, shader.logo_style());
        }
        
        // format header -> error[E0101] or warning[W1010]

        // get label text, label text length, style for different level.
        let (label_text, label_len, style) = match self.diag_level{
            Level::Error => ("error", "error".len(), shader.err_style()),
            Level::Warning => ("warning", "warning".len(), shader.warning_style()),
            Level::Note => ("note", "note".len(), shader.msg_style()),
        };

        sb.puts(line_num, col, label_text, style);
        let mut offset = label_len;

        // for e.g. "error[E1010]"
        sb.putc(line_num, col + offset, '[', shader.msg_style());
        offset = offset + 1;
        sb.puts(
            line_num,
            col + offset,
            &self.diag_code,
            shader.msg_style(),
        );

        offset = offset + self.diag_code.len();
        sb.putc(line_num, col + offset, ']', shader.msg_style());

        offset = offset+1;
        sb.putc(line_num, col + offset, ':', shader.msg_style());
    }
}

pub struct LabelPendant {
    diagnostic_type: String,
}

impl LabelPendant {
    pub fn new(diagnostic_type: String) -> Self {
        Self { diagnostic_type }
    }
}

impl Pendant for LabelPendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer) {
        let line_num = sb.num_lines();
        let col = 0;
        sb.puts(line_num, col, &self.diagnostic_type, shader.msg_style());

        let offset = self.diagnostic_type.len();
        sb.putc(line_num, col + offset, ':', shader.msg_style());
    }
}

pub struct CodeCtxPendant {
    code_pos: Position,
    source_map: Option<Arc<SourceMap>>,
}

impl CodeCtxPendant {
    pub fn new(code_pos: Position, source_map: Option<Arc<SourceMap>>) -> Self {
        Self {
            code_pos,
            source_map,
        }
    }
}

impl Pendant for CodeCtxPendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer) {
        sb.putl(&self.code_pos.info(), shader.file_path_style());

        sb.putl(
            &format!("{} |", self.code_pos.line),
            shader.line_and_column_style(),
        );

        if let Some(sm) = &self.source_map {
            if let Some(source_file) = sm.source_file_by_filename(&self.code_pos.filename) {
                if let Some(line) = source_file.get_line(self.code_pos.line as usize - 1) {
                    sb.putl(&line.to_string(), shader.err_style());
                }
            }
        } else {
            let sm = SourceMap::new(FilePathMapping::empty());
            if let Ok(source_file) = sm.load_file(Path::new(&self.code_pos.filename)) {
                if let Some(line) = source_file.get_line(self.code_pos.line as usize - 1) {
                    sb.putl(&line.to_string(), shader.msg_style());
                }
            }
        }
        sb.putl(&format!("^"), shader.err_style());
    }
}
