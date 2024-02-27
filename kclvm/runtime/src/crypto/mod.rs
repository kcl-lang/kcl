//! Copyright The KCL Authors. All rights reserved.

extern crate md5;
extern crate sha1;
extern crate sha2;

use core::panic;
use std::{fs::File, io::Read};

use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};

use crate::*;
use uuid::Uuid;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

// md5(value: str, encoding: str = "utf-8") -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_crypto_md5(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(s) = args.arg_i_str(0, None) {
        let hex = format!("{:x}", md5::compute(s));
        return ValueRef::str(hex.as_ref()).into_raw(ctx);
    }
    panic!("md5() missing 1 required positional argument: 'value'");
}

// sha1(value: str, encoding: str = "utf-8") -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_crypto_sha1(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(s) = args.arg_i_str(0, None) {
        let hex = sha1::Sha1::from(s).digest().to_string();
        return ValueRef::str(hex.as_ref()).into_raw(ctx);
    }
    panic!("sha1() missing 1 required positional argument: 'value'");
}

// sha224(value: str, encoding: str = "utf-8") -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_crypto_sha224(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(s) = args.arg_i_str(0, None) {
        let mut hasher = Sha224::new();
        hasher.update(&s);
        let result = hasher.finalize();

        let mut hex = String::with_capacity(2 * Sha256::output_size());
        use std::fmt::Write;

        for byte in result {
            let _ = write!(&mut hex, "{byte:02x}");
        }

        return ValueRef::str(hex.as_ref()).into_raw(ctx);
    }
    panic!("sha224() missing 1 required positional argument: 'value'");
}

// sha256(value: str, encoding: str = "utf-8") -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_crypto_sha256(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);
    if let Some(s) = args.arg_i_str(0, None) {
        let mut hasher = Sha256::new();
        hasher.update(&s);
        let result = hasher.finalize();

        let mut hex = String::with_capacity(2 * Sha256::output_size());
        use std::fmt::Write;

        for byte in result {
            let _ = write!(&mut hex, "{byte:02x}");
        }

        return ValueRef::str(hex.as_ref()).into_raw(ctx);
    }
    panic!("sha256() missing 1 required positional argument: 'value'");
}

// sha384(value: str, encoding: str = "utf-8") -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_crypto_sha384(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(s) = args.arg_i_str(0, None) {
        let mut hasher = Sha384::new();
        hasher.update(&s);
        let result = hasher.finalize();

        let mut hex = String::with_capacity(2 * Sha256::output_size());
        use std::fmt::Write;

        for byte in result {
            let _ = write!(&mut hex, "{byte:02x}");
        }

        return ValueRef::str(hex.as_ref()).into_raw(ctx);
    }
    panic!("sha384() missing 1 required positional argument: 'value'");
}

// sha512(value: str, encoding: str = "utf-8") -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_crypto_sha512(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);
    if let Some(s) = args.arg_i_str(0, None) {
        let mut hasher = Sha512::new();
        hasher.update(&s);
        let result = hasher.finalize();

        let mut hex = String::with_capacity(2 * Sha256::output_size());
        use std::fmt::Write;

        for byte in result {
            let _ = write!(&mut hex, "{byte:02x}");
        }

        return ValueRef::str(hex.as_ref()).into_raw(ctx);
    }
    panic!("sha512() missing 1 required positional argument: 'value'");
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_crypto_uuid(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    return ValueRef::str(Uuid::new_v4().to_string().as_ref()).into_raw(ctx);
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_crypto_filesha256(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);
    if let Some(filepath) = args.arg_i_str(0, None) {
        // Open the file
        let mut file = File::open(&filepath)
            .unwrap_or_else(|e| panic!("failed to access file '{}': {}", filepath, e));

        // Create a SHA256 hasher instance
        let mut hasher = Sha256::new();

        // Read the file content and update the hasher
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .unwrap_or_else(|e| panic!("failed to read file '{}': {}", filepath, e));
        hasher.update(&buffer);

        // Compute the SHA256 hash
        let hash_result = hasher.finalize();

        let mut hex = String::with_capacity(2 * Sha256::output_size());
        use std::fmt::Write;

        for byte in hash_result {
            let _ = write!(&mut hex, "{byte:02x}");
        }

        return ValueRef::str(hex.as_str()).into_raw(ctx);
    }
    panic!("filesha256() missing 1 required positional argument: 'filepath'");
}
