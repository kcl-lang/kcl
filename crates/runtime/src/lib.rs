//! Copyright The KCL Authors. All rights reserved.

// api-spec:       kcl_context_t
// api-spec(c):    typedef struct kcl_context_t kcl_context_t;
// api-spec(llvm): %"kcl_context_t" = type { i8* }

// api-spec:       kcl_eval_scope_t
// api-spec(c):    typedef struct kcl_eval_scope_t kcl_eval_scope_t;
// api-spec(llvm): %"kcl_eval_scope_t" = type { i8* }

// api-spec:       kcl_type_t
// api-spec(c):    typedef struct kcl_type_t kcl_type_t;
// api-spec(llvm): %"kcl_type_t" = type { i8* }

// api-spec:       kcl_value_t
// api-spec(c):    typedef struct kcl_value_t kcl_value_t;
// api-spec(llvm): %"kcl_value_t" = type { i8* }

// api-spec:       kcl_value_ref_t
// api-spec(c):    typedef struct kcl_value_ref_t kcl_value_ref_t;
// api-spec(llvm): %"kcl_value_ref_t" = type { i8* }

// api-spec:       kcl_iterator_t
// api-spec(c):    typedef struct kcl_iterator_t kcl_iterator_t;
// api-spec(llvm): %"kcl_iterator_t" = type { i8* }

// api-spec:       kcl_buffer_t
// api-spec(c):    typedef struct kcl_buffer_t kcl_buffer_t;
// api-spec(llvm): %"kcl_buffer_t" = type { i8* }

// api-spec:       kcl_kind_t
// api-spec(c):    typedef enum kcl_kind_t kcl_kind_t;
// api-spec(llvm): %"kcl_kind_t" = type i32

// api-spec:       kcl_size_t
// api-spec(c):    typedef int32_t kcl_size_t;
// api-spec(llvm): %"kcl_size_t" = type i32

// api-spec:       kcl_char_t
// api-spec(c):    typedef char kcl_char_t;
// api-spec(llvm): %"kcl_char_t" = type i8

// api-spec:       kcl_bool_t
// api-spec(c):    typedef int8_t kcl_bool_t;
// api-spec(llvm): %"kcl_bool_t" = type i8

// api-spec:       kcl_int_t
// api-spec(c):    typedef int64_t kcl_int_t;
// api-spec(llvm): %"kcl_int_t" = type i64

// api-spec:       kcl_float_t
// api-spec(c):    typedef double kcl_float_t;
// api-spec(llvm): %"kcl_float_t" = type double

// api-spec:       kcl_decorator_value_t
// api-spec(c):    typedef struct kcl_decorator_value_t kcl_decorator_value_t;
// api-spec(llvm): %"kcl_decorator_value_t" = type opaque

pub mod api;
pub use self::api::*;

pub mod context;
pub use self::context::*;

pub mod types;
pub use self::types::*;

pub mod unification;

pub mod value;
pub use self::value::*;

pub mod base32;
pub use self::base32::*;

pub mod base64;
pub use self::base64::*;

pub mod collection;
pub use self::collection::*;

pub mod crypto;
pub use self::crypto::*;

mod eval;

pub mod datetime;
pub use self::datetime::*;

pub mod encoding;

pub mod json;
pub use self::json::*;

pub mod manifests;
pub use self::manifests::*;

pub mod math;
pub use self::math::*;

pub mod net;
pub use self::net::*;

pub mod regex;
pub use self::regex::*;

pub mod stdlib;
pub use self::stdlib::*;

pub mod units;
pub use self::units::*;

pub mod yaml;
pub use self::yaml::*;

pub mod file;
pub use self::file::*;

pub mod template;
pub use self::template::*;

pub mod panic;
pub use self::panic::*;

pub mod _kcl_run;
pub use self::_kcl_run::*;

pub mod _kcl;
pub use self::_kcl::*;

pub mod _kcl_addr;
pub use self::_kcl_addr::*;
