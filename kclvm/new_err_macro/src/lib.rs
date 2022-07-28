use diagnostics::diagnostic_derive;
use synstructure::decl_derive;

mod diagnostics;

#[cfg(test)]
mod tests;

decl_derive!(
    [DiagnosticBuilder, attributes(
        // struct attributes
        warning,
        error,
        note,
        help,
        position,
        nopendant,
        title,
        // field attributes
        )] => diagnostic_derive
);
