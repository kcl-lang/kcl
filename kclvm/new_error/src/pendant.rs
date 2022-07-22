use std::{path::Path, rc::Rc};
use std::sync::Arc;

use crate::shader::Shader;
use crate::styled_buffer::StyledBuffer;
use kclvm_error::Position;
use kclvm_span::{FilePathMapping, SourceMap};

// TODO(zongz): The pendant can also be replaced by macros.
pub trait Pendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer);
}
pub struct HeaderPendant {
    diagnostic_type: String,
    diagnostic_code: String,
}

impl HeaderPendant {
    pub fn new(diagnostic_type: String, diagnostic_code: String) -> Self {
        Self {
            diagnostic_type,
            diagnostic_code,
        }
    }
}

impl Pendant for HeaderPendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer) {
        // 诊断信息中的KCL logo可以在这里加进去，style也可以给他搞一个logo_style
        let line_num = sb.num_lines();
        let col = 0;
        sb.puts(line_num, col, &self.diagnostic_type, shader.header_style());

        let mut offset = self.diagnostic_type.len();
        sb.putc(line_num, col + offset, '[', shader.header_style());

        offset = offset + 1;
        sb.puts(
            line_num,
            col + offset,
            &self.diagnostic_code,
            shader.header_style(),
        );

        offset = offset + self.diagnostic_code.len();
        sb.putc(line_num, col + offset, ']', shader.header_style());
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
        sb.puts(line_num, col, &self.diagnostic_type, shader.label_style());

        let offset = self.diagnostic_type.len();
        sb.putc(line_num, col + offset, ':', shader.label_style());
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
        sb.putl(&self.code_pos.info(), shader.file_header_style());

        sb.putl(
            &format!("{} |", self.code_pos.line),
            shader.file_header_style(),
        );

        if let Some(sm) = &self.source_map {
            if let Some(source_file) = sm.source_file_by_filename(&self.code_pos.filename) {
                if let Some(line) = source_file.get_line(self.code_pos.line as usize - 1) {
                    sb.putl(&line.to_string(), shader.file_header_style());
                }
            }
        } else {
            let sm = SourceMap::new(FilePathMapping::empty());
            if let Ok(source_file) = sm.load_file(Path::new(&self.code_pos.filename)) {
                if let Some(line) = source_file.get_line(self.code_pos.line as usize - 1) {
                    sb.putl(&line.to_string(), shader.line_and_column_style());
                }
            }
        }
        sb.putl(&format!("^"), shader.line_and_column_style());
    }
}
