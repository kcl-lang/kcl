//! Copyright The KCL Authors. All rights reserved.

use rustc_hash::FxBuildHasher;
use std::collections as imp;

pub type DefaultHashBuilder = FxBuildHasher;
pub type DefaultHasher = <DefaultHashBuilder as core::hash::BuildHasher>::Hasher;
pub type HashMap<K, V, S = DefaultHashBuilder> = imp::HashMap<K, V, S>;
pub type HashSet<V, S = DefaultHashBuilder> = imp::HashSet<V, S>;
pub type IndexMap<K, V, S = DefaultHashBuilder> = indexmap::IndexMap<K, V, S>;
pub type IndexSet<V, S = DefaultHashBuilder> = indexmap::IndexSet<V, S>;
