[package]
name = "pkg_mod"

[dependencies]
pkg_mod_test = { path = "../../pkg_mod_test" }

[profile]
entries = ["../base/base.k", "main.k", "${pkg_mod_test:KCL_MOD}/pkg1/a.k"]