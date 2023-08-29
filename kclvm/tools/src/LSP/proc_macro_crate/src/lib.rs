use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn bench_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;

    let timing_code = quote! {
        {
            let start_time = std::time::Instant::now();
            let result = #fn_body;
            let end_time = std::time::Instant::now();
            let time =  (end_time - start_time).as_micros();
            println!("{} took {} μs", stringify!(#fn_name), (end_time - start_time).as_micros());
            // 200 ms
            assert!(time < 200000, "Bench mark test failed");
            result
        }
    };

    input_fn.block = Box::new(syn::parse2(timing_code).unwrap());

    let output = quote! {
        #input_fn
    };

    output.into()
}
