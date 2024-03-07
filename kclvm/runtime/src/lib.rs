//! Copyright The KCL Authors. All rights reserved.

use kclvm_runtime_internal_macros::runtime_fn;

// api-spec:       kclvm_context_t
// api-spec(c):    typedef struct kclvm_context_t kclvm_context_t;
// api-spec(llvm): %"kclvm_context_t" = type { i8* }

// api-spec:       kclvm_eval_scope_t
// api-spec(c):    typedef struct kclvm_eval_scope_t kclvm_eval_scope_t;
// api-spec(llvm): %"kclvm_eval_scope_t" = type { i8* }

// api-spec:       kclvm_type_t
// api-spec(c):    typedef struct kclvm_type_t kclvm_type_t;
// api-spec(llvm): %"kclvm_type_t" = type { i8* }

// api-spec:       kclvm_value_t
// api-spec(c):    typedef struct kclvm_value_t kclvm_value_t;
// api-spec(llvm): %"kclvm_value_t" = type { i8* }

// api-spec:       kclvm_value_ref_t
// api-spec(c):    typedef struct kclvm_value_ref_t kclvm_value_ref_t;
// api-spec(llvm): %"kclvm_value_ref_t" = type { i8* }

// api-spec:       kclvm_iterator_t
// api-spec(c):    typedef struct kclvm_iterator_t kclvm_iterator_t;
// api-spec(llvm): %"kclvm_iterator_t" = type { i8* }

// api-spec:       kclvm_buffer_t
// api-spec(c):    typedef struct kclvm_buffer_t kclvm_buffer_t;
// api-spec(llvm): %"kclvm_buffer_t" = type { i8* }

// api-spec:       kclvm_kind_t
// api-spec(c):    typedef enum kclvm_kind_t kclvm_kind_t;
// api-spec(llvm): %"kclvm_kind_t" = type i32

// api-spec:       kclvm_size_t
// api-spec(c):    typedef int32_t kclvm_size_t;
// api-spec(llvm): %"kclvm_size_t" = type i32

// api-spec:       kclvm_char_t
// api-spec(c):    typedef char kclvm_char_t;
// api-spec(llvm): %"kclvm_char_t" = type i8

// api-spec:       kclvm_bool_t
// api-spec(c):    typedef int8_t kclvm_bool_t;
// api-spec(llvm): %"kclvm_bool_t" = type i8

// api-spec:       kclvm_int_t
// api-spec(c):    typedef int64_t kclvm_int_t;
// api-spec(llvm): %"kclvm_int_t" = type i64

// api-spec:       kclvm_float_t
// api-spec(c):    typedef double kclvm_float_t;
// api-spec(llvm): %"kclvm_float_t" = type double

// api-spec:       kclvm_decorator_value_t
// api-spec(c):    typedef struct kclvm_decorator_value_t kclvm_decorator_value_t;
// api-spec(llvm): %"kclvm_decorator_value_t" = type opaque

pub mod api;
pub use self::api::*;

pub mod context;
pub use self::context::*;

pub mod types;
pub use self::types::*;

pub mod unification;

pub mod value;
pub use self::value::*;

pub mod base64;
pub use self::base64::*;

pub mod collection;
pub use self::collection::*;

pub mod crypto;
pub use self::crypto::*;

mod eval;

pub mod datetime;
pub use self::datetime::*;

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

pub mod _kcl_run;
pub use self::_kcl_run::*;

pub mod _kclvm;
pub use self::_kclvm::*;

pub mod _kclvm_addr;
pub use self::_kclvm_addr::*;

type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;
