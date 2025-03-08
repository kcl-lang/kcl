[package]
name = "pkg_mod"
version = "0.0.1"

[dependencies]
pkg_mod_test = { path = "../.." }

[profile]
entries = ["../base/base.k", "main.k", "${pkg_mod_test:KCL_MOD}/pkg1/a.k"]