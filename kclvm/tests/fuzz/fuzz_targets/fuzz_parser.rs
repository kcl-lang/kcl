#![no_main]
use kclvm_parser::parse_expr;
use kclvm_runtime::PanicInfo;
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use serde_json;
use std::panic::catch_unwind;
use std::panic::set_hook;

#[derive(Arbitrary, Debug)]
enum ParserMethod<'a> {
    ParseExpr { src: &'a str },
}

fuzz_target!(|method: ParserMethod| {
    // fuzzed code goes here
    match method {
        ParserMethod::ParseExpr { src } => {
            set_hook(Box::new(|_info| {}));
            let result = catch_unwind(|| {
                parse_expr(src);
            });
            match result {
                Err(e) => match e.downcast::<String>() {
                    Ok(_v) => {
                        let got = _v.to_string();
                        let _u: PanicInfo = serde_json::from_str(&got).unwrap();
                    }
                    _ => unreachable!(),
                },
                _ => {}
            };
        }
    }
});
