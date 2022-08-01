use core::panic;

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{punctuated::Iter, Attribute, Ident, Meta, MetaList, MetaNameValue, NestedMeta, Path};
use synstructure::{BindingInfo, Structure};

pub fn diagnostic_derive(s: Structure<'_>) -> TokenStream {
    let result = DiagnosticBuilderGenerator::new().generate_diagnostic_builder(s);
    println!("TOFIX(zongz) - result {}", result);
    result
}

struct DiagnosticBuilderGenerator {
    head_tokens: Vec<TokenStream>,
    tail_tokens: Vec<TokenStream>,
    body_tokens: Vec<TokenStream>,
}

impl DiagnosticBuilderGenerator {
    pub fn new() -> Self {
        Self {
            head_tokens: vec![],
            tail_tokens: vec![],
            body_tokens: vec![],
        }
    }

    pub fn into_tokens(&mut self, s: Structure<'_>) -> TokenStream {
        let head: TokenStream = self.head_tokens.drain(..).collect();
        let tail: TokenStream = self.tail_tokens.drain(..).collect();
        let body: TokenStream = self.body_tokens.drain(..).collect();

        s.gen_impl(quote! {
            gen impl DiagnosticBuilder
                    for @Self
            {
                fn into_diagnostic(
                    self
                ) -> Diagnostic {
                    let mut diagnostic = Diagnostic::new();
                    # head
                    match self {
                        # body
                    }
                    # tail
                    diagnostic
                }
            }
        })
    }

    pub fn generate_diagnostic_builder(&mut self, s: Structure<'_>) -> TokenStream{
        let ast = s.ast();
        let attrs = &ast.attrs;
        if let syn::Data::Struct(..) = ast.data {
            for (index, attr) in attrs.iter().enumerate(){
                self.generate_struct_attr_code(attr, index)
            }
        } else {
            panic!("struct attr to code failed");
        };

        let undep_sentence = s.each(|field_binding| self.generate_field_attrs_code(field_binding));
        self.body_tokens.push(undep_sentence);

        self.into_tokens(s)
    }

    pub fn generate_struct_attr_code(&mut self, attr: &Attribute, index: usize){
        let kind = attr.path.segments.last().unwrap().ident.to_string();
        let kind = kind.as_str();
        let meta = attr.parse_meta().unwrap();

        let nested = match meta {
            Meta::List(MetaList { ref nested, .. }) => nested,
            _ => panic!("nested struct attr err"),
        };
        let mut nested_iter = nested.into_iter();

        match kind {
            "error" | "warning" | "help" | "nopendant" | "note" => {
                self.generate_undep_sentence_code(kind.to_string(), &mut nested_iter, index)
            }
            // TODO(zongz): before adding more macros, delete it first.
            _ => panic!("the attr is not supported"),
        }
    }

    pub fn generate_undep_sentence_code(
        &mut self,
        kind: String,
        nested_iter: &mut Iter<NestedMeta>,
        index: usize,
    ){
        let msg = get_str_value_nested_attr(nested_iter, "msg".to_string());
        let code = get_str_value_nested_attr(nested_iter, "code".to_string());

        if let None = msg {
            panic!("msg can not be None");
        }

        let is_title = is_title(nested_iter);

        let pendant_ident = format_ident!("pendant_{}", index);
        let sentence_ident = format_ident!("pendant_{}", index);
        let sentence = quote! {
            let #pendant_ident = HeaderPendant::new(#kind, #code);
            let #sentence_ident = Sentence::new_sentence_str(
                Box::new(#pendant_ident),
                Message::Str(#msg.to_string())
            );
            diagnostic.add_sentence(#sentence_ident);
        };

        if is_title {
            self.head_tokens.push(sentence);
        } else {
            self.tail_tokens.push(sentence);
        }
    }

    pub fn generate_field_attrs_code(&mut self, binding_info: &BindingInfo<'_>) -> TokenStream {
        let field = binding_info.ast();
        if field.attrs.is_empty() {
            panic!("there is no field attrs");
        } else {
            // FIXME(zongz): field type check here !
            for attr in &field.attrs {
                let meta = attr.parse_meta().unwrap();
                let nested = match meta {
                    Meta::List(MetaList { ref nested, .. }) => nested,
                    _ => panic!("Internal Bugs"),
                };
                let mut nested_iter = nested.into_iter();

                let name = attr.path.segments.last().unwrap().ident.to_string();
                let name = name.as_str();

                match name {
                    "position" => {
                        return self.generate_ctx_code(
                            field.ident.as_ref().unwrap(),
                            &mut nested_iter,
                            0,
                        );
                    }
                    _ => println!("Internal Bug"),
                }
            }
        }
        TokenStream::new()
    }

    pub fn generate_ctx_code(
        &self,
        field_name: &Ident,
        nested_iter: &mut Iter<NestedMeta>,
        index: usize,
    ) -> TokenStream {
        let msg = get_str_value_nested_attr(nested_iter, "msg".to_string());

        if let None = msg {
            panic!("msg can not be None");
        }
        
        let code_ctx_pendant_ident = format_ident!("codectx_pendant_{}", index);
        let code_ctx_sentence_ident = format_ident!("codectx_sentence_{}", index);

        quote! {
            let #code_ctx_pendant_ident = CodeCtxPendant::new(self.#field_name.clone());
            let #code_ctx_sentence_ident = Sentence::new_sentence_str(
                Box::new(#code_ctx_pendant_ident),
                Message::Str(#msg.to_string())
            );
            diagnostic.add_sentence(#code_ctx_sentence_ident);
        }
    }
}

fn is_title(nested_iter: &mut Iter<NestedMeta>) -> bool {
    while let Some(nested_attr) = nested_iter.next() {
        if let NestedMeta::Meta(meta @ Meta::Path(_)) = nested_attr {
            if let Meta::Path(Path { segments, .. }) = &meta {
                return segments.last().unwrap().ident.to_string() == "title";
            } else {
                return false;
            }
        }
    }
    false
}

fn get_str_value_nested_attr(nested_iter: &mut Iter<NestedMeta>, key: String) -> Option<String> {
    while let Some(nested_attr) = nested_iter.next() {
        if let NestedMeta::Meta(meta @ Meta::NameValue(_)) = nested_attr {
            if let Meta::NameValue(MetaNameValue {
                lit: syn::Lit::Str(s),
                ..
            }) = &meta
            {
                if meta.path().segments.last().unwrap().ident.to_string() == key {
                    return Some(s.value());
                }
            } else {
                return None;
            }
        }
    }
    None
}
