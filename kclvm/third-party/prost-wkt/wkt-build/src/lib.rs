use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use quote::{format_ident, quote};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub use prost::Message;
pub use prost_types::FileDescriptorSet;

use prost_build::Module;

pub fn add_serde(out: PathBuf, descriptor: FileDescriptorSet) {
    for fd in &descriptor.file {
        let package_name = match fd.package {
            Some(ref pkg) => pkg,
            None => continue,
        };

        let rust_path = out
            .join(Module::from_protobuf_package_name(package_name).to_file_name_or(package_name));

        // In some cases the generated file would be in empty. These files are no longer created by Prost, so
        // we'll create here. Otherwise we append.
        let mut rust_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(rust_path)
            .unwrap();

        for msg in &fd.message_type {
            let message_name = match msg.name {
                Some(ref name) => name,
                None => continue,
            };

            let type_url = format!("type.googleapis.com/{package_name}.{message_name}");

            gen_trait_impl(&mut rust_file, package_name, message_name, &type_url);
        }
    }
}

// This method uses the `heck` crate (the same that prost uses) to properly format the message name
// to UpperCamelCase as the prost_build::ident::{to_snake, to_upper_camel} methods
// in the `ident` module of prost_build is private.
fn gen_trait_impl(rust_file: &mut File, package_name: &str, message_name: &str, type_url: &str) {
    let type_name = message_name.to_upper_camel_case();
    let type_name = format_ident!("{}", type_name);

    let dummy_const = format_ident!(
        "IMPL_MESSAGE_SERDE_FOR_{}",
        message_name.to_shouty_snake_case()
    );

    let tokens = quote! {
        #[allow(dead_code)]
        const #dummy_const: () = {
            use ::prost_wkt::typetag;
            #[typetag::serde(name=#type_url)]
            impl ::prost_wkt::MessageSerde for #type_name {
                fn package_name(&self) -> &'static str {
                    #package_name
                }
                fn message_name(&self) -> &'static str {
                    #message_name
                }
                fn type_url(&self) -> &'static str {
                    #type_url
                }
                fn new_instance(&self, data: Vec<u8>) -> Result<Box<dyn ::prost_wkt::MessageSerde>, ::prost::DecodeError> {
                    let mut target = Self::default();
                    ::prost::Message::merge(&mut target, data.as_slice())?;
                    let erased: Box<dyn ::prost_wkt::MessageSerde> = Box::new(target);
                    Ok(erased)
                }
                fn try_encoded(&self) -> Result<Vec<u8>, ::prost::EncodeError> {
                    let mut buf = Vec::new();
                    buf.reserve(::prost::Message::encoded_len(self));
                    ::prost::Message::encode(self, &mut buf)?;
                    Ok(buf)
                }
            }

            ::prost_wkt::inventory::submit!{
                ::prost_wkt::MessageSerdeDecoderEntry {
                    type_url: #type_url,
                    decoder: |buf: &[u8]| {
                        let msg: #type_name = ::prost::Message::decode(buf)?;
                        Ok(Box::new(msg))
                    }
                }
            }
        };
    };

    writeln!(rust_file).unwrap();
    writeln!(rust_file, "{}", &tokens).unwrap();
}
