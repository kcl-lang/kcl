// Copyright 2021 The KCL Authors. All rights reserved.

pub mod api;
pub use api::*;
use std::fmt;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = crate::ValueRef;

impl fmt::Display for crate::PanicInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl crate::PanicInfo {
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
}

impl crate::Context {
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    pub fn current_context() -> &'static crate::Context {
        unsafe {
            let ctx = kclvm_context_current();
            &*ctx
        }
    }

    pub fn current_context_mut() -> &'static mut crate::Context {
        unsafe {
            let ctx = kclvm_context_current();
            &mut *ctx
        }
    }

    pub fn main_begin_hook(&mut self) {
        // Nothing to do
    }

    pub fn main_end_hook(
        &mut self,
        return_value: *mut kclvm_value_ref_t,
    ) -> *mut kclvm_value_ref_t {
        self.output.return_value = return_value;

        if self.cfg.list_option_mode {
            self.output.return_value =
                crate::ValueRef::str(self.list_option_help().as_str()).into_raw();
        }

        self.output.return_value
    }

    pub fn get_panic_info_json_string(&self) -> String {
        self.panic_info.to_json_string()
    }

    pub fn set_kcl_pkgpath(&mut self, pkgpath: &str) {
        self.panic_info.kcl_pkgpath = pkgpath.to_string();
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

    pub fn set_err_type(&mut self, err_type: &crate::ErrType) {
        self.panic_info.__kcl_PanicInfo__ = true;
        self.panic_info.err_type_code = *err_type as i32;
    }
    pub fn set_warnning_message(&mut self, msg: &str) {
        self.panic_info.__kcl_PanicInfo__ = true;
        self.panic_info.message = msg.to_string();
        self.panic_info.is_warning = true;
    }

    pub fn set_panic_info(&mut self, info: &std::panic::PanicInfo) {
        self.panic_info.__kcl_PanicInfo__ = true;

        if let Some(s) = info.payload().downcast_ref::<&str>() {
            self.panic_info.message = s.to_string();
        } else if let Some(s) = info.payload().downcast_ref::<&String>() {
            self.panic_info.message = (*s).clone();
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            self.panic_info.message = (*s).clone();
        } else {
            self.panic_info.message = "".to_string();
        }

        if let Some(location) = info.location() {
            self.panic_info.rust_file = location.file().to_string();
            self.panic_info.rust_line = location.line() as i32;
            self.panic_info.rust_col = location.column() as i32;
        } else {
            self.panic_info.rust_file = "".to_string();
            self.panic_info.rust_line = 0;
            self.panic_info.rust_col = 0;
        }
    }
}

impl crate::Context {
    pub fn define_option(
        &mut self,
        name: &str,
        typ: &str,
        required: bool,
        default_value: Option<String>,
        help: &str,
    ) {
        // check dup
        for i in 0..self.option_helps.len() {
            if self.option_helps[i].name == name {
                if typ.is_empty() && !required && default_value == None && help.is_empty() {
                    return;
                }

                if self.option_helps[i].typ.is_empty() {
                    self.option_helps[i].typ = typ.to_string();
                }

                if !self.option_helps[i].required {
                    self.option_helps[i].required = required;
                }
                if self.option_helps[i].default_value == None {
                    self.option_helps[i].default_value = default_value;
                }
                if self.option_helps[i].help.is_empty() {
                    self.option_helps[i].help = help.to_string();
                }

                return;
            }
        }

        self.option_helps.push(crate::OptionHelp {
            name: name.to_string(),
            typ: typ.to_string(),
            required,
            default_value,
            help: help.to_string(),
        });
    }

    pub fn list_option_help(&self) -> String {
        let mut msg: String = "".to_string();

        // name=? (required) set name value
        // name=? (str,required) set name value
        // a=42 set a value
        // b=? set b value
        // obj=?
        // obj2=?

        msg.push_str("option list:\n");
        for opt in &self.option_helps {
            let name = opt.name.clone();

            let mut default_value: String = "?".to_string();
            if let Some(ref v) = opt.default_value {
                default_value = (*v).clone();
            }

            let s = format!("  -D {}={}", name, default_value);
            msg.push_str(s.as_str());

            // (required)
            // (str,required)
            if !opt.typ.is_empty() || opt.required {
                if opt.required && !opt.typ.is_empty() {
                    let s = format!(" ({},{})", opt.typ, "required");
                    msg.push_str(s.as_str());
                } else if !opt.typ.is_empty() {
                    let s = format!(" ({})", opt.typ);
                    msg.push_str(s.as_str());
                } else {
                    msg.push_str(" (required)");
                }
            }

            if !opt.help.is_empty() {
                msg.push(' ');
                msg.push_str(opt.help.as_str());
            }

            msg.push('\n');
        }

        msg = msg.as_str().trim_end_matches('\n').to_string();
        msg
    }
}
