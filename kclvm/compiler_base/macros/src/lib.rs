use diagnostic_builder::diagnostic_builder_derive;
use synstructure::decl_derive;
mod diagnostic_builder;
mod error;

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
