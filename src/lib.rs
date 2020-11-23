//! Easily deserialize whitespace seperated data into any rust data structure supported by serde.
//! Useful for demos, programming contests, and the like.
//!
//! current issues:
//!  * no support for enums with struct variants
//!  * structs or tuples cannot contain an unbounded container, like a `Vec` or `HashMap`.
//!
//! future features:
//!  * defining custom whitespace characters
//!  * `scanf` style formatting for more complex inputs
//!
//! ## Example
//!
//! ```rust
//! extern crate serde;
//! extern crate serde_scan;
//!
//! #[macro_use]
//! extern crate serde_derive;
//!
//! #[derive(Deserialize, Debug, PartialEq)]
//! struct Triple {
//!     a: u32,
//!     b: u32,
//!     c: u32,
//! }
//!
//! #[derive(Deserialize, Debug, PartialEq)]
//! enum Command {
//!     Q,
//!     Help,
//!     Size(usize, usize),
//!     Color(u8),
//! }
//!
//! fn main() {
//!     let s = "1 2 3";
//!
//!     let a: [u32; 3] = serde_scan::from_str(s).unwrap();
//!     assert_eq!(a, [1, 2, 3]);
//!
//!     let b: (u32, u32, u32) = serde_scan::from_str(s).unwrap();
//!     assert_eq!(b, (1, 2, 3));
//!
//!     let c: Triple = serde_scan::from_str(s).unwrap();
//!     assert_eq!(c, Triple { a: 1, b: 2, c: 3 });
//!
//!     let s = "Size 1 2";
//!     let size: Command = serde_scan::from_str(s).unwrap();
//!     assert_eq!(size, Command::Size(1, 2));
//! }
//! ```

extern crate serde;

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate serde_derive;

mod de;

mod errors {
    use serde::de;
    use std::error::Error;
    use std::fmt::{self, Display};
    use std::io;

    // TODO: make this better

    #[derive(Debug)]
    pub enum ScanError {
        Io(io::Error),
        De,
        EOF,
        NS(&'static str),
    }

    impl From<io::Error> for ScanError {
        fn from(e: io::Error) -> Self {
            ScanError::Io(e)
        }
    }

    impl Display for ScanError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                ScanError::Io(ref e) => write!(f, "io: {}", e),
                ScanError::De => write!(f, "deserialization error"),
                ScanError::EOF => write!(f, "unexpected end of input"),
                ScanError::NS(val) => {
                    write!(f, "deseralizing `{}` is not supported at this time.", val)
                }
            }
        }
    }

    impl Error for ScanError {}

    impl de::Error for ScanError {
        fn custom<T: Display>(_msg: T) -> Self {
            ScanError::De
        }
    }
}

pub use errors::ScanError;

use serde::de::{Deserialize, DeserializeOwned};

/// Get a line of input from stdin, and parse it.
///
/// Extra data not needed for parsing `T` is thrown out.
///
pub fn next_line<T: DeserializeOwned>() -> Result<T, ScanError> {
    use std::io;

    let input = io::stdin();
    let mut buf = String::new();

    input.read_line(&mut buf)?;

    from_str(&buf)
}

/// Parse a string contaning whitespace seperated data.
///
pub fn from_str<'a, T: Deserialize<'a>>(s: &'a str) -> Result<T, ScanError> {
    let mut de = de::Deserializer::<fn(char) -> bool>::from_str(s);

    T::deserialize(&mut de)
}

/// Parse a string contaning data seperated by whitespace or any character in the given skip string.
///
pub fn from_str_skipping<'a, T: Deserialize<'a>>(set: &'a str, s: &'a str) -> Result<T, ScanError> {
    from_closure(|ch| ch.is_whitespace() || set.contains(ch), s)
}

#[doc(hidden)]
pub fn from_closure<'a, F, T>(f: F, s: &'a str) -> Result<T, ScanError>
where
    T: Deserialize<'a>,
    F: FnMut(char) -> bool,
{
    let mut de = de::Deserializer::from_closure(f, s);

    T::deserialize(&mut de)
}

