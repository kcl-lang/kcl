Porting ['rustc_span'] code here to enable code reuse due to the unstable and unreusable of the ['rustc_span'] crate now.
We mainly reuse helper structs and functions like `rustc_span::span`, `rustc_span::spandata`, `rustc_span::sourcemap` to manage source positions in KCLVM.

Note: the structs and functions here exist as implementations and will not be exposed to other crates directly.

We remove features on porting code:
+ remove RUST specific features, such as edition and macro hygiene.
+ remove features using unstable Rust features.

Rewrite or use of other implementation projects may be considered in the future.

If anyone feels uncomfortable, please feel free to contact us.