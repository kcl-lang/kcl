use bstr::ByteSlice;
use num_bigint::{BigInt, BigUint, Sign};

/// Convert rust bytes to BigInt with the int base.
pub fn bytes_to_int(lit: &[u8], mut base: u32) -> Option<i64> {
    // split sign
    let mut lit = lit.trim();
    let sign = match lit.first()? {
        b'+' => Some(Sign::Plus),
        b'-' => Some(Sign::Minus),
        _ => None,
    };
    if sign.is_some() {
        lit = &lit[1..];
    }

    // split radix
    let first = *lit.first()?;
    let (has_radix, radix_len) = if first == b'0' {
        match base {
            0 => {
                if let Some(parsed) = lit.get(1) {
                    let (base_deteced, radix_len) = detect_base(parsed);
                    base = base_deteced;
                    (true, radix_len)
                } else {
                    if let [_first, ref others @ .., last] = lit {
                        let is_zero =
                            others.iter().all(|&c| c == b'0' || c == b'_') && *last == b'0';
                        if !is_zero {
                            return None;
                        }
                    }
                    return Some(0);
                }
            }
            16 => lit
                .get(1)
                .map_or((false, 0), |&b| (matches!(b, b'x' | b'X'), 2)),
            2 => lit
                .get(1)
                .map_or((false, 0), |&b| (matches!(b, b'b' | b'B'), 2)),
            8 => lit
                .get(1)
                .map_or((false, 0), |&b| (matches!(b, b'o' | b'O'), 2)),
            _ => (false, 0),
        }
    } else {
        if base == 0 {
            base = 10;
        }
        (false, 0)
    };
    if has_radix {
        lit = &lit[radix_len as usize..];
        if lit.first()? == &b'_' {
            lit = &lit[1..];
        }
    }

    // remove zeroes
    let mut last = *lit.first()?;
    if last == b'0' {
        let mut count = 0;
        for &cur in &lit[1..] {
            if cur == b'_' {
                if last == b'_' {
                    return None;
                }
            } else if cur != b'0' {
                break;
            };
            count += 1;
            last = cur;
        }
        let prefix_last = lit[count];
        lit = &lit[count + 1..];
        if lit.is_empty() && prefix_last == b'_' {
            return None;
        }
    }

    // validate
    for c in lit.iter() {
        let c = *c;
        if !(c.is_ascii_alphanumeric() || c == b'_') {
            return None;
        }

        if c == b'_' && last == b'_' {
            return None;
        }

        last = c;
    }
    if last == b'_' {
        return None;
    }

    // parse
    if lit.is_empty() {
        Some(0)
    } else {
        let uint = BigUint::parse_bytes(lit, base)?;
        let num = BigInt::from_biguint(sign.unwrap_or(Sign::Plus), uint);
        let (sign, data) = num.to_u64_digits();
        if data.len() != 1 {
            None
        } else {
            match sign {
                Sign::Minus => Some(-(data[0] as i64)),
                Sign::Plus | Sign::NoSign => Some(data[0] as i64),
            }
        }
    }
}

fn detect_base(c: &u8) -> (u32, u32) {
    match c {
        b'x' | b'X' => (16, 2),
        b'b' | b'B' => (2, 2),
        b'o' | b'O' => (8, 2),
        _ => (8, 1),
    }
}