/// The `scan!` macro.
///
/// Useful for extracting important bits from simple ad-hoc text files.
///
/// # Example
///
/// ```rust,no_run
/// # use serde_scan::scan;
/// # use serde_scan::ScanError;
///
/// # fn main() -> Result<(), ScanError> {
/// let line = "#1 @ 555,891: 18x12";
/// let parsed = scan!("#{} @ {},{}: {}x{}" <- line)?;
/// # Ok(()) }
/// ```
///
#[macro_export]
macro_rules! scan {
    ($scan_string:tt <- $input:ident) => {{
        let mut chaff = $scan_string.split("{}").flat_map(|s| s.chars()).peekable();

        $crate::from_closure(
            move |next_ch| {
                if let Some(&ch) = chaff.peek() {
                    if next_ch == ch || ch.is_whitespace() && next_ch.is_whitespace() {
                        chaff.next();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            $input,
        )
    }};
    ($($t:tt)*) => {
        compile_error!("invalid format.\nusage: scan!(\"scan literal\" <- value)");
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numbers() {
        let a: u64 = from_str("64").unwrap();
        let b: i64 = from_str("-64").unwrap();

        assert_eq!(a, 64);
        assert_eq!(b, -64);
    }

    #[test]
    fn tuples() {
        let a: (f32,) = from_str("  45.34 ").unwrap();
        let b: (u8, u8) = from_str("   3 4   ").unwrap();
        let c: (u32, String, u32) = from_str(" 413 plus 612 ").unwrap();

        assert_eq!(a.0, 45.34);
        assert_eq!(b, (3, 4));
        assert_eq!(c, (413, String::from("plus"), 612));
    }

    #[test]
    fn options() {
        let a: Result<u32, ScanError> = from_str("    ");
        let b: Option<u32> = from_str("   ").unwrap();
        let c: Option<u32> = from_str(" 7 ").unwrap();

        assert!(a.is_err());
        assert_eq!(b, None);
        assert_eq!(c, Some(7));
    }

    #[test]
    fn three_ways() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Triple {
            a: u32,
            b: u32,
            c: u32,
        }

        let s = r#" 1 
                2 
        3 "#;

        let a: [u32; 3] = from_str(s).unwrap();
        assert_eq!(a, [1, 2, 3]);

        let b: (u32, u32, u32) = from_str(s).unwrap();
        assert_eq!(b, (1, 2, 3));

        let c: Triple = from_str(s).unwrap();
        assert_eq!(c, Triple { a: 1, b: 2, c: 3 });
    }

    #[test]
    fn enums() {
        let color_list = r#"
            red
            blue
            green
            green
            red
            blue
        "#;

        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(rename_all = "snake_case")]
        enum Color {
            Red,
            Blue,
            Green,
        }

        let colors: Vec<Color> = from_str(color_list).unwrap();

        assert_eq!(colors.len(), 6);
        assert_eq!(colors[3], Color::Green);
    }

    #[test]
    fn enum_tuple() {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(rename_all = "snake_case")]
        enum EnumTuple {
            Variant(i32),
            Tuple(String, String, usize),
        }

        // this might work in the future
        let a: EnumTuple = from_str("variant 1").unwrap();
        let b: EnumTuple = from_str("tuple two three 4").unwrap();

        assert_eq!(a, EnumTuple::Variant(1));
        assert_eq!(
            b,
            EnumTuple::Tuple("two".to_string(), "three".to_string(), 4)
        );
    }

    #[test]
    fn byte_bufs() {
        // maybe: add support for 0x, 0o, 0b
        let bytes: Vec<u8> = from_str("0 1 2 255").unwrap();
        assert_eq!(bytes[0], 0x00);
        assert_eq!(bytes.len(), 4);

        let borrowed: Result<&[u8], _> = from_str("0 1 2 255");
        assert!(borrowed.is_err());
    }

    #[test]
    fn unsupported() {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(rename_all = "snake_case")]
        enum Bad {
            StructVariant { a: f64, b: f64 },
        }

        // this might work in the future
        let c: Result<Bad, _> = from_str("struct_variant 0.4 0.5");

        assert!(c.is_err());

        #[derive(Deserialize, Debug, PartialEq)]
        struct VecWithStuff {
            vec: Vec<u32>,
            stuff: String,
        }

        // this will work in the future
        let d: Result<VecWithStuff, _> = from_str("1 2 3 4 6 Stuff");
        assert!(d.is_err())
    }

    #[test]
    fn scan_macro() {
        let test = "Guard #64 is active.";

        let id: u32 = scan!("Guard #{} is active." <- test).unwrap_or(0);

        assert_eq!(id, 64);
    }

    #[test]
    fn scan_macro_enum() {
        #[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
        #[serde(rename_all = "lowercase")]
        enum Damage {
            Fire,
            Cold,
        }

        let tests = [
            ("1 fire damage", 1, Damage::Fire),
            ("2\tcold\tdamage", 2, Damage::Cold),
        ];

        for &(test, test_n, test_damage) in &tests {
            let (n, damage): (u32, Damage) = scan!("{} {} damage" <- test).expect(test);
            assert_eq!(n, test_n);
            assert_eq!(damage, test_damage);
        }
    }

    #[test]
    fn parse_asm() {
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(untagged)]
        enum Value {
            Lit(u8),
            Reg(char),
        }

        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(rename_all = "snake_case")]
        enum Instr {
            Add(Value, Value),
            Sub(Value, Value),
            Load(Value, Value),
        }

        let input = "
            load a 80
            load b 60
            add a b
            sub a 10
        ";

        let expected = vec![
            Instr::Load(Value::Reg('a'), Value::Lit(80)),
            Instr::Load(Value::Reg('b'), Value::Lit(60)),
            Instr::Add(Value::Reg('a'), Value::Reg('b')),
            Instr::Sub(Value::Reg('a'), Value::Lit(10)),
        ];

        let program: Vec<Instr> = input
            .trim()
            .lines()
            .filter_map(|l| from_str(l).ok())
            .collect();

        assert_eq!(program, expected)
    }
}
