//! Various data structures used by the Rust compiler. The intention
//! is that code in here should be not be *specific* to rustc, so that
//! it can be easily unit tested and so forth.
//!
//! # Note
//!
//! This API is completely unstable and subject to change.

#![doc(html_root_url = "https://doc.rust-lang.org/nightly/nightly-rustc/")]
#![allow(rustc::default_hash_types)]
#![deny(unaligned_references)]
#![allow(rustc::potential_query_instability)]

extern crate tracing;
#[macro_use]
extern crate cfg_if;

#[inline(never)]
#[cold]
pub fn cold_path<F: FnOnce() -> R, R>(f: F) -> R {
    f()
}

#[macro_export]
macro_rules! likely {
    ($e:expr) => {
        match $e {
            #[allow(unused_unsafe)]
            e => unsafe { std::intrinsics::likely(e) },
        }
    };
}

pub mod base_n;

pub mod captures;
pub mod flock;
pub mod fx;

pub mod macros;
pub mod stable_map;
pub use ena::snapshot_vec;
pub mod stable_set;
#[macro_use]

mod atomic_ref;
pub mod stack;
pub mod sync;
pub use atomic_ref::AtomicRef;
pub mod frozen;

pub mod temp_dir;
pub mod unhash;

pub use ena::undo_log;
pub use ena::unify;

pub struct OnDrop<F: Fn()>(pub F);

impl<F: Fn()> OnDrop<F> {
    /// Forgets the function which prevents it from running.
    /// Ensure that the function owns no memory, otherwise it will be leaked.
    #[inline]
    pub fn disable(self) {
        std::mem::forget(self);
    }
}

impl<F: Fn()> Drop for OnDrop<F> {
    #[inline]
    fn drop(&mut self) {
        (self.0)();
    }
}

// See comments in src/librustc_middle/lib.rs
#[doc(hidden)]
pub fn __noop_fix_for_27438() {}
