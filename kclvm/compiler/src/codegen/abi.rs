//! Copyright 2021 The KCL Authors. All rights reserved.
//! Reference: https://github.com/rust-lang/rust/blob/master/compiler/rustc_target/src/lib.rs

#![allow(dead_code)]

use std::convert::TryInto;

/// Operations may overflow and they are needed to be checked
#[derive(Copy, Clone)]
pub enum OverflowOp {
    Add,
    Sub,
    Mul,
}

/// An identifier that specifies the address space that some operation
/// should operate on. Special address spaces have an effect on code generation,
/// depending on the target and the address spaces it implements.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddressSpace(pub u32);

impl AddressSpace {
    /// The default address space, corresponding to data space.
    pub const DATA: Self = AddressSpace(0);
}

/// Alignment of a type in bytes (always a power of two).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Align {
    pow2: u8,
}

impl Align {
    pub const ONE: Align = Align { pow2: 0 };

    #[inline]
    pub fn from_bits(bits: u64) -> Result<Align, String> {
        Align::from_bytes(Size::from_bits(bits).bytes())
    }

    #[inline]
    pub fn from_bytes(align: u64) -> Result<Align, String> {
        // Treat an alignment of 0 bytes like 1-byte alignment.
        if align == 0 {
            return Ok(Align::ONE);
        }

        #[cold]
        fn not_power_of_2(align: u64) -> String {
            format!("`{}` is not a power of 2", align)
        }

        #[cold]
        fn too_large(align: u64) -> String {
            format!("`{}` is too large", align)
        }

        let mut bytes = align;
        let mut pow2: u8 = 0;
        while (bytes & 1) == 0 {
            pow2 += 1;
            bytes >>= 1;
        }
        if bytes != 1 {
            return Err(not_power_of_2(align));
        }
        if pow2 > 29 {
            return Err(too_large(align));
        }

        Ok(Align { pow2 })
    }

    #[inline]
    pub fn bytes(self) -> u64 {
        1 << self.pow2
    }

    #[inline]
    pub fn bits(self) -> u64 {
        self.bytes() * 8
    }

    /// Computes the best alignment possible for the given offset
    /// (the largest power of two that the offset is a multiple of).
    ///
    /// N.B., for an offset of `0`, this happens to return `2^64`.
    #[inline]
    pub fn max_for_offset(offset: Size) -> Align {
        Align {
            pow2: offset.bytes().trailing_zeros() as u8,
        }
    }

    /// Lower the alignment, if necessary, such that the given offset
    /// is aligned to it (the offset is a multiple of the alignment).
    #[inline]
    pub fn restrict_for_offset(self, offset: Size) -> Align {
        self.min(Align::max_for_offset(offset))
    }
}

/// Size of a type in bytes.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Size {
    // The top 3 bits are ALWAYS zero.
    raw: u64,
}

impl Size {
    pub const ZERO: Size = Size { raw: 0 };

    /// Rounds `bits` up to the next-higher byte boundary, if `bits` is
    /// is not aligned.
    pub fn from_bits(bits: impl TryInto<u64>) -> Size {
        let bits = bits.try_into().ok().unwrap();

        #[cold]
        fn overflow(bits: u64) -> ! {
            panic!("Size::from_bits({}) has overflowed", bits);
        }

        // This is the largest value of `bits` that does not cause overflow
        // during rounding, and guarantees that the resulting number of bytes
        // cannot cause overflow when multiplied by 8.
        if bits > 0xffff_ffff_ffff_fff8 {
            overflow(bits);
        }

        // Avoid potential overflow from `bits + 7`.
        Size {
            raw: bits / 8 + ((bits % 8) + 7) / 8,
        }
    }

    #[inline]
    pub fn from_bytes(bytes: impl TryInto<u64>) -> Size {
        let bytes: u64 = bytes.try_into().ok().unwrap();
        Size { raw: bytes }
    }

    #[inline]
    pub fn bytes(self) -> u64 {
        self.raw
    }

    #[inline]
    pub fn bytes_usize(self) -> usize {
        self.bytes().try_into().unwrap()
    }

    #[inline]
    pub fn bits(self) -> u64 {
        self.raw << 3
    }

    #[inline]
    pub fn bits_usize(self) -> usize {
        self.bits().try_into().unwrap()
    }

    #[inline]
    pub fn align_to(self, align: Align) -> Size {
        let mask = align.bytes() - 1;
        Size::from_bytes((self.bytes() + mask) & !mask)
    }

    #[inline]
    pub fn is_aligned(self, align: Align) -> bool {
        let mask = align.bytes() - 1;
        self.bytes() & mask == 0
    }

    /// Truncates `value` to `self` bits and then sign-extends it to 128 bits
    /// (i.e., if it is negative, fill with 1's on the left).
    #[inline]
    pub fn sign_extend(self, value: u128) -> u128 {
        let size = self.bits();
        if size == 0 {
            // Truncated until nothing is left.
            return 0;
        }
        // Sign-extend it.
        let shift = 128 - size;
        // Shift the unsigned value to the left, then shift back to the right as signed
        // (essentially fills with sign bit on the left).
        (((value << shift) as i128) >> shift) as u128
    }

    /// Truncates `value` to `self` bits.
    #[inline]
    pub fn truncate(self, value: u128) -> u128 {
        let size = self.bits();
        if size == 0 {
            // Truncated until nothing is left.
            return 0;
        }
        let shift = 128 - size;
        // Truncate (shift left to drop out leftover values, shift right to fill with zeroes).
        (value << shift) >> shift
    }

    #[inline]
    pub fn signed_int_min(&self) -> i128 {
        self.sign_extend(1_u128 << (self.bits() - 1)) as i128
    }

    #[inline]
    pub fn signed_int_max(&self) -> i128 {
        i128::MAX >> (128 - self.bits())
    }

    #[inline]
    pub fn unsigned_int_max(&self) -> u128 {
        u128::MAX >> (128 - self.bits())
    }
}
