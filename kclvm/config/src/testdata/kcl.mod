[package]
name = "test_add_deps"
edition = "0.0.1"
version = "0.0.1"

[dependencies]
pkg0 = { git = "test_url", tag = "test_tag" }
pkg1 = "oci_tag1"
pkg2 = { oci = "oci://ghcr.io/kcl-lang/helloworld", tag = "0.1.1" }
pkg3 = { path = "../pkg"}

[profile]
entries = ["main.k"]
