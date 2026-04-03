//! SNBT (Stringified Named Binary Tag) formatting support.
//!
//! This module provides formatting implementations for NBT values
//! that output in the SNBT format used by Minecraft.

use crate::Value;
use std::fmt::{self, Display, Formatter, Write};

/// Determines if a string needs to be quoted in SNBT format.
fn needs_quoting(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }

    let first_char = s.chars().next();
    if let Some(c) = first_char
        && (c.is_ascii_digit() || c == '-' || c == '.' || c == '+')
    {
        return true;
    }

    s.chars().any(|c| {
        !matches!(c,
            '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | '-' | '.' | '+'
        )
    })
}

/// Chooses the appropriate quote character for a string.
fn choose_quote_char(s: &str) -> char {
    let has_double = s.contains('"');
    let has_single = s.contains('\'');

    if has_double && !has_single { '\'' } else { '"' }
}

/// Escapes special characters in a string for SNBT format.
fn escape_string(s: &str, quote_char: char) -> String {
    let mut result = String::with_capacity(s.len() + 10);

    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '"' if quote_char == '"' => result.push_str("\\\""),
            '\'' if quote_char == '\'' => result.push_str("\\'"),
            _ => result.push(c),
        }
    }

    result
}

/// Formats a string value for SNBT output.
#[must_use]
pub fn format_snbt_string(s: &str) -> String {
    if needs_quoting(s) {
        let quote_char = choose_quote_char(s);
        let escaped = escape_string(s, quote_char);
        format!("{quote_char}{escaped}{quote_char}")
    } else {
        s.to_string()
    }
}

fn write_indent(f: &mut dyn Write, level: usize) -> fmt::Result {
    for _ in 0..level {
        write!(f, "    ")?; // 4 spaces indentation
    }
    Ok(())
}

fn fmt_list(list: &[Value], f: &mut dyn Write, indent: Option<usize>) -> fmt::Result {
    if list.is_empty() {
        return write!(f, "[]");
    }

    write!(f, "[")?;

    if let Some(lvl) = indent {
        writeln!(f)?;
        let next_lvl = lvl + 1;
        for (i, value) in list.iter().enumerate() {
            write_indent(f, next_lvl)?;
            fmt_snbt_internal(value, f, Some(next_lvl))?;
            if i < list.len() - 1 {
                writeln!(f, ",")?;
            } else {
                writeln!(f)?;
            }
        }
        write_indent(f, lvl)?;
    } else {
        for (i, value) in list.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            fmt_snbt_internal(value, f, None)?;
        }
    }
    write!(f, "]")
}

fn fmt_compound(
    map: &indexmap::IndexMap<String, Value>,
    f: &mut dyn Write,
    indent: Option<usize>,
) -> fmt::Result {
    if map.is_empty() {
        return write!(f, "{{}}");
    }

    write!(f, "{{")?;

    if let Some(lvl) = indent {
        writeln!(f)?;
        let next_lvl = lvl + 1;
        for (i, (key, value)) in map.iter().enumerate() {
            write_indent(f, next_lvl)?;
            write!(f, "{}: ", format_snbt_string(key))?; // Note the space after colon
            fmt_snbt_internal(value, f, Some(next_lvl))?;
            if i < map.len() - 1 {
                writeln!(f, ",")?;
            } else {
                writeln!(f)?;
            }
        }
        write_indent(f, lvl)?;
    } else {
        for (i, (key, value)) in map.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}:", format_snbt_string(key))?;
            fmt_snbt_internal(value, f, None)?;
        }
    }
    write!(f, "}}")
}

// Helper function to handle the formatting logic
fn fmt_snbt_internal(val: &Value, f: &mut dyn Write, indent: Option<usize>) -> fmt::Result {
    match val {
        Value::Byte(v) => write!(f, "{v}b"),
        Value::Short(v) => write!(f, "{v}s"),
        Value::Int(v) => write!(f, "{v}"),
        Value::Long(v) => write!(f, "{v}L"),
        Value::Float(v) => {
            if v.is_nan() {
                write!(f, "NaNf")
            } else if v.is_infinite() {
                if v.is_sign_positive() {
                    write!(f, "Infinityf")
                } else {
                    write!(f, "-Infinityf")
                }
            } else {
                write!(f, "{v}f")
            }
        }
        Value::Double(v) => {
            if v.is_nan() {
                write!(f, "NaN")
            } else if v.is_infinite() {
                if v.is_sign_positive() {
                    write!(f, "Infinity")
                } else {
                    write!(f, "-Infinity")
                }
            } else {
                write!(f, "{v}")
            }
        }
        Value::String(s) => write!(f, "{}", format_snbt_string(s)),
        Value::ByteArray(arr) => {
            write!(f, "[B;")?;
            for (i, byte) in arr.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                    if indent.is_some() {
                        write!(f, " ")?;
                    } // Small space after comma in arrays
                }
                let signed_byte = i8::from_ne_bytes([*byte]);
                write!(f, "{signed_byte}b")?;
            }
            write!(f, "]")
        }
        Value::IntArray(arr) => {
            write!(f, "[I;")?;
            for (i, int) in arr.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                    if indent.is_some() {
                        write!(f, " ")?;
                    }
                }
                write!(f, "{int}")?;
            }
            write!(f, "]")
        }
        Value::LongArray(arr) => {
            write!(f, "[L;")?;
            for (i, long) in arr.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                    if indent.is_some() {
                        write!(f, " ")?;
                    }
                }
                write!(f, "{long}L")?;
            }
            write!(f, "]")
        }
        Value::List(list) => fmt_list(list, f, indent),
        Value::Compound(map) => fmt_compound(map, f, indent),
    }
}

