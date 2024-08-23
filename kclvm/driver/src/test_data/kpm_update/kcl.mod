[package]
name = "kpm_update"
edition = "0.0.1"
version = "0.0.1"

[dependencies]
flask = { git = "https://github.com/kcl-lang/flask-demo-kcl-manifests", commit = "ade147b" }
helloworld = { oci = "oci://ghcr.io/kcl-lang/helloworld", tag = "0.1.0" }
