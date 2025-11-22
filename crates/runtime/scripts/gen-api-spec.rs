use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::process;
use std::process::Command;
use std::process::ExitStatus;
use walkdir::WalkDir;

const ROOT: &str = "./src";
const C_API_FILE: &str = "./src/_kclvm.h";
const LL_API_FILE: &str = "./src/_kclvm.ll";
const RUST_API_ENUM: &str = "./src/_kclvm.rs";
const RUST_API_ADDR: &str = "./src/_kclvm_addr.rs";

#[derive(Debug, Default)]
struct ApiSpec {
    file: String,
    line: usize,
    name: String,
    spec_c: String,
    spec_ll: String,
    is_type: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        std::env::set_var("KCLVM_RUNTIME_GEN_API_SPEC", "1");
    }
    let specs = load_all_api_spec(ROOT);
    let src = gen_c_api(&specs);
    fs::write(C_API_FILE, src).unwrap_or_else(|err| {
        eprintln!("Failed to write C API file: {}", err);
        process::exit(1);
    });

    let src = gen_ll_api(&specs);
    fs::write(LL_API_FILE, src).unwrap_or_else(|err| {
        eprintln!("Failed to write LLVM API file: {}", err);
        process::exit(1);
    });

    let src = gen_rust_api_enum(&specs);
    fs::write(RUST_API_ENUM, src).unwrap_or_else(|err| {
        eprintln!("Failed to write Rust API Enum file: {}", err);
        process::exit(1);
    });

    let src = gen_rust_api_addr(&specs);
    fs::write(RUST_API_ADDR, src).unwrap_or_else(|err| {
        eprintln!("Failed to write Rust API Addr file: {}", err);
        process::exit(1);
    });

    run_llvm_as(LL_API_FILE)?;
    run_cargo_fmt()?;
    Ok(())
}

fn load_all_api_spec(root: &str) -> Vec<ApiSpec> {
    let mut specs: HashMap<String, ApiSpec> = HashMap::new();
    let api_spec_prefix_name = "// api-spec:";
    let api_spec_prefix_c = "// api-spec(c):";
    let api_spec_prefix_ll = "// api-spec(llvm):";

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir()
            || !path
                .to_str()
                .expect(&format!("{path:?} not found"))
                .ends_with(".rs")
        {
            continue;
        }
        let data = fs::read_to_string(path).expect(&format!("{path:?} not found"));
        let mut spec = ApiSpec::default();

        for (i, line) in data.lines().enumerate() {
            let line = line.trim();

            if line.starts_with(api_spec_prefix_name) {
                spec.file = path.display().to_string();
                spec.line = i + 1;
                spec.name = line
                    .trim_start_matches(api_spec_prefix_name)
                    .trim()
                    .to_string();
                spec.is_type = spec.name.ends_with("_t");
            } else if line.starts_with(api_spec_prefix_c) {
                if !spec.spec_c.is_empty() {
                    spec.spec_c.push(' ');
                }
                spec.spec_c
                    .push_str(line.trim_start_matches(api_spec_prefix_c).trim());
            } else if line.starts_with(api_spec_prefix_ll) {
                if !spec.spec_ll.is_empty() {
                    spec.spec_ll.push(' ');
                }
                spec.spec_ll
                    .push_str(line.trim_start_matches(api_spec_prefix_ll).trim());
            } else {
                if !spec.name.is_empty() {
                    if let Some(existing) = specs.get(&spec.name) {
                        eprintln!(
                            "WARN: {}:{} {} api-spec exists ({}:{})",
                            path.display(),
                            i + 1,
                            spec.name,
                            existing.file,
                            existing.line
                        );
                    }
                    specs.insert(spec.name.clone(), spec);
                }
                spec = ApiSpec::default();
            }
        }
    }

    let mut spec_list: Vec<ApiSpec> = specs.into_values().collect();
    spec_list.sort_by(|a, b| a.name.cmp(&b.name));
    spec_list
}

fn gen_c_api(specs: &[ApiSpec]) -> String {
    let mut buf = String::new();

    buf.push_str("// Copyright The KCL Authors. All rights reserved.\n\n");
    buf.push_str("// Auto generated, DONOT EDIT!!!\n\n");
    buf.push_str("#pragma once\n\n");
    buf.push_str("#ifndef _kclvm_h_\n#define _kclvm_h_\n\n");
    buf.push_str("#include <stdarg.h>\n#include <stdbool.h>\n#include <stdint.h>\n\n");
    buf.push_str("#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n");

    buf.push_str("// please keep same as 'kclvm/runtime/src/kind/mod.rs#Kind'\n\n");
    buf.push_str("enum kclvm_kind_t {\n");
    buf.push_str("    Invalid = 0,\n");
    buf.push_str("    Undefined = 1,\n");
    buf.push_str("    None = 2,\n");
    buf.push_str("    Bool = 3,\n");
    buf.push_str("    Int = 4,\n");
    buf.push_str("    Float = 5,\n");
    buf.push_str("    Str = 6,\n");
    buf.push_str("    List = 7,\n");
    buf.push_str("    Dict = 8,\n");
    buf.push_str("    Schema = 9,\n");
    buf.push_str("    Error = 10,\n");
    buf.push_str("    Any = 11,\n");
    buf.push_str("    Union = 12,\n");
    buf.push_str("    BoolLit = 13,\n");
    buf.push_str("    IntLit = 14,\n");
    buf.push_str("    FloatLit = 15,\n");
    buf.push_str("    StrLit = 16,\n");
    buf.push_str("    Func = 17,\n");
    buf.push_str("    Max = 18,\n");
    buf.push_str("};\n\n");

    for spec in specs {
        if spec.is_type {
            buf.push_str(&spec.spec_c);
            buf.push_str("\n\n");
        }
    }

    for spec in specs {
        if !spec.is_type {
            buf.push_str(&spec.spec_c);
            buf.push_str("\n\n");
        }
    }

    buf.push_str("#ifdef __cplusplus\n} // extern \"C\"\n#endif\n\n");
    buf.push_str("#endif // _kclvm_h_\n");

    fmt_code(&buf)
}

