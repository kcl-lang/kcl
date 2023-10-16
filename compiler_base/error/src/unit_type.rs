//! This file provides some of the self-encapsulated types used in handling error messages.

/// [`TyeWithUnit`] is a trait for types that can be converted into a string with a unit.
pub trait TypeWithUnit {
    fn into_string_with_unit(self) -> String;
}

/// [`UnitUsize`] is a [`usize`] type that can be converted into a string with a unit.
pub struct UnitUsize(pub usize, pub String);

impl TypeWithUnit for UnitUsize {
    /// [`into_string_with_unit`] converts [`UnitUsize`] into a string with a unit.
    ///
    /// # Examples
    ///
    /// ```
    /// use compiler_base_error::unit_type::{TypeWithUnit, UnitUsize};
    ///
    /// let unit_usize = UnitUsize(1, "byte".to_string());
    /// assert_eq!(unit_usize.into_string_with_unit(), "1 byte");
    /// let unit_usize = UnitUsize(2, "byte".to_string());
    /// assert_eq!(unit_usize.into_string_with_unit(), "2 bytes");
    /// ```
    fn into_string_with_unit(self) -> String {
        if self.0 > 1 {
            format!("{} {}s", self.0, self.1)
        } else {
            format!("{} {}", self.0, self.1)
        }
    }
}