impl Value {
    /// Returns the SNBT (Stringified NBT) representation of this value.
    /// This output is compact (no unnecessary whitespace).
    ///
    /// # Errors
    /// Returns an error if writing to the string buffer fails.
    pub fn to_snbt(&self) -> crate::Result<String> {
        let mut s = String::new();
        fmt_snbt_internal(self, &mut s, None).map_err(|e| crate::Error::Message(e.to_string()))?;
        Ok(s)
    }

    /// Returns the "pretty" SNBT representation of this value.
    /// This output includes newlines and 4-space indentation for better readability.
    ///
    /// # Errors
    /// Returns an error if writing to the string buffer fails.
    pub fn to_snbt_pretty(&self) -> crate::Result<String> {
        let mut s = String::new();
        fmt_snbt_internal(self, &mut s, Some(0))
            .map_err(|e| crate::Error::Message(e.to_string()))?;
        Ok(s)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt_snbt_internal(self, f, None)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            fmt_snbt_internal(self, f, Some(0))
        } else {
            fmt_snbt_internal(self, f, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_needs_quoting() {
        // Simple identifiers don't need quoting
        assert!(!needs_quoting("simple"));
        assert!(!needs_quoting("with_underscore"));
        assert!(!needs_quoting("with-dash"));
        assert!(!needs_quoting("with.dot"));
        assert!(!needs_quoting("with+plus"));
        assert!(!needs_quoting("MixedCase123"));

        // Strings starting with special chars need quoting
        assert!(needs_quoting("123abc"));
        assert!(needs_quoting("-negative"));
        assert!(needs_quoting(".dotstart"));
        assert!(needs_quoting("+positive"));

        // Strings with spaces or special chars need quoting
        assert!(needs_quoting("hello world"));
        assert!(needs_quoting("special!char"));
        assert!(needs_quoting(""));
    }

    #[test]
    fn test_choose_quote_char() {
        assert_eq!(choose_quote_char("no quotes"), '"');
        assert_eq!(choose_quote_char("has \"double\" quotes"), '\'');
        assert_eq!(choose_quote_char("has 'single' quotes"), '"');
        assert_eq!(choose_quote_char("has \"both\" 'quotes'"), '"');
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("simple", '"'), "simple");
        assert_eq!(escape_string("line\nbreak", '"'), "line\\nbreak");
        assert_eq!(escape_string("tab\there", '"'), "tab\\there");
        assert_eq!(escape_string("back\\slash", '"'), "back\\\\slash");
        assert_eq!(escape_string("quote\"here", '"'), "quote\\\"here");
        assert_eq!(escape_string("quote'here", '\''), "quote\\'here");
    }

    #[test]
    fn test_format_snbt_string() {
        assert_eq!(format_snbt_string("simple"), "simple");
        assert_eq!(format_snbt_string("hello world"), "\"hello world\"");
        assert_eq!(format_snbt_string("has \"quotes\""), "'has \"quotes\"'");
        assert_eq!(format_snbt_string("has 'quotes'"), "\"has 'quotes'\"");
    }

    #[test]
    fn test_pretty_printing() {
        let val = Value::Compound(IndexMap::from([
            ("key".into(), Value::String("value".into())),
            (
                "list".into(),
                Value::List(vec![Value::Int(1), Value::Int(2)]),
            ),
        ]));
        let pretty = val.to_snbt_pretty().unwrap();

        assert!(pretty.contains('\n'));
        assert!(pretty.contains("    "));
        assert!(pretty.contains("key: value"));
    }

    #[test]
    fn test_debug_alternate() {
        let val = Value::List(vec![Value::Int(1)]);
        let debug = format!("{val:#?}");
        assert!(debug.contains('\n'));
    }

    mod snbt_lists {
        use crate::Value;

        #[test]
        fn test_snbt_byte_array_nonempty() {
            let v = Value::ByteArray(vec![1, 2, 3, 127, 128]);
            assert_eq!(v.to_string(), "[B;1b,2b,3b,127b,-128b]");
        }

        #[test]
        fn test_snbt_byte_array_empty() {
            let v = Value::ByteArray(vec![]);
            assert_eq!(v.to_string(), "[B;]");
        }

        #[test]
        fn test_snbt_int_array_nonempty() {
            let v = Value::IntArray(vec![1, 2, 3, 2_147_483_647, -2_147_483_648]);
            assert_eq!(v.to_string(), "[I;1,2,3,2147483647,-2147483648]");
        }

        #[test]
        fn test_snbt_int_array_empty() {
            let v = Value::IntArray(vec![]);
            assert_eq!(v.to_string(), "[I;]");
        }

        #[test]
        fn test_snbt_long_array_nonempty() {
            let v = Value::LongArray(vec![1, 2, 3, 9_223_372_036_854_775_807]);
            assert_eq!(v.to_string(), "[L;1L,2L,3L,9223372036854775807L]");
        }

        #[test]
        fn test_snbt_long_array_empty() {
            let v = Value::LongArray(vec![]);
            assert_eq!(v.to_string(), "[L;]");
        }

        #[test]
        fn test_snbt_list_integers() {
            let v = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
            assert_eq!(v.to_string(), "[1,2,3]");
        }

        #[test]
        fn test_snbt_list_strings() {
            let v = Value::List(vec![
                Value::String("foo".into()),
                Value::String("bar".into()),
            ]);
            assert_eq!(v.to_string(), "[foo,bar]");
        }

        #[test]
        fn test_snbt_list_empty() {
            let v = Value::List(vec![]);
            assert_eq!(v.to_string(), "[]");
        }

        #[test]
        fn test_snbt_list_doubles() {
            let v = Value::List(vec![
                Value::Double(3.2),
                Value::Double(64.5),
                Value::Double(129.5),
            ]);
            assert_eq!(v.to_string(), "[3.2,64.5,129.5]");
        }
    }

    mod snbt_compound {
        use crate::Value;
        use indexmap::IndexMap;

        #[test]
        fn test_snbt_compound() {
            let v = Value::Compound(IndexMap::from([
                ("X".into(), Value::Int(3)),
                ("Y".into(), Value::Int(64)),
                ("Z".into(), Value::Int(129)),
            ]));
            assert_eq!(v.to_string(), "{X:3,Y:64,Z:129}");
        }

        #[test]
        fn test_snbt_empty_compound() {
            let v = Value::Compound(IndexMap::new());
            assert_eq!(v.to_string(), "{}");
        }

        #[test]
        fn test_snbt_quoted_compound() {
            let v = Value::Compound(IndexMap::from([
                ("simple key".into(), Value::Int(1)),
                ("123numeric".into(), Value::Int(2)),
            ]));
            assert_eq!(v.to_string(), r#"{"simple key":1,"123numeric":2}"#);
        }

        #[test]
        fn test_snbt_nested_compound() {
            let v = Value::Compound(IndexMap::from([(
                "egg".into(),
                Value::Compound(IndexMap::from([
                    ("name".into(), Value::String("Eggbert".into())),
                    ("value".into(), Value::Float(0.5)),
                ])),
            )]));
            assert_eq!(v.to_string(), "{egg:{name:Eggbert,value:0.5f}}");
        }

        #[test]
        fn test_snbt_complex_structure() {
            let v = Value::Compound(IndexMap::from([(
                "name".into(),
                Value::String("Bananrama".into()),
            )]));
            assert_eq!(v.to_string(), "{name:Bananrama}");
        }
    }

    mod snbt_primitives {
        use crate::Value;

        #[test]
        fn test_snbt_byte_positive() {
            assert_eq!(Value::Byte(42).to_string(), "42b");
        }

        #[test]
        fn test_snbt_byte_negative() {
            assert_eq!(Value::Byte(-128).to_string(), "-128b");
        }

        #[test]
        fn test_snbt_short_positive() {
            assert_eq!(Value::Short(1000).to_string(), "1000s");
        }

        #[test]
        fn test_snbt_short_negative() {
            assert_eq!(Value::Short(-32768).to_string(), "-32768s");
        }

        #[test]
        fn test_snbt_int_positive() {
            assert_eq!(Value::Int(123_456).to_string(), "123456");
        }

        #[test]
        fn test_snbt_int_negative() {
            assert_eq!(Value::Int(-2_147_483_648).to_string(), "-2147483648");
        }

        #[test]
        fn test_snbt_long_positive() {
            assert_eq!(
                Value::Long(9_223_372_036_854_775_807).to_string(),
                "9223372036854775807L"
            );
        }

        #[test]
        fn test_snbt_long_negative() {
            assert_eq!(Value::Long(-1).to_string(), "-1L");
        }

        #[test]
        fn test_snbt_float_positive() {
            assert_eq!(Value::Float(1.23).to_string(), "1.23f");
        }

        #[test]
        fn test_snbt_float_negative() {
            assert_eq!(Value::Float(-0.5).to_string(), "-0.5f");
        }

        #[test]
        fn test_snbt_double_positive() {
            assert_eq!(
                Value::Double(3.922_337_203_685_477_6).to_string(),
                "3.9223372036854776"
            );
        }

        #[test]
        fn test_snbt_double_negative() {
            assert_eq!(Value::Double(-1.5).to_string(), "-1.5");
        }

        #[test]
        fn test_snbt_precise_float() {
            assert_eq!(
                Value::Double(164.399_948_120_117_2).to_string(),
                "164.3999481201172"
            );
        }
    }

    mod snbt_strings {
        use crate::Value;

        #[test]
        fn test_snbt_empty_string() {
            let v = Value::String(String::new());
            assert_eq!(v.to_string(), r#""""#);
        }

        #[test]
        fn test_snbt_string_simple() {
            let v = Value::String("simple".into());
            assert_eq!(v.to_string(), "simple");
        }

        #[test]
        fn test_snbt_string_with_underscore() {
            let v = Value::String("with_underscore".into());
            assert_eq!(v.to_string(), "with_underscore");
        }

        #[test]
        fn test_snbt_string_with_dash() {
            let v = Value::String("with-dash".into());
            assert_eq!(v.to_string(), "with-dash");
        }

        #[test]
        fn test_snbt_string_with_dot() {
            let v = Value::String("with.dot".into());
            assert_eq!(v.to_string(), "with.dot");
        }

        #[test]
        fn test_snbt_string_mixed_case() {
            let v = Value::String("MixedCase123".into());
            assert_eq!(v.to_string(), "MixedCase123");
        }

        #[test]
        fn test_snbt_string_with_spaces() {
            let v = Value::String("hello world".into());
            assert_eq!(v.to_string(), r#""hello world""#);
        }

        #[test]
        fn test_snbt_string_leading_number() {
            let v = Value::String("123abc".into());
            assert_eq!(v.to_string(), r#""123abc""#);
        }

        #[test]
        fn test_snbt_string_special_chars() {
            let v = Value::String("special!@#$".into());
            assert_eq!(v.to_string(), r#""special!@#$""#);
        }

        #[test]
        fn test_snbt_string_containing_double_quotes() {
            let v = Value::String((r#"has "double" quotes"#).into());
            assert_eq!(v.to_string(), r#"'has "double" quotes'"#);
        }

        #[test]
        fn test_snbt_string_escape_newline() {
            let v = Value::String("line1\nline2".into());
            assert_eq!(v.to_string(), r#""line1\nline2""#);
        }

        #[test]
        fn test_snbt_string_escape_tab() {
            let v = Value::String("tab\there".into());
            assert_eq!(v.to_string(), r#""tab\there""#);
        }

        #[test]
        fn test_snbt_string_escape_backslash() {
            let v = Value::String((r"back\slash").into());
            assert_eq!(v.to_string(), r#""back\\slash""#);
        }

        #[test]
        fn test_snbt_string_containing_single_quote() {
            let v = Value::String("it's".into());
            assert_eq!(v.to_string(), r#""it's""#);
        }
    }

    mod snbt_special_numbers {
        use crate::Value;

        #[test]
        fn test_snbt_float_nan() {
            let v = Value::Float(f32::NAN);
            assert_eq!(v.to_string(), "NaNf");
        }

        #[test]
        fn test_snbt_double_nan() {
            let v = Value::Double(f64::NAN);
            assert_eq!(v.to_string(), "NaN");
        }

        #[test]
        fn test_snbt_float_infinity() {
            let v = Value::Float(f32::INFINITY);
            assert_eq!(v.to_string(), "Infinityf");
        }

        #[test]
        fn test_snbt_double_infinity() {
            let v = Value::Double(f64::INFINITY);
            assert_eq!(v.to_string(), "Infinity");
        }

        #[test]
        fn test_snbt_float_neg_infinity() {
            let v = Value::Float(f32::NEG_INFINITY);
            assert_eq!(v.to_string(), "-Infinityf");
        }

        #[test]
        fn test_snbt_double_neg_infinity() {
            let v = Value::Double(f64::NEG_INFINITY);
            assert_eq!(v.to_string(), "-Infinity");
        }
    }
}
