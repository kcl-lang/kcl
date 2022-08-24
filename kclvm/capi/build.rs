/// According to the file KCLVM/internal/kclvm_py/spec/gpyrpc/gpyrpc.proto, automatically generate
/// the corresponding rust source file to the directory src/model
fn main() {
    protobuf_codegen::Codegen::new()
        .protoc()
        .protoc_path(&protoc_bin_vendored::protoc_bin_path().unwrap())
        .out_dir("src/model")
        .include("../../internal/kclvm_py/spec/gpyrpc")
        .inputs(&["../../internal/kclvm_py/spec/gpyrpc/gpyrpc.proto"])
        .run()
        .expect("Running protoc failed.");
}
