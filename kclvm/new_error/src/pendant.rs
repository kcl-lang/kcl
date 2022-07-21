use std::path::Path;
use std::sync::Arc;

use kclvm_span::{SourceMap, FilePathMapping};
use crate::{shader::Shader};
use crate::styled_buffer::StyledBuffer;
use kclvm_error::Position;

// TODO(zongz): The pendant can also be replaced by macros.
trait Pendant{
    fn format(&self, shader: Box<&dyn Shader>, sb: &mut StyledBuffer);
}
struct HeaderPendant {
    diagnostic_type: String,
    diagnostic_code: String,
}

impl HeaderPendant{
    pub fn new(diagnostic_type: String, diagnostic_code: String,) -> Self{
        Self { diagnostic_type, diagnostic_code}
    }
}

impl Pendant for HeaderPendant{
    fn format(&self, shader: Box<&dyn Shader>, sb: &mut StyledBuffer) {
        let line_num = sb.num_lines();
        let col = 0;
        sb.puts(line_num, col, &self.diagnostic_type, shader.header_style());

        let mut offset = self.diagnostic_type.len()+1;
        sb.putc(line_num, col+offset, '[', shader.header_style());

        offset = offset+1;
        sb.puts(line_num, col+offset, &self.diagnostic_code, shader.header_style());

        offset = offset+1+self.diagnostic_code.len();
        sb.putc(line_num, col+offset, ']', shader.header_style());
    }
}

struct LabelPendant {
    diagnostic_type: String,
}

impl LabelPendant{
    fn new(diagnostic_type: String) -> Self{
        Self { diagnostic_type }
    }
}

impl Pendant for LabelPendant{
    fn format(&self, shader: Box<&dyn Shader>, sb: &mut StyledBuffer) {
        let line_num = sb.num_lines();
        let col = 0;
        sb.puts(line_num, col, &self.diagnostic_type, shader.label_style());

        let offset = self.diagnostic_type.len()+1;
        sb.putc(line_num, col+offset, ':', shader.label_style());
    }
}

struct CodeCtxPendant {
    code_pos: Position,
    source_map: Option<Arc<SourceMap>>,
}

impl CodeCtxPendant{
    pub fn new(code_pos: Position, source_map: Option<Arc<SourceMap>>) -> Self{
        Self {code_pos, source_map}
    }
}

impl Pendant for CodeCtxPendant{
    fn format(&self, shader: Box<&dyn Shader>, sb: &mut StyledBuffer) {
        sb.putl(&self.code_pos.info(), shader.file_header_style());

        sb.putl(&format!("{} |", self.code_pos.line), shader.file_header_style());

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
