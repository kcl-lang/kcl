//! Ref: https://github.com/RustPython/RustPython/blob/main/vm/src/format.rs
//!
//! Copyright The KCL Authors. All rights reserved.

use itertools::{Itertools, PeekingNext};
use std::cmp;
use std::fmt;
use std::str::FromStr;

use crate::*;

#[derive(Debug)]
pub enum Case {
    Lower,
    Upper,
}

/// If s represents a floating point value, trailing zeros and a possibly trailing
/// decimal point will be removed.
/// This function does NOT work with decimal commas.
fn remove_trailing_redundant_chars(s: String) -> String {
    if s.contains('.') {
        // only truncate floating point values
        let s = remove_trailing_zeros(s);
        remove_trailing_decimal_point(s)
    } else {
        s
    }
}

fn remove_trailing_zeros(s: String) -> String {
    let mut s = s;
    while s.ends_with('0') {
        s.pop();
    }
    s
}

fn remove_trailing_decimal_point(s: String) -> String {
    let mut s = s;
    if s.ends_with('.') {
        s.pop();
    }
    s
}

fn format_nan(case: Case) -> String {
    let nan = match case {
        Case::Lower => "nan",
        Case::Upper => "NAN",
    };

    nan.to_string()
}

fn format_inf(case: Case) -> String {
    let inf = match case {
        Case::Lower => "inf",
        Case::Upper => "INF",
    };

    inf.to_string()
}

pub fn format_fixed(precision: usize, magnitude: f64, case: Case) -> String {
    match magnitude {
        magnitude if magnitude.is_finite() => format!("{magnitude:.precision$}"),
        magnitude if magnitude.is_nan() => format_nan(case),
        magnitude if magnitude.is_infinite() => format_inf(case),
        _ => "".to_string(),
    }
}

pub fn is_integer(v: f64) -> bool {
    (v - v.round()).abs() < std::f64::EPSILON
}

pub fn float_to_string(value: f64) -> String {
    let lit = format!("{value:e}");
    if let Some(position) = lit.find('e') {
        let significand = &lit[..position];
        let exponent = &lit[position + 1..];
        let exponent = exponent.parse::<i32>().unwrap();
        if exponent < 16 && exponent > -5 {
            if is_integer(value) {
                format!("{value:.1?}")
            } else {
                value.to_string()
            }
        } else {
            format!("{significand}e{exponent:+#03}")
        }
    } else {
        value.to_string()
    }
}

pub fn format_general(precision: usize, magnitude: f64, case: Case) -> String {
    match magnitude {
        magnitude if magnitude.is_finite() => {
            let r_exp = format!("{:.*e}", precision.saturating_sub(1), magnitude);
            let mut parts = r_exp.splitn(2, 'e');
            let base = parts.next().unwrap();
            let exponent = parts.next().unwrap().parse::<i64>().unwrap();
            if exponent < -4 || exponent >= (precision as i64) {
                let e = match case {
                    Case::Lower => 'e',
                    Case::Upper => 'E',
                };

                let base = remove_trailing_redundant_chars(format!("{:.*}", precision + 1, base));
                format!("{base}{e}{exponent:+#03}")
            } else {
                let precision = (precision as i64) - 1 - exponent;
                let precision = precision as usize;
                remove_trailing_redundant_chars(format!("{magnitude:.precision$}"))
            }
        }
        magnitude if magnitude.is_nan() => format_nan(case),
        magnitude if magnitude.is_infinite() => format_inf(case),
        _ => "".to_string(),
    }
}

