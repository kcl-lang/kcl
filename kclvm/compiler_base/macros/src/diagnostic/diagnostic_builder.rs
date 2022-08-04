use compiler_base_diagnostic::{DiagnosticBuilder, ErrHandler};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, Ident, Meta, MetaList, MetaNameValue,
    NestedMeta, Path,
};
use synstructure::{BindingInfo, Structure};

use super::error::*;

// generate the implementation code for trait 'DiagnosticBuilder'
pub fn diagnostic_builder_derive(s: Structure<'_>) -> TokenStream {
    DiagnosticBuilderGenerator::new().generate_diagnostic_builder(s)
}
struct DiagnosticBuilderGenerator {
    err_handler: ErrHandler,
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
            err_handler: ErrHandler::new(),
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

    pub fn generate_diagnostic_builder(&mut self, s: Structure<'_>) -> TokenStream {
        let ast = s.ast();
        let attrs = &ast.attrs;
        if let syn::Data::Struct(..) = ast.data {
            for (index, attr) in attrs.iter().enumerate() {
                self.generate_struct_attr_code(attr, index)
            }
        } else {
            self.err_handler.emit_err(UnexpectedDiagnosticType::new());
        };

        let undep_sentence = s.each(|field_binding| self.generate_field_attrs_code(field_binding));
        self.body_tokens.push(undep_sentence);

        self.into_tokens(s)
    }

    pub fn generate_struct_attr_code(&mut self, attr: &Attribute, index: usize) {
        let kind = attr.path.segments.last().unwrap().ident.to_string();
        let kind = kind.as_str();
        let meta = attr.parse_meta().unwrap();

        let nested = match meta {
            Meta::List(MetaList { ref nested, .. }) => nested,
            _ => {
                self.err_handler.emit_err(InternalBug::new());
                return;
            }
        };

        self.generate_undep_sentence_code(kind.to_string(), &nested, index)
    }

    // generate code that does not depend on struct fields.
    pub fn generate_undep_sentence_code(
        &mut self,
        kind: String,
        nested_attrs: &Punctuated<NestedMeta, Comma>,
        index: usize,
    ) {
        let pendant_ident = format_ident!("pendant_{}", index);
        let sentence_ident = format_ident!("pendant_{}", index);
        let mut sentence_info = SentenceInfo::new(kind.to_string(), pendant_ident, sentence_ident);

        self.get_sentence_info_from_nested_attr(nested_attrs, &mut sentence_info);

        let sentence = match kind.as_str() {
            "error" | "warning" | "help" | "note" => {
                SentenceTokensFactory::gen_header_pendant_sentence_tokens(&sentence_info)
                    .unwrap_or_else(|err| {
                        self.err_handler.emit_err(err);
                        TokenStream::new()
                    })
            }
            "nopendant" => SentenceTokensFactory::gen_nopendant_sentence_tokens(&sentence_info)
                .unwrap_or_else(|err| {
                    self.err_handler.emit_err(err);
                    TokenStream::new()
                }),
            // TODO(zongz): before adding more macros, delete it first.
            _ => {
                self.err_handler
                    .emit_err(UnexpectedLabel::new(kind.to_string()));
                TokenStream::new()
            }
        };

        if sentence_info.is_title {
            self.head_tokens.push(sentence);
        } else {
            self.tail_tokens.push(sentence);
        }
    }

    pub fn generate_field_attrs_code(&mut self, binding_info: &BindingInfo<'_>) -> TokenStream {
        let field = binding_info.ast();
        if !field.attrs.is_empty() {
            for attr in &field.attrs {
                let meta = attr.parse_meta().unwrap();
                let nested = match meta {
                    Meta::List(MetaList { ref nested, .. }) => nested,
                    _ => {
                        self.err_handler.emit_err(InternalBug::new());
                        return TokenStream::new();
                    }
                };

                let kind = attr.path.segments.last().unwrap().ident.to_string();
                let kind = kind.as_str();

                match kind {
                    "position" => {
                        return self.generate_dep_code_sentence(
                            field.ident.as_ref().unwrap(),
                            &nested,
                            0,
                        );
                    }
                    _ => self
                        .err_handler
                        .emit_err(UnexpectedLabel::new(kind.to_string())),
                }
            }
        }
        TokenStream::new()
    }

