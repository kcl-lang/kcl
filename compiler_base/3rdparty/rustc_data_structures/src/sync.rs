//! This module defines types which are thread safe if cfg!(parallel_compiler) is true.
//!
//! `Lrc` is an alias of `Arc` if cfg!(parallel_compiler) is true, `Rc` otherwise.
//!
//! `Lock` is a mutex.
//! It internally uses `parking_lot::Mutex` if cfg!(parallel_compiler) is true,
//! `RefCell` otherwise.
//!
//! `RwLock` is a read-write lock.
//! It internally uses `parking_lot::RwLock` if cfg!(parallel_compiler) is true,
//! `RefCell` otherwise.
//!
//! `MTLock` is a mutex which disappears if cfg!(parallel_compiler) is false.
//!
//! `MTRef` is an immutable reference if cfg!(parallel_compiler), and a mutable reference otherwise.
//!
//! `rustc_erase_owner!` erases an OwningRef owner into Erased or Erased + Send + Sync
//! depending on the value of cfg!(parallel_compiler).

use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

pub use std::sync::atomic::Ordering;
pub use std::sync::atomic::Ordering::SeqCst;

pub use std::marker::Send;
pub use std::marker::Sync;

pub use parking_lot::MappedRwLockReadGuard as MappedReadGuard;
pub use parking_lot::MappedRwLockWriteGuard as MappedWriteGuard;
pub use parking_lot::RwLockReadGuard as ReadGuard;
pub use parking_lot::RwLockWriteGuard as WriteGuard;

pub use parking_lot::MappedMutexGuard as MappedLockGuard;
pub use parking_lot::MutexGuard as LockGuard;

pub use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize};

pub use std::sync::Arc as Lrc;
pub use std::sync::Weak;

pub type MTRef<'a, T> = &'a T;

pub use rayon::{join, scope};

/// Runs a list of blocks in parallel. The first block is executed immediately on
/// the current thread. Use that for the longest running block.
#[macro_export]
macro_rules! parallel {
            (impl $fblock:tt [$($c:tt,)*] [$block:tt $(, $rest:tt)*]) => {
                parallel!(impl $fblock [$block, $($c,)*] [$($rest),*])
            };
            (impl $fblock:tt [$($blocks:tt,)*] []) => {
                ::rustc_data_structures::sync::scope(|s| {
                    $(
                        s.spawn(|_| $blocks);
                    )*
                    $fblock;
                })
            };
            ($fblock:tt, $($blocks:tt),*) => {
                // Reverse the order of the later blocks since Rayon executes them in reverse order
                // when using a single thread. This ensures the execution order matches that
                // of a single threaded rustc
                parallel!(impl $fblock [] [$($blocks),*]);
            };
        }

pub use rayon_core::WorkerLocal;

use rayon::iter::IntoParallelIterator;
pub use rayon::iter::ParallelIterator;

pub fn par_iter<T: IntoParallelIterator>(t: T) -> T::Iter {
    t.into_par_iter()
}

pub fn par_for_each_in<T: IntoParallelIterator>(t: T, for_each: impl Fn(T::Item) + Sync + Send) {
    t.into_par_iter().for_each(for_each)
}

#[macro_export]
macro_rules! rustc_erase_owner {
    ($v:expr) => {{
        let v = $v;
        ::rustc_data_structures::sync::assert_send_val(&v);
        v.erase_send_sync_owner()
    }};
}

pub fn assert_sync<T: ?Sized + Sync>() {}
pub fn assert_send<T: ?Sized + Send>() {}
pub fn assert_send_val<T: ?Sized + Send>(_t: &T) {}
pub fn assert_send_sync_val<T: ?Sized + Sync + Send>(_t: &T) {}

pub trait HashMapExt<K, V> {
    /// Same as HashMap::insert, but it may panic if there's already an
    /// entry for `key` with a value not equal to `value`
    fn insert_same(&mut self, key: K, value: V);
}

impl<K: Eq + Hash, V: Eq, S: BuildHasher> HashMapExt<K, V> for HashMap<K, V, S> {
    fn insert_same(&mut self, key: K, value: V) {
        self.entry(key)
            .and_modify(|old| assert!(*old == value))
            .or_insert(value);
    }
}
