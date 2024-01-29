//! Copyright The KCL Authors. All rights reserved.

use crate::*;
use bstr::ByteSlice;
use unic_ucd_bidi::BidiClass;
use unic_ucd_category::GeneralCategory;
use unicode_casing::CharExt;

const ASCII_WHITESPACES: [u8; 6] = [0x20, 0x09, 0x0a, 0x0c, 0x0d, 0x0b];

fn adjust_indices(
    start: Option<&ValueRef>,
    end: Option<&ValueRef>,
    len: usize,
) -> std::ops::Range<usize> {
    let mut start = start.map_or(0, |v| {
        if v.is_none_or_undefined() {
            0
        } else {
            v.as_int() as isize
        }
    });
    let mut end = end.map_or(len as isize, |v| {
        if v.is_none_or_undefined() {
            len as isize
        } else {
            v.as_int() as isize
        }
    });
    if end > len as isize {
        end = len as isize;
    } else if end < 0 {
        end += len as isize;
        if end < 0 {
            end = 0;
        }
    }
    if start < 0 {
        start += len as isize;
        if start < 0 {
            start = 0;
        }
    }
    start as usize..end as usize
}

fn is_case<F, G>(value: &str, is_case: F, is_opposite: G) -> bool
where
    F: Fn(char) -> bool,
    G: Fn(char) -> bool,
{
    let mut cased = false;
    for c in value.chars() {
        if is_opposite(c) {
            return false;
        } else if !cased && is_case(c) {
            cased = true
        }
    }
    cased
}

trait RangeNormal {
    fn is_normal(&self) -> bool;
}

impl RangeNormal for std::ops::Range<usize> {
    fn is_normal(&self) -> bool {
        self.start <= self.end
    }
}

