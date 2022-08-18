Porting ['rustc_errors/styled_buffer.rs'] and ['rustc_errors/lock.rs'] code here to enable code reuse due to the unstable and unreusable of the ['rustc_errors'] crate now.
We mainly reuse helper structs and functions like `StyledBuffer`, `StyledString` to render text in Compiler-Base, and reuse helper function `acquire_global_lock` to emit the diagnostic messages.

Note: the structs and functions here exist as implementations and will not be exposed to other crates directly.

Reuse 'styled_buffer.rs' and 'lock.rs' in 'rustc_errors', 
and 'styled_buffer.rs' has been modified to fit the feature of 'Compiler-Base'.

We modified some features on porting code:
- add method `appendl()` and `pushs()` to 'StyledBuffer'.
- replaced the `enum Style` with generics `T: Clone + PartialEq + Eq + Style` to support extending more styles, because we need that `StyledBuffer` is still valid when facing the user-defined style, rather than just supporting a built-in `enum Style`.
- added some test cases for 'StyledBuffer' with 'trait Style'.

If anyone feels uncomfortable, please feel free to contact us.