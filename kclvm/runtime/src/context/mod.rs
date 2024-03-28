//! Copyright The KCL Authors. All rights reserved.

pub mod api;
pub use api::*;
use std::fmt;

use crate::{BacktraceFrame, PanicInfo, RuntimePanicRecord};

impl fmt::Display for PanicInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PanicInfo {
    pub fn to_json_string(&self) -> String {
        let result = serde_json::to_string(&self);
        match result {
            Ok(res) => res,
            _ => {
                panic!("PanicInfo Deserialize Failed")
            }
        }
    }

    /// Parse a json string to a PanicInfo.
    pub fn from_json_string(s: &str) -> Self {
        let result = serde_json::from_str(s);
        match result {
            Ok(res) => res,
            _ => {
                panic!("PanicInfo Deserialize Failed")
            }
        }
    }

    ///  Parse a string or json string to a PanicInfo.
    pub fn from_string(s: &str) -> Self {
        let result = serde_json::from_str(s);
        match result {
            Ok(res) => res,
            Err(_) => PanicInfo {
                __kcl_PanicInfo__: true,
                message: s.to_string(),
                err_type_code: crate::RuntimeErrorType::EvaluationError as i32,
                ..Default::default()
            },
        }
    }
}

impl From<String> for PanicInfo {
    fn from(value: String) -> Self {
        Self::from_string(&value)
    }
}

impl PanicInfo {
    /// New a [`PanicInfo`] from error message [`value`] and the position that error occur.
    pub fn from_ast_pos(value: String, pos: (String, u64, u64, u64, u64)) -> Self {
        let mut panic_info = Self::from_string(&value);
        panic_info.kcl_file = pos.0;
        panic_info.kcl_line = pos.1 as i32;
        panic_info.kcl_col = pos.2 as i32;
        panic_info
    }
}

impl From<&str> for PanicInfo {
    fn from(value: &str) -> Self {
        Self::from_string(value)
    }
}

impl crate::Context {
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    pub fn get_panic_info_json_string(&self) -> Option<String> {
        if self.panic_info.__kcl_PanicInfo__ {
            Some(self.panic_info.to_json_string())
        } else {
            None
        }
    }

    pub fn set_kcl_pkgpath(&mut self, pkgpath: &str) {
        self.panic_info.kcl_pkgpath = pkgpath.to_string();
    }

    pub fn set_kcl_module_path(&mut self, module_path: &str) {
        self.module_path = module_path.to_string();
    }

    pub fn set_kcl_workdir(&mut self, workdir: &str) {
        self.workdir = workdir.to_string();
    }

    pub fn set_kcl_filename(&mut self, file: &str) {
        if !file.is_empty() {
            self.panic_info.kcl_file = file.to_string();
        }
    }

    pub fn set_kcl_line_col(&mut self, line: i32, col: i32) {
        self.panic_info.kcl_line = line;
        self.panic_info.kcl_col = col;
    }

    pub fn set_kcl_location_info(
        &mut self,
        arg_msg: Option<&str>,
        file: Option<&str>,
        line: Option<i32>,
        col: Option<i32>,
    ) {
        if let Some(s) = arg_msg {
            self.panic_info.kcl_arg_msg = s.to_string();
        }
        if let Some(s) = file {
            self.panic_info.kcl_file = s.to_string();
        }
        if let Some(line) = line {
            self.panic_info.kcl_line = line;
        }
        if let Some(col) = col {
            self.panic_info.kcl_col = col;
        }
    }

    pub fn set_kcl_config_meta_location_info(
        &mut self,
        arg_msg: Option<&str>,
        file: Option<&str>,
        line: Option<i32>,
        col: Option<i32>,
    ) {
        if let Some(s) = arg_msg {
            self.panic_info.kcl_config_meta_arg_msg = s.to_string();
        }
        if let Some(s) = file {
            self.panic_info.kcl_config_meta_file = s.to_string();
        }
        if let Some(line) = line {
            self.panic_info.kcl_config_meta_line = line;
        }
        if let Some(col) = col {
            self.panic_info.kcl_config_meta_col = col;
        }
    }

    pub fn set_err_type(&mut self, err_type: &crate::RuntimeErrorType) {
        self.panic_info.__kcl_PanicInfo__ = true;
        self.panic_info.err_type_code = *err_type as i32;
    }

    pub fn set_warning_message(&mut self, msg: &str) {
        self.panic_info.__kcl_PanicInfo__ = true;
        self.panic_info.message = msg.to_string();
        self.panic_info.is_warning = true;
    }

    pub fn set_panic_info(&mut self, record: &RuntimePanicRecord) {
        self.panic_info.__kcl_PanicInfo__ = true;

        self.panic_info.message = record.message.clone();
        if self.cfg.debug_mode {
            self.panic_info.backtrace = self.backtrace.clone();
            self.panic_info.backtrace.push(BacktraceFrame {
                file: self.panic_info.kcl_file.clone(),
                func: self.panic_info.kcl_func.clone(),
                col: self.panic_info.kcl_col,
                line: self.panic_info.kcl_line,
            });
        }

        self.panic_info.rust_file = record.rust_file.clone();
        self.panic_info.rust_line = record.rust_line;
        self.panic_info.rust_col = record.rust_col;
    }
}
