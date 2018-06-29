extern crate serde;

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate serde_derive;

mod de;

mod errors {
    use std::io;
    use std::error::Error;
    use std::fmt::{self, Display};
    use serde::de;

    // TODO: make this better


    #[derive(Debug)]
    pub enum ScanError {
        Io( io::Error ),
        De,
        EOF,
        NS(&'static str)
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
                ScanError::NS(val) => write!(f, "deseralizing `{}` is not supported at this time.", val),
            }
        }
    }

    impl Error for ScanError {
        fn description(&self) -> &str {
            match *self {
                ScanError::Io(ref e) => e.description(),
                ScanError::De => "deserialization error",
                ScanError::EOF => "unexpected end of input",
                ScanError::NS(_) => "unsupported format",
            }
        }
    }

    impl de::Error for ScanError {
        fn custom<T: Display>(_msg: T) -> Self {
            ScanError::De
        }
    }
}

use errors::*;

use serde::de::{Deserialize, DeserializeOwned};


/// get a line of input from stdin, and parse it. 
/// extra data not needed for parsing `T` is thrown out
pub fn next_line<T: DeserializeOwned>() -> Result<T, ScanError> {
    use std::io;
    
    let input = io::stdin();
    let mut buf = String::new();

    input.read_line(&mut buf)?;

    from_str(&buf)
}

/// 
pub fn from_str<'a, T: Deserialize<'a>>(s: &'a str) -> Result<T, ScanError> {
    let mut de = de::Deserializer::from_str(s);

    T::deserialize(&mut de)
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
        assert_eq!(b, EnumTuple::Tuple("two".to_string(), "three".to_string(), 4));
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
            StructVariant {
                a: f64,
                b: f64,
            },
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
}
