Porting ['rustc_errors/styled_buffer.rs'] code here to enable code reuse due to the unstable and unreusable of the ['rustc_errors'] crate now.
We mainly reuse helper structs and functions like `StyledBuffer`, `StyledString` to render text in Compiler-Base.
Note: the structs and functions here exist as implementations and will not be exposed to other crates directly.

Reuse 'styled_buffer.rs' in 'rustc_errors', 
and 'styled_buffer.rs' has been modified to fit the feature of 'Compiler-Base'.

We modified some features on porting code:
- add method 'appendl()' and 'putl()' to 'StyledBuffer'.
- replaced the 'enum Style' with 'trait Style' to support extending more styles.
- added some test cases for 'StyledBuffer' with 'trait Style'.

If anyone feels uncomfortable, please feel free to contact us.