use diagnostic::diagnostic_builder::diagnostic_builder_derive;
use synstructure::decl_derive;
mod diagnostic;

// 'DiagnosticBuilderMacro' is a custom #[derive] macros 
// that specify the implementation code for trait 'DiagnosticBuilder' 
// added with the derive attribute used on error/warning structs.
//
// 'DiagnosticBuilderMacro' provides 6 attributes to define the message
// defined in 'Diagnostic' generated from 'DiagnosticBuilder'. 
// 
// error : The label of the output information is 'error', 
//         sub-attribute 'msg' is required.
//         sub-attribute 'code' is optional.
//         sub-attribure 'title' is optional. 
//
// note: 'title' will not change the content of the generated diagnostic 
// information, and 'title' decides that the location where the diagnostic 
// information is generated is above the entire diagnostic information.
//
// # Examples
// ```
// #[error(msg = "hello error !")]
// "error: hello error !" 
// #[error(msg = "hello error !"), code = "E0101"]
// "error[E0101]: hello error !" 
// ```
//
// warning : The label of the output information is 'warning', 
//           sub-attribute 'msg' is required.
//           sub-attribute 'code' is optional.
//           sub-attribure 'title' is optional. 
//
// help : The label of the output information is 'help', 
//        sub-attribute 'msg' is required.
//        sub-attribure 'title' is optional. 
//
// nopendant : The output information has no label, 
//             sub-attribute 'msg' is required.
//             sub-attribure 'title' is optional. 
//
// note : The label of the output information is 'note', 
//           sub-attribute 'msg' is required.
//           sub-attribute 'code' is optional.
//           sub-attribure 'title' is optional. 
//
// position : The output information has no label, 
//            sub-attribute 'msg' is required.
//
// 'position' needs to be bound to a field of the struct 
// to generate message with code context, and the bound 
// field type is 'Position'.
//
// # Examples
// ```
// ....
// struct TypeError{
//     ...
//     #[position(msg="error message !")]
//     pos: Postion // Bad code context information.
// }
// 
// The diagnostic message looks like:
// 
// --> /code_src/mycode.rs:3:5
// |
// 3 |     sad()
// |     ^ error message !
// ```
decl_derive!(
    [DiagnosticBuilderMacro, attributes(
        // struct attributes
        error,
        warning,
        help,
        nopendant,
        note,
        // field attributes
        position,
        )] => diagnostic_builder_derive
);
