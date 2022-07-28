use proc_macro2::{TokenStream};
use quote::quote;
use syn::{
    punctuated::{Iter},
    Attribute, Meta, MetaList, MetaNameValue, NestedMeta, Ident,
};
use synstructure::{Structure, BindingInfo};


pub fn diagnostic_derive(s: Structure<'_>) -> TokenStream {

    let structure = s;
    let ast = structure.ast();
    let attrs = &ast.attrs;
    let b = if let syn::Data::Struct(..) = ast.data {
         let _preamble = {
            let preamble = attrs.iter().map(|attr| {
                generate_struct_attr(attr)
            });

            quote! {
                #(#preamble)*
            }
        };
        _preamble
    }else{
        panic!();
    };

    let s = structure;
    let a = s.each(|field_binding| {
        generate_field_attrs_code(field_binding)
    });

    
    
    let result = s.gen_impl(quote! {
        gen impl DiagnosticBuilder
                for @Self
        {
            fn into_diagnostic(
                self
            ) -> Diagnostic {
                let mut diagnostic = Diagnostic::new();
                match self {
                    #a
                }
                #b
                diagnostic.add_sentence(title_sentence);
                // diagnostic.add_sentence(codectx_sentence);
                diagnostic.add_sentence(label_sentence);

                diagnostic
            }
        }
    });
    println!("result {}", result);
    result
}

pub fn generate_struct_attr(attr: &Attribute) -> TokenStream {
    let name = attr.path.segments.last().unwrap().ident.to_string();
    let name = name.as_str();
    let meta = attr.parse_meta().unwrap();
    let nested = match meta {
        Meta::List(MetaList { ref nested, .. }) => nested,
        _ => panic!("Internal Bugs"),
    };
    let mut nested_iter = nested.into_iter();
    match name {
        "title" => generate_title(&mut nested_iter),
        "note" => generate_note(&mut nested_iter),
        _ => panic!("Internal Bug"),
    }
}

/// 这里可以generate_struct_code 里面加两个for循环。
pub fn generate_title(nested_iter: &mut Iter<NestedMeta>) -> TokenStream {
    let mut kind = String::new();
    let mut msg = String::new();
    let mut code = String::new();

    while let Some(nested_attr) = nested_iter.next() {
        if let NestedMeta::Meta(meta @ Meta::NameValue(_)) = nested_attr {
            if let Meta::NameValue(MetaNameValue {
                lit: syn::Lit::Str(s),
                ..
            }) = &meta
            {
                if meta.path().segments.last().unwrap().ident.to_string() == "kind" {
                    kind = s.value();
                } else if meta.path().segments.last().unwrap().ident.to_string() == "msg" {
                    msg = s.value();
                } else if meta.path().segments.last().unwrap().ident.to_string() == "code" {
                    code = s.value();
                }else{
                    panic!("Internal Bugs"); 
                }
            } else {
                panic!("Internal Bugs");
            }
        }
    }

    quote! {
        let title_pendant = HeaderPendant::new(Level::Error, #code.to_string());
        let title_sentence = Sentence::new_sentence_str(
            Box::new(title_pendant),
            Message::Str(#msg.to_string())
        );
    }
}

pub fn generate_note(nested_iter: &mut Iter<NestedMeta>) -> TokenStream {
    let mut label = String::new();
    let mut msg = String::new();

    while let Some(nested_attr) = nested_iter.next() {
        if let NestedMeta::Meta(meta @ Meta::NameValue(_)) = nested_attr {
            if let Meta::NameValue(MetaNameValue {
                lit: syn::Lit::Str(s),
                ..
            }) = &meta
            {
                if meta.path().segments.last().unwrap().ident.to_string() == "label" {
                    label = s.value();
                } else if meta.path().segments.last().unwrap().ident.to_string() == "msg" {
                    msg = s.value();
                } else{
                    panic!("Internal Bugs");
                }
            } else {
                panic!("Internal Bugs");
            }
        }
    }

    quote! {
        let label_pendant = LabelPendant::new(#label.to_string());
        let label_sentence = Sentence::new_sentence_str(
            Box::new(label_pendant),
            Message::Str(#msg.to_string())
        );
    }
}

pub fn generate_position(field_name: &Ident, nested_iter: &mut Iter<NestedMeta>) -> TokenStream{
    let mut msg = String::new();

    while let Some(nested_attr) = nested_iter.next() {
        if let NestedMeta::Meta(meta @ Meta::NameValue(_)) = nested_attr {
            if let Meta::NameValue(MetaNameValue {
                lit: syn::Lit::Str(s),
                ..
            }) = &meta
            {
                if meta.path().segments.last().unwrap().ident.to_string() == "msg" {
                    msg = s.value();
                } else{
                    panic!("Internal Bugs");
                }
            } else {
                panic!("Internal Bugs");
            }
        }
    }

    quote! {
        let codectx_pendant = CodeCtxPendant::new(self.#field_name.clone());
        let codectx_sentence = Sentence::new_sentence_str(
            Box::new(codectx_pendant),
            Message::Str(#msg.to_string())
        );
        diagnostic.add_sentence(codectx_sentence);
    }
}

pub fn generate_field_attrs_code(binding_info: &BindingInfo<'_>) -> TokenStream {   
    let field = binding_info.ast();
    if field.attrs.is_empty() {
        panic!("internal bugs");
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

            match name{
                "position" => {
                    let ident = field.ident.as_ref().unwrap();
                    return generate_position(ident, &mut nested_iter);
                },
                _ => {
                    println!("Internal Bug");
                }
            }
        }
    }
    TokenStream::new()
}