use diagnostic::diagnostic_derive;
use synstructure::decl_derive;
mod diagnostic;

decl_derive!(
    [DiagnosticBuilder, attributes(
        // struct attributes
        error,
        warning,
        help,
        nopendant,
        note,
        help,
        // field attributes
        position,
        )] => diagnostic_derive
);