impl ValueRef {
    pub fn str_len(&self) -> usize {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => v.len(),
            _ => panic!("Invalid str object in str len"),
        }
    }
    pub fn str_resize(&mut self, n: usize) {
        match &mut *self.rc.borrow_mut() {
            Value::str_value(str) => {
                *str = "?".repeat(n);
            }
            _ => panic!("Invalid str object in str resize"),
        }
    }

    pub fn str_lower(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => ValueRef::str(&v.to_lowercase()),
            _ => panic!("Invalid str object in lower"),
        }
    }

    pub fn str_upper(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => ValueRef::str(&v.to_uppercase()),
            _ => panic!("Invalid str object in upper"),
        }
    }

    pub fn str_capitalize(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let mut chars = v.chars();
                let value = if let Some(first_char) = chars.next() {
                    format!(
                        "{}{}",
                        first_char.to_uppercase(),
                        &chars.as_str().to_lowercase(),
                    )
                } else {
                    "".to_owned()
                };
                ValueRef::str(value.as_str())
            }
            _ => panic!("Invalid str object in str_capitalize"),
        }
    }

    pub fn str_count(
        &self,
        sub: &ValueRef,
        start: Option<&ValueRef>,
        end: Option<&ValueRef>,
    ) -> ValueRef {
        let start = adjust_parameter(start);
        let end = adjust_parameter(end);

        match (&*self.rc.borrow(), &*sub.rc.borrow()) {
            (Value::str_value(ref v), Value::str_value(ref sub_str)) => {
                let range = adjust_indices(start, end, v.len());
                let count = if range.is_normal() {
                    v.get(range).unwrap().matches(sub_str).count()
                } else {
                    0
                };
                ValueRef::int(count as i64)
            }
            _ => panic!("Invalid str object in str_count"),
        }
    }

    pub fn str_startswith(
        &self,
        prefix: &ValueRef,
        start: Option<&ValueRef>,
        end: Option<&ValueRef>,
    ) -> ValueRef {
        let start = adjust_parameter(start);
        let end = adjust_parameter(end);

        match (&*self.rc.borrow(), &*prefix.rc.borrow()) {
            (Value::str_value(ref v), Value::str_value(ref prefix)) => {
                let range = adjust_indices(start, end, v.len());
                let result = if range.is_normal() {
                    v.get(range).unwrap().starts_with(prefix)
                } else {
                    false
                };
                ValueRef::bool(result)
            }
            _ => panic!("Invalid str object in str_startswith"),
        }
    }

    pub fn str_endswith(
        &self,
        suffix: &ValueRef,
        start: Option<&ValueRef>,
        end: Option<&ValueRef>,
    ) -> ValueRef {
        let start = adjust_parameter(start);
        let end = adjust_parameter(end);

        match (&*self.rc.borrow(), &*suffix.rc.borrow()) {
            (Value::str_value(ref v), Value::str_value(ref suffix)) => {
                let range = adjust_indices(start, end, v.len());
                let result = if range.is_normal() {
                    v.get(range).unwrap().ends_with(suffix)
                } else {
                    false
                };
                ValueRef::bool(result)
            }
            _ => panic!("Invalid str object in str_endswith"),
        }
    }

    pub fn str_format(&self, args: &ValueRef, kwargs: &ValueRef) -> ValueRef {
        match (&*self.rc.borrow(), &*args.rc.borrow()) {
            (Value::str_value(ref v), Value::list_value(_)) => {
                match FormatString::from_str(v.as_str()) {
                    Ok(format_string) => {
                        let result = format_string.format(args, kwargs);
                        ValueRef::str(result.as_str())
                    }
                    Err(_) => panic!("format error"),
                }
            }
            _ => panic!("Invalid str object in str_format"),
        }
    }

    pub fn str_find(
        &self,
        sub: &ValueRef,
        start: Option<&ValueRef>,
        end: Option<&ValueRef>,
    ) -> ValueRef {
        let start = adjust_parameter(start);
        let end = adjust_parameter(end);

        match (&*self.rc.borrow(), &*sub.rc.borrow()) {
            (Value::str_value(ref v), Value::str_value(ref sub)) => {
                let range = adjust_indices(start, end, v.len());
                let range_start = range.start;
                let result: i64 = if range.is_normal() {
                    match v.get(range).unwrap().find(sub) {
                        Some(index) => (index + range_start) as i64,
                        None => -1,
                    }
                } else {
                    -1
                };
                ValueRef::int(result)
            }
            _ => panic!("Invalid str object in str_find"),
        }
    }

    pub fn str_rfind(
        &self,
        sub: &ValueRef,
        start: Option<&ValueRef>,
        end: Option<&ValueRef>,
    ) -> ValueRef {
        let start = adjust_parameter(start);
        let end = adjust_parameter(end);

        match (&*self.rc.borrow(), &*sub.rc.borrow()) {
            (Value::str_value(ref v), Value::str_value(ref sub)) => {
                let range = adjust_indices(start, end, v.len());
                let range_start = range.start;
                let result: i64 = if range.is_normal() {
                    match v.get(range).unwrap().rfind(sub) {
                        Some(index) => (index + range_start) as i64,
                        None => -1,
                    }
                } else {
                    -1
                };
                ValueRef::int(result)
            }
            _ => panic!("Invalid str object in str_rfind"),
        }
    }

    pub fn str_index(
        &self,
        sub: &ValueRef,
        start: Option<&ValueRef>,
        end: Option<&ValueRef>,
    ) -> ValueRef {
        let start = adjust_parameter(start);
        let end = adjust_parameter(end);

        match (&*self.rc.borrow(), &*sub.rc.borrow()) {
            (Value::str_value(ref v), Value::str_value(ref sub)) => {
                let range = adjust_indices(start, end, v.len());
                let range_start = range.start;
                let result: i64 = if range.is_normal() {
                    match v.get(range).unwrap().find(sub) {
                        Some(index) => (index + range_start) as i64,
                        None => panic!("substring not found"),
                    }
                } else {
                    panic!("substring not found");
                };
                ValueRef::int(result)
            }
            _ => panic!("Invalid str object in str_index"),
        }
    }

    pub fn str_rindex(
        &self,
        sub: &ValueRef,
        start: Option<&ValueRef>,
        end: Option<&ValueRef>,
    ) -> ValueRef {
        let start = adjust_parameter(start);
        let end = adjust_parameter(end);

        match (&*self.rc.borrow(), &*sub.rc.borrow()) {
            (Value::str_value(ref v), Value::str_value(ref sub)) => {
                let range = adjust_indices(start, end, v.len());
                let range_start = range.start;
                let result: i64 = if range.is_normal() {
                    match v.get(range).unwrap().rfind(sub) {
                        Some(index) => (index + range_start) as i64,
                        None => panic!("substring not found"),
                    }
                } else {
                    panic!("substring not found");
                };
                ValueRef::int(result)
            }
            _ => panic!("Invalid str object in str_rindex"),
        }
    }

    pub fn str_isalnum(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let result = !v.is_empty() && v.chars().all(char::is_alphanumeric);
                ValueRef::bool(result)
            }
            _ => panic!("Invalid str object in str_isalnum"),
        }
    }

    pub fn str_isalpha(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let result = !v.is_empty() && v.chars().all(char::is_alphabetic);
                ValueRef::bool(result)
            }
            _ => panic!("Invalid str object in str_isalpha"),
        }
    }

    pub fn str_isdigit(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let valid_unicodes: [u16; 10] = [
                    0x2070, 0x00B9, 0x00B2, 0x00B3, 0x2074, 0x2075, 0x2076, 0x2077, 0x2078, 0x2079,
                ];

                let result = !v.is_empty()
                    && v.chars()
                        .filter(|c| !c.is_ascii_digit())
                        .all(|c| valid_unicodes.contains(&(c as u16)));
                ValueRef::bool(result)
            }
            _ => panic!("Invalid str object in str_isdigit"),
        }
    }

    pub fn str_islower(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let result = is_case(v, char::is_lowercase, char::is_uppercase);
                ValueRef::bool(result)
            }
            _ => panic!("Invalid str object in str_islower"),
        }
    }

    pub fn str_isspace(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                if v.is_empty() {
                    return ValueRef::bool(false);
                }
                use unic_ucd_bidi::bidi_class::abbr_names::*;
                let result = v.chars().all(|c| {
                    GeneralCategory::of(c) == GeneralCategory::SpaceSeparator
                        || matches!(BidiClass::of(c), WS | B | S)
                });
                ValueRef::bool(result)
            }
            _ => panic!("Invalid str object in str_isspace"),
        }
    }

    pub fn str_istitle(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                if v.is_empty() {
                    return ValueRef::bool(false);
                }

                let mut cased = false;
                let mut previous_is_cased = false;
                for c in v.chars() {
                    if c.is_uppercase() || c.is_titlecase() {
                        if previous_is_cased {
                            return ValueRef::bool(false);
                        }
                        previous_is_cased = true;
                        cased = true;
                    } else if c.is_lowercase() {
                        if !previous_is_cased {
                            return ValueRef::bool(false);
                        }
                        previous_is_cased = true;
                        cased = true;
                    } else {
                        previous_is_cased = false;
                    }
                }
                ValueRef::bool(cased)
            }
            _ => panic!("Invalid str object in str_istitle"),
        }
    }

    pub fn str_isupper(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let result = is_case(v, char::is_uppercase, char::is_lowercase);
                ValueRef::bool(result)
            }
            _ => panic!("Invalid str object in str_isupper"),
        }
    }

    pub fn str_isnumeric(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let result = !v.is_empty() && v.chars().all(char::is_numeric);
                ValueRef::bool(result)
            }
            _ => panic!("Invalid str object in str_isnumeric"),
        }
    }

    pub fn str_join(&self, value: &ValueRef) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let mut joined = String::new();
                let mut iter = value.iter();
                while !iter.is_end() {
                    let iter_value = iter.next(value).unwrap();
                    joined.push_str(iter_value.to_string().as_str());
                    if !iter.is_end() {
                        joined.push_str(v);
                    }
                }
                ValueRef::str(joined.as_str())
            }
            _ => panic!("Invalid str object in str_joined"),
        }
    }

    pub fn str_lstrip(&self, value: Option<&ValueRef>) -> ValueRef {
        let value = adjust_parameter(value);

        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let value = match value {
                    Some(chars) => {
                        let chars = chars.as_str();
                        let chars = chars.as_str();
                        v.trim_start_matches(|c| chars.contains(c))
                    }
                    None => v.trim_start(),
                };
                ValueRef::str(value)
            }
            _ => panic!("Invalid str object in str_lstrip"),
        }
    }

    pub fn str_rstrip(&self, value: Option<&ValueRef>) -> ValueRef {
        let value = adjust_parameter(value);

        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let value = match value {
                    Some(chars) => {
                        let chars = chars.as_str();
                        let chars = chars.as_str();
                        v.trim_end_matches(|c| chars.contains(c))
                    }
                    None => v.trim_end(),
                };
                ValueRef::str(value)
            }
            _ => panic!("Invalid str object in str_rstrip"),
        }
    }

    pub fn str_replace(
        &self,
        old: &ValueRef,
        new: &ValueRef,
        count: Option<&ValueRef>,
    ) -> ValueRef {
        let count = adjust_parameter(count);

        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let old = old.as_str();
                let new = new.as_str();
                let result = match count {
                    Some(count) => {
                        let maxcount = count.as_int();
                        if maxcount >= 0 {
                            v.replacen(old.as_str(), new.as_str(), maxcount as usize)
                        } else {
                            v.replace(old.as_str(), new.as_str())
                        }
                    }
                    None => v.replace(old.as_str(), new.as_str()),
                };
                ValueRef::str(result.as_str())
            }
            _ => panic!("Invalid str object in str_replace"),
        }
    }

    /// If the string starts with the prefix string, return string[len(prefix):].
    /// Otherwise, return a copy of the original string.
    pub fn str_removeprefix(&self, prefix: &ValueRef) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let prefix = prefix.as_str();
                match v.strip_prefix(&prefix) {
                    Some(r) => ValueRef::str(r),
                    None => ValueRef::str(v),
                }
            }
            _ => panic!("Invalid str object in str_rstrip"),
        }
    }

    /// If the string ends with the suffix string and that suffix is not empty, return string[:-len(suffix)].
    /// Otherwise, return a copy of the original string.
    pub fn str_removesuffix(&self, suffix: &ValueRef) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let suffix = suffix.as_str();
                match v.strip_suffix(&suffix) {
                    Some(r) => ValueRef::str(r),
                    None => ValueRef::str(v),
                }
            }
            _ => panic!("Invalid str object in str_removesuffix"),
        }
    }

    pub fn str_split(&self, sep: Option<&ValueRef>, maxsplit: Option<&ValueRef>) -> ValueRef {
        let sep = adjust_parameter(sep);
        let maxsplit = adjust_parameter(maxsplit);

        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let convert = ValueRef::str;
                let maxsplit = match maxsplit {
                    Some(v) => v.as_int(),
                    None => -1,
                };
                let splited = if let Some(pattern) = sep {
                    let s = pattern.as_str();
                    if maxsplit < 0 {
                        v.split(s.as_str()).map(convert).collect()
                    } else {
                        v.splitn(maxsplit as usize + 1, s.as_str())
                            .map(convert)
                            .collect()
                    }
                } else {
                    // Split whitespace
                    let mut splited = Vec::new();
                    let mut last_offset = 0;
                    let mut count = maxsplit;
                    for (offset, _) in
                        v.match_indices(|c: char| c.is_ascii_whitespace() || c == '\x0b')
                    {
                        if last_offset == offset {
                            last_offset += 1;
                            continue;
                        }
                        if count == 0 {
                            break;
                        }
                        splited.push(convert(&v[last_offset..offset]));
                        last_offset = offset + 1;
                        count -= 1;
                    }
                    if last_offset != v.len() {
                        splited.push(convert(&v[last_offset..]));
                    }
                    splited
                };
                ValueRef::list_value(Some(splited.as_slice()))
            }
            _ => panic!("Invalid str object in str_split"),
        }
    }

    pub fn str_rsplit(&self, sep: Option<&ValueRef>, maxsplit: Option<&ValueRef>) -> ValueRef {
        let sep = adjust_parameter(sep);
        let maxsplit = adjust_parameter(maxsplit);

        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let convert = ValueRef::str;
                let maxsplit = match maxsplit {
                    Some(v) => v.as_int(),
                    None => -1,
                };
                let mut splited = if let Some(pattern) = sep {
                    let s = pattern.as_str();
                    if maxsplit < 0 {
                        v.rsplit(s.as_str()).map(convert).collect()
                    } else {
                        v.rsplitn(maxsplit as usize + 1, s.as_str())
                            .map(convert)
                            .collect()
                    }
                } else {
                    // Split whitespace
                    let mut splited = Vec::new();
                    let mut count = maxsplit;
                    let mut haystack = v.as_bytes();
                    while let Some(offset) = haystack.rfind_byteset(ASCII_WHITESPACES) {
                        if offset + 1 != haystack.len() {
                            if count == 0 {
                                break;
                            }
                            splited.push(convert(
                                std::str::from_utf8(&haystack[offset + 1..]).unwrap(),
                            ));
                            count -= 1;
                        }
                        haystack = &haystack[..offset];
                    }
                    if !haystack.is_empty() {
                        splited.push(convert(std::str::from_utf8(haystack).unwrap()));
                    }
                    splited
                };
                splited.reverse();
                ValueRef::list_value(Some(splited.as_slice()))
            }
            _ => panic!("Invalid str object in str_rsplit"),
        }
    }

    pub fn str_splitlines(&self, keepends: Option<&ValueRef>) -> ValueRef {
        let keepends = adjust_parameter(keepends);

        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let convert = ValueRef::str;
                let keepends = match keepends {
                    Some(v) => v.as_bool(),
                    None => false,
                };
                let keep = keepends as usize;
                let mut elements = Vec::new();
                let mut last_i = 0;
                let mut enumerated = v.as_bytes().iter().enumerate().peekable();
                while let Some((i, ch)) = enumerated.next() {
                    let (end_len, i_diff) = match *ch {
                        b'\n' => (keep, 1),
                        b'\r' => {
                            let is_rn = enumerated.peek().map_or(false, |(_, ch)| **ch == b'\n');
                            if is_rn {
                                let _ = enumerated.next();
                                (keep + keep, 2)
                            } else {
                                (keep, 1)
                            }
                        }
                        _ => {
                            continue;
                        }
                    };
                    let range = last_i..i + end_len;
                    last_i = i + i_diff;
                    elements.push(convert(&v[range]));
                }
                if last_i != v.len() {
                    elements.push(convert(&v[last_i..v.len()]));
                }
                ValueRef::list_value(Some(elements.as_slice()))
            }
            _ => panic!("Invalid str object in str_splitlines"),
        }
    }

    pub fn str_strip(&self, value: Option<&ValueRef>) -> ValueRef {
        let value = adjust_parameter(value);

        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let value = match value {
                    Some(chars) => {
                        let chars = chars.as_str();
                        let chars = chars.as_str();
                        v.trim_matches(|c| chars.contains(c))
                    }
                    None => v.trim(),
                };
                ValueRef::str(value)
            }
            _ => panic!("Invalid str object in str_strip"),
        }
    }

    pub fn str_title(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => {
                let mut title = String::with_capacity(v.len());
                let mut previous_is_cased = false;
                for c in v.chars() {
                    if c.is_lowercase() {
                        if !previous_is_cased {
                            title.extend(c.to_titlecase());
                        } else {
                            title.push(c);
                        }
                        previous_is_cased = true;
                    } else if c.is_uppercase() || c.is_titlecase() {
                        if previous_is_cased {
                            title.extend(c.to_lowercase());
                        } else {
                            title.push(c);
                        }
                        previous_is_cased = true;
                    } else {
                        previous_is_cased = false;
                        title.push(c);
                    }
                }
                ValueRef::str(title.as_str())
            }
            _ => panic!("Invalid str object in title"),
        }
    }

    pub fn str_equal(&self, value: &str) -> bool {
        match &*self.rc.borrow() {
            Value::str_value(ref v) => *v == *value,
            _ => false,
        }
    }
}