// Formats floats into Python style exponent notation, by first formatting in Rust style
// exponent notation (`1.0000e0`), then convert to Python style (`1.0000e+00`).
pub fn format_exponent(precision: usize, magnitude: f64, case: Case) -> String {
    match magnitude {
        magnitude if magnitude.is_finite() => {
            let r_exp = format!("{magnitude:.precision$e}");
            let mut parts = r_exp.splitn(2, 'e');
            let base = parts.next().unwrap();
            let exponent = parts.next().unwrap().parse::<i64>().unwrap();
            let e = match case {
                Case::Lower => 'e',
                Case::Upper => 'E',
            };
            format!("{base}{e}{exponent:+#03}")
        }
        magnitude if magnitude.is_nan() => format_nan(case),
        magnitude if magnitude.is_infinite() => format_inf(case),
        _ => "".to_string(),
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FormatPreconversor {
    Str,
    Repr,
    Ascii,
    Bytes,
}

impl FormatPreconversor {
    fn from_char(c: char) -> Option<FormatPreconversor> {
        match c {
            's' => Some(FormatPreconversor::Str),
            'r' => Some(FormatPreconversor::Repr),
            'a' => Some(FormatPreconversor::Ascii),
            'b' => Some(FormatPreconversor::Bytes),
            _ => None,
        }
    }

    fn from_string(text: &str) -> Option<FormatPreconversor> {
        let mut chars = text.chars();
        if chars.next() != Some('!') {
            return None;
        }

        FormatPreconversor::from_char(chars.next()?)
    }

    fn parse_and_consume(text: &str) -> (Option<FormatPreconversor>, &str) {
        let preconversor = FormatPreconversor::from_string(text);
        match preconversor {
            None => (None, text),
            Some(_) => {
                let mut chars = text.chars();
                chars.next(); // Consume the bang
                chars.next(); // Consume one r,s,a char
                (preconversor, chars.as_str())
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FormatAlign {
    Left,
    Right,
    AfterSign,
    Center,
}

impl FormatAlign {
    fn from_char(c: char) -> Option<FormatAlign> {
        match c {
            '<' => Some(FormatAlign::Left),
            '>' => Some(FormatAlign::Right),
            '=' => Some(FormatAlign::AfterSign),
            '^' => Some(FormatAlign::Center),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FormatSign {
    Plus,
    Minus,
    MinusOrSpace,
}

#[derive(Debug, PartialEq)]
enum FormatGrouping {
    Comma,
    Underscore,
}

#[derive(Debug, PartialEq)]
enum FormatType {
    String,
    Binary,
    Character,
    Decimal,
    Octal,
    HexLower,
    HexUpper,
    Number,
    ExponentLower,
    ExponentUpper,
    GeneralFormatLower,
    GeneralFormatUpper,
    FixedPointLower,
    FixedPointUpper,
    Percentage,
}

#[derive(Debug, PartialEq)]
pub(crate) struct FormatSpec {
    preconversor: Option<FormatPreconversor>,
    fill: Option<char>,
    align: Option<FormatAlign>,
    sign: Option<FormatSign>,
    alternate_form: bool,
    width: Option<usize>,
    grouping_option: Option<FormatGrouping>,
    precision: Option<usize>,
    format_type: Option<FormatType>,
}

pub(crate) fn get_num_digits(text: &str) -> usize {
    for (index, character) in text.char_indices() {
        if !character.is_ascii_digit() {
            return index;
        }
    }
    text.len()
}

fn parse_preconversor(text: &str) -> (Option<FormatPreconversor>, &str) {
    FormatPreconversor::parse_and_consume(text)
}

fn parse_align(text: &str) -> (Option<FormatAlign>, &str) {
    let mut chars = text.chars();
    let maybe_align = chars.next().and_then(FormatAlign::from_char);
    if maybe_align.is_some() {
        (maybe_align, chars.as_str())
    } else {
        (None, text)
    }
}

fn parse_fill_and_align(text: &str) -> (Option<char>, Option<FormatAlign>, &str) {
    let char_indices: Vec<(usize, char)> = text.char_indices().take(3).collect();
    if char_indices.is_empty() {
        (None, None, text)
    } else if char_indices.len() == 1 {
        let (maybe_align, remaining) = parse_align(text);
        (None, maybe_align, remaining)
    } else {
        let (maybe_align, remaining) = parse_align(&text[char_indices[1].0..]);
        if maybe_align.is_some() {
            (Some(char_indices[0].1), maybe_align, remaining)
        } else {
            let (only_align, only_align_remaining) = parse_align(text);
            (None, only_align, only_align_remaining)
        }
    }
}

fn parse_number(text: &str) -> Result<(Option<usize>, &str), &'static str> {
    let num_digits: usize = get_num_digits(text);
    if num_digits == 0 {
        return Ok((None, text));
    }
    if let Ok(num) = text[..num_digits].parse::<usize>() {
        Ok((Some(num), &text[num_digits..]))
    } else {
        // NOTE: this condition is different from CPython
        Err("Too many decimal digits in format string")
    }
}

fn parse_sign(text: &str) -> (Option<FormatSign>, &str) {
    let mut chars = text.chars();
    match chars.next() {
        Some('-') => (Some(FormatSign::Minus), chars.as_str()),
        Some('+') => (Some(FormatSign::Plus), chars.as_str()),
        Some(' ') => (Some(FormatSign::MinusOrSpace), chars.as_str()),
        _ => (None, text),
    }
}

fn parse_alternate_form(text: &str) -> (bool, &str) {
    let mut chars = text.chars();
    match chars.next() {
        Some('#') => (true, chars.as_str()),
        _ => (false, text),
    }
}

fn parse_zero(text: &str) -> (bool, &str) {
    let mut chars = text.chars();
    match chars.next() {
        Some('0') => (true, chars.as_str()),
        _ => (false, text),
    }
}

fn parse_precision(text: &str) -> Result<(Option<usize>, &str), &'static str> {
    let mut chars = text.chars();
    Ok(match chars.next() {
        Some('.') => {
            let (size, remaining) = parse_number(chars.as_str())?;
            if size.is_some() {
                (size, remaining)
            } else {
                (None, text)
            }
        }
        _ => (None, text),
    })
}

fn parse_grouping_option(text: &str) -> (Option<FormatGrouping>, &str) {
    let mut chars = text.chars();
    match chars.next() {
        Some('_') => (Some(FormatGrouping::Underscore), chars.as_str()),
        Some(',') => (Some(FormatGrouping::Comma), chars.as_str()),
        _ => (None, text),
    }
}

fn parse_format_type(text: &str) -> (Option<FormatType>, &str) {
    let mut chars = text.chars();
    match chars.next() {
        Some('s') => (Some(FormatType::String), chars.as_str()),
        Some('b') => (Some(FormatType::Binary), chars.as_str()),
        Some('c') => (Some(FormatType::Character), chars.as_str()),
        Some('d') => (Some(FormatType::Decimal), chars.as_str()),
        Some('o') => (Some(FormatType::Octal), chars.as_str()),
        Some('x') => (Some(FormatType::HexLower), chars.as_str()),
        Some('X') => (Some(FormatType::HexUpper), chars.as_str()),
        Some('e') => (Some(FormatType::ExponentLower), chars.as_str()),
        Some('E') => (Some(FormatType::ExponentUpper), chars.as_str()),
        Some('f') => (Some(FormatType::FixedPointLower), chars.as_str()),
        Some('F') => (Some(FormatType::FixedPointUpper), chars.as_str()),
        Some('g') => (Some(FormatType::GeneralFormatLower), chars.as_str()),
        Some('G') => (Some(FormatType::GeneralFormatUpper), chars.as_str()),
        Some('n') => (Some(FormatType::Number), chars.as_str()),
        Some('%') => (Some(FormatType::Percentage), chars.as_str()),
        _ => (None, text),
    }
}

fn parse_format_spec(text: &str) -> Result<FormatSpec, &'static str> {
    // get_integer in CPython
    let (preconversor, after_preconversor) = parse_preconversor(text);
    let (mut fill, mut align, after_align) = parse_fill_and_align(after_preconversor);
    let (sign, after_sign) = parse_sign(after_align);
    let (alternate_form, after_alternate_form) = parse_alternate_form(after_sign);
    let (zero, after_zero) = parse_zero(after_alternate_form);
    let (width, after_width) = parse_number(after_zero)?;
    let (grouping_option, after_grouping_option) = parse_grouping_option(after_width);
    let (precision, after_precision) = parse_precision(after_grouping_option)?;
    let (format_type, after_format_type) = parse_format_type(after_precision);
    if !after_format_type.is_empty() {
        return Err("Invalid format specifier");
    }

    if zero && fill.is_none() {
        fill.replace('0');
        align = align.or(Some(FormatAlign::AfterSign));
    }

    Ok(FormatSpec {
        preconversor,
        fill,
        align,
        sign,
        alternate_form,
        width,
        grouping_option,
        precision,
        format_type,
    })
}

impl FormatSpec {
    pub(crate) fn parse(text: &str) -> Result<FormatSpec, &'static str> {
        parse_format_spec(text)
    }

    fn compute_fill_string(fill_char: char, fill_chars_needed: i32) -> String {
        (0..fill_chars_needed)
            .map(|_| fill_char)
            .collect::<String>()
    }

    fn add_magnitude_separators_for_char(
        magnitude_string: String,
        interval: usize,
        separator: char,
    ) -> String {
        let mut result = String::new();

        // Don't add separators to the floating decimal point of numbers
        let mut parts = magnitude_string.splitn(2, '.');
        let magnitude_integer_string = parts.next().unwrap();
        let mut remaining: usize = magnitude_integer_string.len();
        for c in magnitude_integer_string.chars() {
            result.push(c);
            remaining -= 1;
            if remaining % interval == 0 && remaining > 0 {
                result.push(separator);
            }
        }
        if let Some(part) = parts.next() {
            result.push('.');
            result.push_str(part);
        }
        result
    }

    fn get_separator_interval(&self) -> usize {
        match self.format_type {
            Some(FormatType::Binary) => 4,
            Some(FormatType::Decimal) => 3,
            Some(FormatType::Octal) => 4,
            Some(FormatType::HexLower) => 4,
            Some(FormatType::HexUpper) => 4,
            Some(FormatType::Number) => 3,
            Some(FormatType::FixedPointLower) | Some(FormatType::FixedPointUpper) => 3,
            None => 3,
            _ => panic!("Separators only valid for numbers!"),
        }
    }

    fn add_magnitude_separators(&self, magnitude_string: String) -> String {
        match self.grouping_option {
            Some(FormatGrouping::Comma) => FormatSpec::add_magnitude_separators_for_char(
                magnitude_string,
                self.get_separator_interval(),
                ',',
            ),
            Some(FormatGrouping::Underscore) => FormatSpec::add_magnitude_separators_for_char(
                magnitude_string,
                self.get_separator_interval(),
                '_',
            ),
            None => magnitude_string,
        }
    }

    pub(crate) fn format_float(&self, num: f64) -> Result<String, &'static str> {
        let precision = self.precision.unwrap_or(6);
        let magnitude = num.abs();
        let raw_magnitude_string_result: Result<String, &'static str> = match self.format_type {
            Some(FormatType::FixedPointUpper) => {
                Ok(format_fixed(precision, magnitude, Case::Upper))
            }
            Some(FormatType::FixedPointLower) => {
                Ok(format_fixed(precision, magnitude, Case::Lower))
            }
            Some(FormatType::Decimal) => Err("Unknown format code 'd' for object of type 'float'"),
            Some(FormatType::Binary) => Err("Unknown format code 'b' for object of type 'float'"),
            Some(FormatType::Octal) => Err("Unknown format code 'o' for object of type 'float'"),
            Some(FormatType::HexLower) => Err("Unknown format code 'x' for object of type 'float'"),
            Some(FormatType::HexUpper) => Err("Unknown format code 'X' for object of type 'float'"),
            Some(FormatType::String) => Err("Unknown format code 's' for object of type 'float'"),
            Some(FormatType::Character) => {
                Err("Unknown format code 'c' for object of type 'float'")
            }
            Some(FormatType::Number) => {
                Err("Format code 'n' for object of type 'float' not implemented yet")
            }
            Some(FormatType::GeneralFormatUpper) => {
                let precision = if precision == 0 { 1 } else { precision };
                Ok(format_general(precision, magnitude, Case::Upper))
            }
            Some(FormatType::GeneralFormatLower) => {
                let precision = if precision == 0 { 1 } else { precision };
                Ok(format_general(precision, magnitude, Case::Lower))
            }
            Some(FormatType::ExponentUpper) => {
                Ok(format_exponent(precision, magnitude, Case::Upper))
            }
            Some(FormatType::ExponentLower) => {
                Ok(format_exponent(precision, magnitude, Case::Lower))
            }
            Some(FormatType::Percentage) => match magnitude {
                magnitude if magnitude.is_nan() => Ok("nan%".to_owned()),
                magnitude if magnitude.is_infinite() => Ok("inf%".to_owned()),
                _ => Ok(format!("{:.*}%", precision, magnitude * 100.0)),
            },
            None => match magnitude {
                magnitude if magnitude.is_nan() => Ok("nan".to_owned()),
                magnitude if magnitude.is_infinite() => Ok("inf".to_owned()),
                _ => Ok(float_to_string(magnitude)),
            },
        };

        let raw_magnitude_string = raw_magnitude_string_result?;
        let magnitude_string = self.add_magnitude_separators(raw_magnitude_string);
        let format_sign = self.sign.unwrap_or(FormatSign::Minus);
        let sign_str = if num.is_sign_negative() && !num.is_nan() {
            "-"
        } else {
            match format_sign {
                FormatSign::Plus => "+",
                FormatSign::Minus => "",
                FormatSign::MinusOrSpace => " ",
            }
        };

        self.format_sign_and_align(&magnitude_string, sign_str)
    }

    pub(crate) fn format_int(&self, num: &i64) -> Result<String, &'static str> {
        let magnitude = num.abs();
        let prefix = if self.alternate_form {
            match self.format_type {
                Some(FormatType::Binary) => "0b",
                Some(FormatType::Octal) => "0o",
                Some(FormatType::HexLower) => "0x",
                Some(FormatType::HexUpper) => "0x",
                _ => "",
            }
        } else {
            ""
        };
        let raw_magnitude_string_result: Result<String, &'static str> = match self.format_type {
            Some(FormatType::Binary) => Ok(format!("{magnitude:b}")),
            Some(FormatType::Decimal) => Ok(format!("{magnitude}")),
            Some(FormatType::Octal) => Ok(format!("{magnitude:o}")),
            Some(FormatType::HexLower) => Ok(format!("{magnitude:x}")),
            Some(FormatType::HexUpper) => {
                let mut result = format!("{magnitude:x}");
                result.make_ascii_uppercase();
                Ok(result)
            }
            Some(FormatType::Number) => Ok(format!("{magnitude}")),
            Some(FormatType::String) => Err("Unknown format code 's' for object of type 'int'"),
            Some(FormatType::Character) => Err("Unknown format code 'c' for object of type 'int'"),
            Some(FormatType::GeneralFormatUpper) => {
                Err("Unknown format code 'G' for object of type 'int'")
            }
            Some(FormatType::GeneralFormatLower) => {
                Err("Unknown format code 'g' for object of type 'int'")
            }
            Some(FormatType::FixedPointUpper)
            | Some(FormatType::FixedPointLower)
            | Some(FormatType::ExponentUpper)
            | Some(FormatType::ExponentLower)
            | Some(FormatType::Percentage) => self.format_float(*num as f64),
            None => Ok(magnitude.to_string()),
        };
        let raw_magnitude_string = raw_magnitude_string_result?;
        let magnitude_string = format!(
            "{}{}",
            prefix,
            self.add_magnitude_separators(raw_magnitude_string)
        );

        let format_sign = self.sign.unwrap_or(FormatSign::Minus);
        let sign_str = match num.signum() {
            -1 => "-",
            _ => match format_sign {
                FormatSign::Plus => "+",
                FormatSign::Minus => "",
                FormatSign::MinusOrSpace => " ",
            },
        };

        self.format_sign_and_align(&magnitude_string, sign_str)
    }

    fn format_sign_and_align(
        &self,
        magnitude_string: &str,
        sign_str: &str,
    ) -> Result<String, &'static str> {
        let align = self.align.unwrap_or(FormatAlign::Right);

        // Use the byte length as the string length since we're in ascii
        let num_chars = magnitude_string.len();
        let fill_char = self.fill.unwrap_or(' ');
        let fill_chars_needed: i32 = self.width.map_or(0, |w| {
            cmp::max(0, (w as i32) - (num_chars as i32) - (sign_str.len() as i32))
        });
        Ok(match align {
            FormatAlign::Left => format!(
                "{}{}{}",
                sign_str,
                magnitude_string,
                FormatSpec::compute_fill_string(fill_char, fill_chars_needed)
            ),
            FormatAlign::Right => format!(
                "{}{}{}",
                FormatSpec::compute_fill_string(fill_char, fill_chars_needed),
                sign_str,
                magnitude_string
            ),
            FormatAlign::AfterSign => format!(
                "{}{}{}",
                sign_str,
                FormatSpec::compute_fill_string(fill_char, fill_chars_needed),
                magnitude_string
            ),
            FormatAlign::Center => {
                let left_fill_chars_needed = fill_chars_needed / 2;
                let right_fill_chars_needed = fill_chars_needed - left_fill_chars_needed;
                let left_fill_string =
                    FormatSpec::compute_fill_string(fill_char, left_fill_chars_needed);
                let right_fill_string =
                    FormatSpec::compute_fill_string(fill_char, right_fill_chars_needed);
                format!("{left_fill_string}{sign_str}{magnitude_string}{right_fill_string}")
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum FormatParseError {
    UnmatchedBracket,
    MissingStartBracket,
    UnescapedStartBracketInLiteral,
    InvalidFormatSpecifier,
    EmptyAttribute,
    MissingRightBracket,
    InvalidCharacterAfterRightBracket,
}

impl FromStr for FormatSpec {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FormatSpec::parse(s)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum FieldNamePart {
    Attribute(String),
    Index(usize),
    StringIndex(String),
}

impl FieldNamePart {
    fn parse_part(
        chars: &mut impl PeekingNext<Item = char>,
    ) -> Result<Option<FieldNamePart>, FormatParseError> {
        chars
            .next()
            .map(|ch| match ch {
                '.' => {
                    let mut attribute = String::new();
                    for ch in chars.peeking_take_while(|ch| *ch != '.' && *ch != '[') {
                        attribute.push(ch);
                    }
                    if attribute.is_empty() {
                        Err(FormatParseError::EmptyAttribute)
                    } else {
                        Ok(FieldNamePart::Attribute(attribute))
                    }
                }
                '[' => {
                    let mut index = String::new();
                    for ch in chars {
                        if ch == ']' {
                            return if index.is_empty() {
                                Err(FormatParseError::EmptyAttribute)
                            } else if let Ok(index) = index.parse::<usize>() {
                                Ok(FieldNamePart::Index(index))
                            } else {
                                Ok(FieldNamePart::StringIndex(index))
                            };
                        }
                        index.push(ch);
                    }
                    Err(FormatParseError::MissingRightBracket)
                }
                _ => Err(FormatParseError::InvalidCharacterAfterRightBracket),
            })
            .transpose()
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum FieldType {
    Auto,
    Index(usize),
    Keyword(String),
}

#[derive(Debug, PartialEq)]
pub(crate) struct FieldName {
    pub field_type: FieldType,
    pub parts: Vec<FieldNamePart>,
}

impl FieldName {
    pub(crate) fn parse(text: &str) -> Result<FieldName, FormatParseError> {
        let mut chars = text.chars().peekable();
        let mut first = String::new();
        for ch in chars.peeking_take_while(|ch| *ch != '.' && *ch != '[') {
            first.push(ch);
        }

        let field_type = if first.is_empty() {
            FieldType::Auto
        } else if let Ok(index) = first.parse::<usize>() {
            FieldType::Index(index)
        } else {
            FieldType::Keyword(first)
        };

        let mut parts = Vec::new();
        while let Some(part) = FieldNamePart::parse_part(&mut chars)? {
            parts.push(part)
        }

        Ok(FieldName { field_type, parts })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum FormatPart {
    Field {
        field_name: String,
        format_spec: String,
    },
    Literal(String),
}

#[derive(Debug, PartialEq)]
pub(crate) struct FormatString {
    pub format_parts: Vec<FormatPart>,
}

impl FormatString {
    fn parse_literal_single(text: &str) -> Result<(char, &str), FormatParseError> {
        let mut chars = text.chars();
        // This should never be called with an empty str
        let first_char = chars.next().unwrap();
        // isn't this detectable only with bytes operation?
        if first_char == '{' || first_char == '}' {
            let maybe_next_char = chars.next();
            // if we see a bracket, it has to be escaped by doubling up to be in a literal
            return if maybe_next_char.is_none() || maybe_next_char.unwrap() != first_char {
                Err(FormatParseError::UnescapedStartBracketInLiteral)
            } else {
                Ok((first_char, chars.as_str()))
            };
        }
        Ok((first_char, chars.as_str()))
    }

    fn parse_literal(text: &str) -> Result<(FormatPart, &str), FormatParseError> {
        let mut cur_text = text;
        let mut result_string = String::new();
        while !cur_text.is_empty() {
            match FormatString::parse_literal_single(cur_text) {
                Ok((next_char, remaining)) => {
                    result_string.push(next_char);
                    cur_text = remaining;
                }
                Err(err) => {
                    return if !result_string.is_empty() {
                        Ok((FormatPart::Literal(result_string), cur_text))
                    } else {
                        Err(err)
                    };
                }
            }
        }
        Ok((FormatPart::Literal(result_string), ""))
    }

    fn parse_part_in_brackets(text: &str) -> Result<FormatPart, FormatParseError> {
        let parts: Vec<&str> = text.splitn(2, ':').collect();
        // before the comma is a keyword or arg index, after the comma is maybe a spec.
        let arg_part = parts[0];

        let format_spec = if parts.len() > 1 {
            parts[1].to_owned()
        } else {
            String::new()
        };

        // On parts[0] can still be the preconversor (!r, !s, !a)
        let parts: Vec<&str> = arg_part.splitn(2, '!').collect();
        // before the bang is a keyword or arg index, after the comma is maybe a conversor spec.
        let arg_part = parts[0];

        Ok(FormatPart::Field {
            field_name: arg_part.to_owned(),
            format_spec,
        })
    }

    fn parse_spec(text: &str) -> Result<(FormatPart, &str), FormatParseError> {
        let mut nested = false;
        let mut end_bracket_pos = None;
        let mut left = String::new();

        // There may be one layer nesting brackets in spec
        for (idx, c) in text.chars().enumerate() {
            if idx == 0 {
                if c != '{' {
                    return Err(FormatParseError::MissingStartBracket);
                }
            } else if c == '{' {
                if nested {
                    return Err(FormatParseError::InvalidFormatSpecifier);
                } else {
                    nested = true;
                    left.push(c);
                    continue;
                }
            } else if c == '}' {
                if nested {
                    nested = false;
                    left.push(c);
                    continue;
                } else {
                    end_bracket_pos = Some(idx);
                    break;
                }
            } else {
                left.push(c);
            }
        }
        if let Some(pos) = end_bracket_pos {
            let (_, right) = text.split_at(pos);
            let format_part = FormatString::parse_part_in_brackets(&left)?;
            Ok((format_part, &right[1..]))
        } else {
            Err(FormatParseError::UnmatchedBracket)
        }
    }

    fn format_internal(&self, args: &ValueRef, kwargs: &ValueRef) -> String {
        let mut final_string = String::new();
        let mut auto_argument_index = 0;
        for part in &self.format_parts {
            let result_string = match part {
                FormatPart::Field {
                    field_name,
                    format_spec,
                } => {
                    let FieldName {
                        field_type, parts, ..
                    } = FieldName::parse(field_name.as_str()).unwrap();
                    let mut argument = match field_type {
                        FieldType::Auto => {
                            auto_argument_index += 1;
                            args.arg_i(auto_argument_index - 1)
                                .expect("argument tuple index out of range")
                        }
                        FieldType::Index(index) => {
                            if auto_argument_index != 0 {
                                panic!("cannot switch from automatic field numbering to manual field specification");
                            }
                            args.arg_i(index)
                                .expect("argument tuple index out of range")
                        }
                        FieldType::Keyword(keyword) => kwargs
                            .dict_get_value(keyword.as_str())
                            .unwrap_or_else(|| panic!("keyword argument '{keyword}' not found"))
                            .clone(),
                    };
                    for name_part in parts {
                        match name_part {
                            // Load attr
                            FieldNamePart::Attribute(attr) => {
                                argument = argument.dict_get_value(attr.as_str()).unwrap().clone();
                            }
                            // List subscript
                            FieldNamePart::Index(index) => {
                                argument = argument.list_get(index as isize).unwrap().clone();
                            }
                            // Dict subscript
                            FieldNamePart::StringIndex(value) => {
                                argument = argument.dict_get_value(value.as_str()).unwrap().clone();
                            }
                        }
                    }
                    argument.to_string_with_spec(format_spec)
                }
                FormatPart::Literal(literal) => literal.clone(),
            };
            final_string.push_str(result_string.as_str());
        }
        final_string
    }

    pub(crate) fn format(&self, args: &ValueRef, kwargs: &ValueRef) -> String {
        self.format_internal(args, kwargs)
    }
}

pub(crate) trait FromTemplate<'a>: Sized {
    type Err;
    fn from_str(s: &'a str) -> Result<Self, Self::Err>;
}

impl<'a> FromTemplate<'a> for FormatString {
    type Err = FormatParseError;

    fn from_str(text: &'a str) -> Result<Self, Self::Err> {
        let mut cur_text: &str = text;
        let mut parts: Vec<FormatPart> = Vec::new();
        while !cur_text.is_empty() {
            // Try to parse both literals and bracketed format parts util we
            // run out of text
            cur_text = FormatString::parse_literal(cur_text)
                .or_else(|_| FormatString::parse_spec(cur_text))
                .map(|(part, new_text)| {
                    parts.push(part);
                    new_text
                })?;
        }
        Ok(FormatString {
            format_parts: parts,
        })
    }
}

/// Convert a runtime value to a quoted string e.g., abc -> 'abc'
pub fn value_to_quoted_string(value: &ValueRef) -> String {
    if value.is_str() {
        let value = value.as_str();
        quoted_string(&value)
    } else {
        value.to_string()
    }
}

/// Convert a Rust string to a quoted string e.g., abc -> 'abc'
pub fn quoted_string(value: &str) -> String {
    let has_double_quote = value.contains('\'');
    let has_single_quote = value.contains('\"');
    if !has_single_quote {
        format!("'{value}'")
    } else if !has_double_quote {
        format!("\"{value}\"")
    } else {
        format!("\"{}\"", value.replace('\"', "\\\""))
    }
}

impl fmt::Display for ValueRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self.rc.borrow() {
            Value::undefined => write!(f, "Undefined"),
            Value::none => write!(f, "None"),
            Value::bool_value(ref v) => {
                if *v {
                    write!(f, "True")
                } else {
                    write!(f, "False")
                }
            }
            Value::int_value(ref v) => write!(f, "{v}"),
            Value::float_value(ref v) => {
                let mut float_str = v.to_string();
                if !float_str.contains('.') {
                    float_str.push_str(".0");
                }
                write!(f, "{float_str}")
            }
            Value::unit_value(_, raw, unit) => {
                write!(f, "{raw}{unit}")
            }
            Value::str_value(ref v) => write!(f, "{v}"),
            Value::list_value(ref v) => {
                let values: Vec<String> = v.values.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", values.join(", "))
            }
            Value::dict_value(ref v) => {
                let values: Vec<String> = v
                    .values
                    .iter()
                    .map(|(k, v)| format!("{}: {}", quoted_string(k), value_to_quoted_string(v)))
                    .collect();
                write!(f, "{{{}}}", values.join(", "))
            }
            Value::schema_value(ref v) => {
                let values: Vec<String> = v
                    .config
                    .values
                    .iter()
                    .map(|(k, v)| format!("{}: {}", quoted_string(k), value_to_quoted_string(v)))
                    .collect();
                write!(f, "{{{}}}", values.join(", "))
            }
            Value::func_value(_) => write!(f, "function"),
        }
    }
}

impl ValueRef {
    /// to_string_with_spec e.g., "{:.0f}".format(1.0)
    pub fn to_string_with_spec(&self, spec: &str) -> String {
        match &*self.rc.borrow() {
            Value::int_value(ref v) => {
                match FormatSpec::parse(spec).and_then(|format_spec| format_spec.format_int(v)) {
                    Ok(string) => string,
                    Err(err) => panic!("{}", err),
                }
            }
            Value::float_value(ref v) => {
                match FormatSpec::parse(spec).and_then(|format_spec| format_spec.format_float(*v)) {
                    Ok(string) => string,
                    Err(err) => panic!("{}", err),
                }
            }
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod test_value_fmt {
    use crate::*;

    #[test]
    fn test_string_format() {
        let mut ctx = Context::new();
        let cases = [
            (r#""{} {}""#, r#"["Hello","World"]"#, "\"Hello World\""),
            (r#""{:.0f}""#, r#"[1.0]"#, "\"1\""),
            (r#""{:.1f} {:.0f}""#, r#"[1.0,2.0]"#, "\"1.0 2\""),
            (
                r#""0{0[0]}, 1{0[1]}, Hello{1[Hello]}""#,
                r#"[["0","1"],{ "Hello": "World" }]"#,
                "\"00, 11, HelloWorld\"",
            ),
        ];
        for (format_string, args, expected) in cases {
            let format_string = FormatString::from_str(format_string).unwrap();
            let args = ValueRef::from_json(&mut ctx, args).unwrap();
            let kwargs = ValueRef::dict(None);
            let result = format_string.format(&args, &kwargs);
            assert_eq!(&result, expected)
        }
    }
}