    // generate code that depends on struct fields.
    pub fn generate_dep_code_sentence(
        &mut self,
        field_name: &Ident,
        nested_attrs: &Punctuated<NestedMeta, Comma>,
        index: usize,
    ) -> TokenStream {
        let code_ctx_pendant_ident = format_ident!("codectx_pendant_{}", index);
        let code_ctx_sentence_ident = format_ident!("codectx_sentence_{}", index);
        let mut sentence_info = SentenceInfo::new(
            "position".to_string(),
            code_ctx_pendant_ident,
            code_ctx_sentence_ident,
        );

        self.get_sentence_info_from_nested_attr(nested_attrs, &mut sentence_info);

        SentenceTokensFactory::gen_code_ctx_pendant_sentence_tokens(field_name, &sentence_info)
            .unwrap_or_else(|err| {
                self.err_handler.emit_err(err);
                TokenStream::new()
            })
    }

    fn get_sentence_info_from_nested_attr(
        &mut self,
        nested_attrs: &Punctuated<NestedMeta, Comma>,
        sentence_info: &mut SentenceInfo,
    ) {
        for nested_attr in nested_attrs {
            if let NestedMeta::Meta(meta @ Meta::NameValue(_)) = nested_attr {
                if let Meta::NameValue(MetaNameValue {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &meta
                {
                    if meta.path().segments.last().unwrap().ident.to_string() == "msg" {
                        // Sub-attribute 'msg' is only allowed once.
                        sentence_info
                            .set_msg_only_once(s.value())
                            .unwrap_or_else(|err| {
                                self.err_handler.emit_err(err);
                                false
                            });
                    } else if meta.path().segments.last().unwrap().ident.to_string() == "code" {
                        // Sub-attribute 'code' is only allowed once.
                        sentence_info
                            .set_code_only_once(s.value())
                            .unwrap_or_else(|err| {
                                self.err_handler.emit_err(err);
                                false
                            });
                    } else {
                        self.err_handler.emit_err(UnexpectedAttr::new(
                            sentence_info.label_name.to_string(),
                            meta.path().segments.last().unwrap().ident.to_string(),
                        ))
                    }
                }
            } else if let NestedMeta::Meta(meta @ Meta::Path(_)) = nested_attr {
                if let Meta::Path(Path { segments, .. }) = &meta {
                    if segments.last().unwrap().ident.to_string() == "title" {
                        // Sub-attribute 'title' is only allowed once.
                        sentence_info
                            .set_is_title_true_only_once()
                            .unwrap_or_else(|err| {
                                self.err_handler.emit_err(err);
                                false
                            });
                    } else {
                        self.err_handler.emit_err(UnexpectedAttr::new(
                            sentence_info.label_name.to_string(),
                            segments.last().unwrap().ident.to_string(),
                        ))
                    }
                }
            }
        }
        sentence_info.msg_is_required().unwrap_or_else(|err| {
            self.err_handler.emit_err(err);
            false
        });
        return;
    }
}

struct SentenceInfo {
    pendant_ident: Ident,
    sentence_ident: Ident,
    label_name: String,
    msg: Option<String>,
    code: Option<String>,
    is_title: bool,
}

impl SentenceInfo {
    pub fn new(label_name: String, pendant_ident: Ident, sentence_ident: Ident) -> Self {
        Self {
            pendant_ident,
            sentence_ident,
            label_name,
            msg: None,
            code: None,
            is_title: false,
        }
    }

    pub fn set_msg_only_once(&mut self, msg: String) -> Result<bool, impl DiagnosticBuilder> {
        if let Some(_) = self.msg {
            self.throw_duplicate_attr_err("msg".to_string())
        } else {
            self.msg = Some(msg);
            Ok(true)
        }
    }

    pub fn set_code_only_once(&mut self, code: String) -> Result<bool, impl DiagnosticBuilder> {
        if let Some(_) = self.code {
            self.throw_duplicate_attr_err("code".to_string())
        } else {
            self.code = Some(code);
            Ok(true)
        }
    }

    pub fn set_is_title_true_only_once(&mut self) -> Result<bool, impl DiagnosticBuilder> {
        if self.is_title {
            self.throw_duplicate_attr_err("title".to_string())
        } else {
            self.is_title = true;
            Ok(true)
        }
    }

    pub fn msg_is_required(&self) -> Result<bool, impl DiagnosticBuilder> {
        if let None = self.msg {
            Err(MissingAttr::new(
                self.label_name.to_string(),
                "msg".to_string(),
            ))
        } else {
            Ok(true)
        }
    }

    pub fn throw_duplicate_attr_err(
        &mut self,
        attr_name: String,
    ) -> Result<bool, impl DiagnosticBuilder> {
        Err(DuplicateAttr::new(attr_name, self.label_name.clone()))
    }
}

struct SentenceTokensFactory;

impl SentenceTokensFactory {
    pub fn gen_header_pendant_sentence_tokens(
        sentence_info: &SentenceInfo,
    ) -> Result<TokenStream, impl DiagnosticBuilder> {
        let pendant_ident = &sentence_info.pendant_ident;
        let sentence_ident = &sentence_info.sentence_ident;
        let code = match &sentence_info.code {
            Some(s) => quote! {Some(#s.to_string())},
            None => quote! {None},
        };

        // sub-attribute 'msg' is required.
        let msg_is_required = sentence_info.msg_is_required();

        match msg_is_required {
            Ok(_) => {
                let msg = sentence_info.msg.as_ref().unwrap();
                let label_name = &sentence_info.label_name;
                Ok(quote! {
                    let #pendant_ident = HeaderPendant::new(#label_name.to_string(), #code);
                    let #sentence_ident = Sentence::new_sentence_str(
                        Box::new(#pendant_ident),
                        Message::Str(#msg.to_string())
                    );
                    diagnostic.add_sentence(#sentence_ident);
                })
            }
            Err(err) => Err(err),
        }
    }
   
    pub fn gen_nopendant_sentence_tokens(
        sentence_info: &SentenceInfo,
    ) -> Result<TokenStream, impl DiagnosticBuilder> {
        let pendant_ident = &sentence_info.pendant_ident;
        let sentence_ident = &sentence_info.sentence_ident;

        let msg_is_required = sentence_info.msg_is_required();

        match msg_is_required {
            Ok(_) => {
                let msg = sentence_info.msg.as_ref().unwrap();

                Ok(quote! {
                    let #pendant_ident = NoPendant::new();
                    let #sentence_ident = Sentence::new_sentence_str(
                        Box::new(#pendant_ident),
                        Message::Str(#msg.to_string())
                    );
                    diagnostic.add_sentence(#sentence_ident);
                })
            }
            Err(err) => Err(err),
        }
    }

    pub fn gen_code_ctx_pendant_sentence_tokens(
        code_pos_field_name: &Ident,
        sentence_info: &SentenceInfo,
    ) -> Result<TokenStream, impl DiagnosticBuilder> {
        let pendant_ident = &sentence_info.pendant_ident;
        let sentence_ident = &sentence_info.sentence_ident;

        let msg_is_required = sentence_info.msg_is_required();

        match msg_is_required {
            Ok(_) => {
                let msg = sentence_info.msg.as_ref().unwrap();
                Ok(quote! {
                    let #pendant_ident = CodeCtxPendant::new(self.#code_pos_field_name.clone());
                    let #sentence_ident = Sentence::new_sentence_str(
                        Box::new(#pendant_ident),
                        Message::Str(#msg.to_string())
                    );
                    diagnostic.add_sentence(#sentence_ident);
                })
            }
            Err(err) => Err(err),
        }
    }
}
