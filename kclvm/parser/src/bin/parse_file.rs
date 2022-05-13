extern crate kclvm_parser;

fn main() {
    let filename = std::env::args().nth(1).expect("filename missing");
    let m = kclvm_parser::parse_file(filename.as_str(), None);
    let json = serde_json::ser::to_string(&m).unwrap();
    println!("{}", json);
}
