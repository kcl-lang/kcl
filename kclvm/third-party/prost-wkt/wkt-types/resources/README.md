The files contained in this directory is from [prost-types](https://raw.githubusercontent.com/tokio-rs/prost/v0.10.0/prost-types/src/). It is used by `build.rs` to create the appropriate `prost_snippet.rs` in `./src/pbtime.rs`.

When updating the Prost dependencies in this project you should run the `update.sh` script in this directory. This script
will update the above mentioned files. If the files are updated, do validate whether the line numbers selected in the
`../build.rs` are still correct.
