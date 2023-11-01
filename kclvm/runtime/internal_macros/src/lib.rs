use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg};

// ----------------------------------------------------------------------------

#[proc_macro_attribute]
pub fn runtime_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_fn = parse_macro_input!(item as syn::ItemFn);

    if std::env::var("KCLVM_RUNTIME_GEN_API_SPEC").is_ok() {
        print_api_spec(&parsed_fn);
    }

    let x = quote! {
        #parsed_fn
    };
    x.into()
}

// ----------------------------------------------------------------------------
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
enum TargetName {
    C,
    LLVM,
}

fn print_api_spec(fn_item: &syn::ItemFn) {
    let fn_name = get_fn_name(fn_item);
    let fn_c_sig = get_fn_sig(fn_item, &TargetName::C);
    let fn_llvm_sig = get_fn_sig(fn_item, &TargetName::LLVM);

    // skip _fn_name()
    if !fn_name.starts_with('_') {
        println!("// api-spec:       {}", fn_name);
        println!("// api-spec(c):    {};", fn_c_sig);
        println!("// api-spec(llvm): {};", fn_llvm_sig);
        println!();
    }
}

// ----------------------------------------------------------------------------

fn get_fn_name(fn_item: &syn::ItemFn) -> String {
    fn_item.sig.ident.to_string()
}

fn get_fn_sig(fn_item: &syn::ItemFn, target: &TargetName) -> String {
    let fn_name = get_fn_name(fn_item);
    let args_type = get_fn_args_type(fn_item, target);
    let output_type = get_fn_output_type(fn_item, target);

    match target {
        TargetName::C => format!("{} {}({})", output_type, fn_name, args_type),
        TargetName::LLVM => format!("declare {} @{}({})", output_type, fn_name, args_type),
    }
}

fn get_fn_output_type(fn_item: &syn::ItemFn, target: &TargetName) -> String {
    match target {
        TargetName::C => match &fn_item.sig.output {
            syn::ReturnType::Type(_, ty) => build_c_type(ty),
            syn::ReturnType::Default => "void".to_string(),
        },
        TargetName::LLVM => match &fn_item.sig.output {
            syn::ReturnType::Type(_, ty) => build_llvm_type(ty),
            syn::ReturnType::Default => "void".to_string(),
        },
    }
}

// ----------------------------------------------------------------------------

fn get_fn_args_type(fn_item: &syn::ItemFn, target: &TargetName) -> String {
    let fn_name = get_fn_name(fn_item);
    let inputs = &fn_item.sig.inputs;

    let mut result = String::new();
    for (i, arg) in inputs.iter().enumerate() {
        let arg_name = get_fn_arg_name(arg, target);
        let arg_typ = get_fn_arg_type(arg, target);

        if arg_name.is_empty() {
            panic!("{}", format!("{}, arg{}: invalid arg type", fn_name, i));
        }
        if arg_typ.is_empty() {
            panic!("{}", format!("{}, arg{}: invalid arg type", fn_name, i));
        }

        if i > 0 {
            result.push_str(", ");
        }
        result.push_str(format!("{} {}", arg_typ, arg_name).as_str());
    }

    result
}

// ----------------------------------------------------------------------------

fn get_fn_arg_name(arg: &FnArg, target: &TargetName) -> String {
    match arg {
        syn::FnArg::Typed(ty) => {
            let syn::PatType { pat, .. } = ty;
            match &**pat {
                syn::Pat::Ident(x) => match target {
                    TargetName::C => x.ident.to_string(),
                    TargetName::LLVM => format!("%{}", x.ident),
                },
                _ => panic!("unsupported type: {}", quote!(#ty)),
            }
        }
        _ => panic!("unsupported arg: {}", quote!(#arg)),
    }
}

fn get_fn_arg_type(arg: &FnArg, target: &TargetName) -> String {
    match arg {
        syn::FnArg::Typed(ty) => {
            let syn::PatType { ty, .. } = ty;
            match target {
                TargetName::C => build_c_type(ty),
                TargetName::LLVM => build_llvm_type(ty),
            }
        }
        _ => panic!("unsupported fn arg: {}", quote!(#arg)),
    }
}

// ----------------------------------------------------------------------------

fn build_c_type(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(ty_path) => {
            let ty_name = ty_path.path.segments[0].ident.to_string();

            match ty_name.as_str() {
                "c_void" => "void".to_string(),
                "c_char" => "char".to_string(),

                "bool" => "uint8_t".to_string(),

                "i8" => "int8_t".to_string(),
                "u8" => "uint8_t".to_string(),

                "i16" => "int16_t".to_string(),
                "u16" => "uint16_t".to_string(),

                "i32" => "int32_t".to_string(),
                "u32" => "uint32_t".to_string(),

                "i64" => "int64_t".to_string(),
                "u64" => "uint64_t".to_string(),
                "i128" | "u128" => "".to_string(),

                "f32" => "float".to_string(),
                "f64" => "double".to_string(),

                _ => ty_name,
            }
        }
        syn::Type::Ptr(ty_ptr) => {
            let base_ty = &ty_ptr.elem;
            let base_constr = build_c_type(base_ty);
            format!("{}*", base_constr)
        }
        syn::Type::Reference(ty_ref) => {
            let base_ty = &ty_ref.elem;
            let base_constr = build_c_type(base_ty);
            format!("{}*", base_constr)
        }
        syn::Type::BareFn(_) => "void*".to_string(),
        syn::Type::Never(_) => "void".to_string(),
        _ => panic!("unsupported type: {}", quote!(#ty)),
    }
}

fn build_llvm_type(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(ty_path) => {
            let ty_name = ty_path.path.segments[0].ident.to_string();

            match ty_name.as_str() {
                "c_void" => "void".to_string(),
                "c_char" => "i8".to_string(),

                "bool" => "i8".to_string(),

                "i8" | "u8" => "i8".to_string(),
                "i16" | "u16" => "i16".to_string(),
                "i32" | "u32" => "i32".to_string(),
                "i64" | "u64" => "i64".to_string(),
                "i128" | "u128" => "i128".to_string(),

                "f32" => "float".to_string(),
                "f64" => "double".to_string(),

                _ => format!("%{}", ty_name),
            }
        }
        syn::Type::Ptr(ty_ptr) => {
            let base_ty = &ty_ptr.elem;
            let base_constr = build_llvm_type(base_ty);
            format!("{}*", base_constr)
        }
        syn::Type::Reference(ty_ref) => {
            let base_ty = &ty_ref.elem;
            let base_constr = build_llvm_type(base_ty);
            format!("{}*", base_constr)
        }
        syn::Type::BareFn(_) => "i8*".to_string(),
        syn::Type::Never(_) => "void".to_string(),
        _ => panic!("unsupported type: {}", quote!(#ty)),
    }
}

// ----------------------------------------------------------------------------
// END
// ----------------------------------------------------------------------------
