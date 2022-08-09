//! 'Pendant' defines the template of pendant, 
//! and provides three builtin templates, 
//! HeaderPendant, CodeC&txPendant and NoPendant.

use crate::{Pendant, Position};
use compiler_base_style::diagnostic_style::Shader;
use rustc_errors::styled_buffer::StyledBuffer;
use rustc_span::source_map::FilePathMapping;
use rustc_span::{SourceMap, Span, Loc};
use std::path::PathBuf;
use std::sync::Arc;
use std::{path::Path, rc::Rc};

/// HeaderPendant: A pendant to shown some short messages for diagnostics.
/// 
/// e.g. 
/// error: this is an error!
/// warning[W0011]: this is an warning!
/// note: this is note.
/// KCL error[E1000]: this is a KCL error!
/// 
/// 'error', 'warning[W0011]', 'KCL error[E1000]' and 'note' are 'HeaderPendant'.
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

/// CodePosPendant: A pendant to shown some code context for diagnostics.
/// 
/// --> mycode.rs:3:5
///  |
///3 |     a:int
///  |     ^ error here!
/// 
/// The CodePosPendant looks like:
/// --> mycode.rs:3:5
///  |
///3 |     a:int
///  |     ^ 
pub struct CodePosPendant {
    code_pos: Position,
    // TOFIX(zongz): 这里应该用sourcefile而不是sourcemap，map是很多文件。
    // 不对这里就应该是sourcemap，然后通过span从sourcemap中定位对应的filename，line之类的属性。
    source_map: Option<Arc<SourceMap>>,
}

impl CodePosPendant {
    /// Share source_map with the outside through input parameter 'source_map: Option<Arc<SourceMap>>'.
    pub fn new_with_source_map(code_pos: Position, source_map: Option<Arc<SourceMap>>) -> Self {
        Self {
            code_pos,
            source_map,
        }
    }

    /// Create a new source_map by code_pos.filename.
    pub fn new(code_pos: Position) -> Self {
        let source_map = Arc::new(CodePosPendant::init_source_map(&code_pos.filename));

        Self {
            code_pos,
            source_map: Some(source_map),
        }
    }

    pub fn init_source_map(filename: &String) -> SourceMap {
        let src = std::fs::read_to_string(filename.clone()).unwrap();
        let sm = SourceMap::new(FilePathMapping::empty());
        sm.new_source_file(PathBuf::from(filename.clone()).into(), src.to_string());
        sm
    }
}

impl Pendant for CodePosPendant {
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
            sb.appendl(&format!("{:>col$} ", format!("^{}",col)), shader.need_fix_style());
        }
    }
}

/// CodeSpanPendant: A pendant to shown some code context for diagnostics.
/// 
/// --> mycode.rs:3:5
///  |
///3 |     a:int
///  |     ^ error here!
/// 
/// The CodeSpanPendant looks like:
/// --> mycode.rs:3:5
///  |
///3 |     a:int
///  |     ^ 
pub struct CodeSpanPendant {
    code_span: Span,
    source_map: Option<Arc<SourceMap>>,
}

impl CodeSpanPendant {
    /// Share source_map with the outside through input parameter 'source_map: Option<Arc<SourceMap>>'.
    pub fn new_with_source_map(code_span: Span, source_map: Option<Arc<SourceMap>>) -> Self {
        Self {
            code_span,
            source_map,
        }
    }
}

impl CodeSpanPendant{
    //TOFIX(zongz): 这里后面应该直接由minihandler接管。
    fn require_loc_in_same_line(&self, start: &Loc, end: &Loc){
        assert!(start.line == end.line)
    }

    //TOFIX(zongz): 这里后面应该直接由minihandler接管。
    fn require_end_behind_start(&self, start: &Loc, end: &Loc){
        assert!(start.col_display <= end.col_display)
    }

    fn draw_code_underscore(&self, start: Loc, end: Loc, shader: Rc<dyn Shader>, sb: &mut StyledBuffer){
        self.require_loc_in_same_line(&start, &end);
        self.require_end_behind_start(&start, &end);

        let start_col = start.col_display;
        let end_col = end.col_display;
        let col_offset = end_col - start_col;

        sb.appendl(&format!("{:>start_col$}", "^"), shader.need_fix_style());
        sb.appendl(&format!("{:^>col_offset$} ", ""), shader.need_fix_style());
    }
}

impl Pendant for CodeSpanPendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer) {
        sb.putl("---> File: ", shader.need_attention_style());

        let filename = format!("{}", self.source_map.as_ref().unwrap().span_to_filename(self.code_span).prefer_remapped());
        sb.appendl(&filename, shader.url_style());

        let loc_lo = self.source_map.as_ref().unwrap().lookup_char_pos(self.code_span.lo());
        let loc_hi = self.source_map.as_ref().unwrap().lookup_char_pos(self.code_span.hi());

        let line = loc_lo.line.to_string();
        let indent = line.len() + 1;

        sb.putl(&format!("{:<indent$}|", ""), shader.normal_msg_style());
        sb.putl(&format!("{:<indent$}", &line), shader.url_style());
        sb.appendl("|", shader.normal_msg_style());

        if let Some(sm) = &self.source_map {
            if let Some(source_file) = sm.source_file_by_filename(&filename) {
                if let Some(line) = source_file.get_line(loc_lo.line as usize - 1) {
                    sb.appendl(&line.to_string(), shader.normal_msg_style());
                }
            }
        } else {
            let sm = SourceMap::new(FilePathMapping::empty());
            if let Ok(source_file) = sm.load_file(Path::new(&filename)) {
                if let Some(line) = source_file.get_line(loc_lo.line as usize - 1) {
                    sb.appendl(&line.to_string(), shader.normal_msg_style());
                }
            }
        }
        sb.putl(&format!("{:<indent$}|", ""), shader.normal_msg_style());
        self.draw_code_underscore(loc_lo, loc_hi, shader, sb);
    }
}


/// NoPendant: A pendant for some sentences with no pendants.
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
