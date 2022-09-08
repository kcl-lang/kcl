use std::{error, fmt, panic};

/// `bug!` macro is used to report compiler internal bug.
/// You can use bug! macros directly by adding `#[macro_use]extern crate kclvm_error;`
/// in the lib.rs, and then call as follows:
/// ```no_check
/// bug!();
/// bug!("an error msg");
/// bug!("an error msg with string format {}", "msg");
/// ```
#[macro_export]
macro_rules! bug {
    () => ( $crate::bug::bug("impossible case reached") );
    ($msg:expr) => ({ $crate::bug::bug(&format!($msg)) });
    ($msg:expr,) => ({ $crate::bug::bug($msg) });
    ($fmt:expr, $($arg:tt)+) => ({
        $crate::bug::bug(&format!($fmt, $($arg)+))
    });
}

/// Signifies that the compiler died with an explicit call to `.bug`
/// rather than a failed assertion, etc.
#[derive(Clone, Debug)]
pub struct ExplicitBug {
    msg: String,
}

impl fmt::Display for ExplicitBug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Internal error, please report a bug to us. The error message is: {}",
            self.msg
        )
    }
}

impl error::Error for ExplicitBug {}

#[inline]
pub fn bug(msg: &str) -> ! {
    panic!(
        "{}",
        ExplicitBug {
            msg: msg.to_string()
        }
    );
}