fn gen_ll_api(specs: &[ApiSpec]) -> String {
    let mut buf = String::new();

    buf.push_str("; Copyright The KCL Authors. All rights reserved.\n\n");
    buf.push_str("; Auto generated, DONOT EDIT!!!\n\n");

    for spec in specs {
        if spec.is_type {
            buf.push_str(&spec.spec_ll);
            buf.push_str("\n\n");
        }
    }

    for spec in specs {
        if !spec.is_type {
            buf.push_str(&spec.spec_ll);
            buf.push_str("\n\n");
        }
    }

    buf.push_str(
        "define void @__kcl_keep_link_runtime(%kclvm_value_ref_t* %_a, %kclvm_context_t* %_b) {\n",
    );
    buf.push_str("    call %kclvm_value_ref_t* @kclvm_value_None(%kclvm_context_t* %_b)\n");
    buf.push_str("    ret void\n");
    buf.push_str("}\n");

    fmt_code(&buf)
}

fn gen_rust_api_enum(specs: &[ApiSpec]) -> String {
    let mut buf = String::new();

    buf.push_str("// Copyright The KCL Authors. All rights reserved.\n\n");
    buf.push_str("// Auto generated, DONOT EDIT!!!\n\n");

    // Enum ApiType
    buf.push_str("#[allow(dead_code, non_camel_case_types)]\n");
    buf.push_str("#[derive(Clone, PartialEq, Eq, Debug, Hash)]\n");
    buf.push_str("pub enum ApiType {\n");
    buf.push_str("    Value,\n");
    buf.push_str("}\n");
    buf.push('\n');
    buf.push_str("impl std::fmt::Display for ApiType {\n");
    buf.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n");
    buf.push_str("        match self {\n");
    buf.push_str("            ApiType::Value => write!(f, \"{:?}\", \"api::kclvm::Value\"),\n");
    buf.push_str("        }\n");
    buf.push_str("    }\n");
    buf.push_str("}\n");
    buf.push('\n');
    buf.push_str("impl ApiType {\n");
    buf.push_str("    #[allow(dead_code)]\n");
    buf.push_str("    pub fn name(&self) -> String {\n");
    buf.push_str("        format!(\"{self:?}\")\n");
    buf.push_str("    }\n");
    buf.push_str("}\n");
    buf.push('\n');
    // Enum ApiFunc
    buf.push_str("#[allow(dead_code, non_camel_case_types)]\n");
    buf.push_str("#[derive(Clone, PartialEq, Eq, Debug, Hash)]\n");
    buf.push_str("pub enum ApiFunc {\n");

    for spec in specs {
        if !spec.is_type {
            buf.push_str("    ");
            buf.push_str(&spec.name);
            buf.push_str(",\n");
        }
    }

    buf.push_str("}\n");
    buf.push('\n');
    buf.push_str("impl std::fmt::Display for ApiFunc {\n");
    buf.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n");
    buf.push_str("        write!(f, \"{self:?}\")\n");
    buf.push_str("    }\n");
    buf.push_str("}\n");
    buf.push('\n');
    buf.push_str("impl ApiFunc {\n");
    buf.push_str("    #[allow(dead_code)]\n");
    buf.push_str("    pub fn name(&self) -> String {\n");
    buf.push_str("        format!(\"{self:?}\")\n");
    buf.push_str("    }\n");
    buf.push_str("}\n");

    fmt_code(&buf)
}

fn gen_rust_api_addr(specs: &[ApiSpec]) -> String {
    let mut buf = String::new();

    buf.push_str("// Copyright The KCL Authors. All rights reserved.\n\n");
    buf.push_str("// Auto generated, DONOT EDIT!!!\n\n");

    buf.push_str("#[allow(dead_code)]\n");
    buf.push_str("pub fn _kclvm_get_fn_ptr_by_name(name: &str) -> u64 {\n");
    buf.push_str("    match name {\n");

    for spec in specs {
        if !spec.is_type {
            buf.push_str("        \"");
            buf.push_str(&spec.name);
            buf.push_str("\" => crate::");
            buf.push_str(&spec.name);
            buf.push_str(" as *const () as u64,\n");
        }
    }

    buf.push_str("        _ => panic!(\"unknown {name}\"),\n");
    buf.push_str("    }\n");
    buf.push_str("}\n");

    fmt_code(&buf)
}

fn fmt_code(s: &str) -> String {
    s.split("\n\n\n")
        .collect::<Vec<&str>>()
        .join("\n\n")
        .trim()
        .to_string()
        + "\n"
}

fn run_llvm_as(file_path: &str) -> Result<ExitStatus, std::io::Error> {
    Command::new("llvm-as").arg(file_path).status()
}

fn run_cargo_fmt() -> Result<ExitStatus, std::io::Error> {
    Command::new("cargo").arg("fmt").status()
}
