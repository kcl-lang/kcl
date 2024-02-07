use std::fs;

use crate::*;

// https://docs.python.org/3/library/math.html
// https://doc.rust-lang.org/std/primitive.f64.html
// https://github.com/RustPython/RustPython/blob/main/stdlib/src/math.rs

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_file_read(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(x) = args.arg_i_str(0, None) {
        let contents =
            fs::read_to_string(&x).expect(&format!("failed to access the file in {}", x));

        let s = ValueRef::str(contents.as_ref());
        return s.into_raw(ctx);
    }

    panic!("read() takes exactly one argument (0 given)");
}
