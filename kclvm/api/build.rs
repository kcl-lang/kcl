use std::{env, path::PathBuf};

use prost_wkt_build::{FileDescriptorSet, Message};

/// According to the file kclvm/spec/gpyrpc/gpyrpc.proto, automatically generate
/// the corresponding rust source file to the directory src/model
fn main() {
    std::env::set_var(
        "PROTOC",
        protoc_bin_vendored::protoc_bin_path().unwrap().as_os_str(),
    );

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_file = out.join("kclvm_service_descriptor.bin");

    let mut prost_build = prost_build::Config::new();
    prost_build
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .field_attribute(".", "#[serde(default)]")
        .extern_path(".google.protobuf.Any", "::prost_wkt_types::Any")
        .extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
        .extern_path(".google.protobuf.Value", "::prost_wkt_types::Value")
        .file_descriptor_set_path(&descriptor_file)
        .compile_protos(&["../spec/gpyrpc/gpyrpc.proto"], &["../spec/gpyrpc/"])
        .expect("Running prost build failed.");

    let descriptor_bytes = std::fs::read(descriptor_file).unwrap();

    let descriptor = FileDescriptorSet::decode(&descriptor_bytes[..]).unwrap();

    prost_wkt_build::add_serde(out, descriptor);
}
