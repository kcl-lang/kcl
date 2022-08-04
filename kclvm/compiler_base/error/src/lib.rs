use compiler_base_diagnostic::{
    pendant::*, Diagnostic, DiagnosticBuilder, Message, Position, Sentence,
};
use compiler_base_macros::DiagnosticBuilderMacro;

// before
// 
// KCL Complier Error[E2L28] : Unique key error
// ---> File /schema/same_name/main.k:5:1
// 5 |schema Person:
//  1 ^  -> Failure
// Variable name 'Person' must be unique in package context

// after
//
// error[E2L28]:Unique key error
// ---> File /schema/same_name_fail/main.k:5:2
//   |
// 5 |schema Person:
//   |^1 Failure
// - Variable name must be unique in package context

#[derive(DiagnosticBuilderMacro)]
#[error(title, msg="Unique key error", code="E2L28")]
#[nopendant(msg="Variable name must be unique in package context")]
struct UniqueKeyError{
    #[position(msg="Failure")]
    pos: Position
}
