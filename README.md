# serde_scan

easily deserialize whitespace seperated data into any rust data structure supported by serde. useful for demos, programming contests, and the like.

current issues:
 * no support for enums beyond basic c-style ones
 * structs or tuples cannot contain an unbounded container, like a `Vec` or `HashMap`.

future features:
 * defining custom whitespace characters
 * `scanf` style formatting for more complex inputs

## examples

```rust
    extern crate serde;
    extern crate serde_scan;
    
    #[macro_use]
    extern crate serde_derive;

    #[derive(Deserialize, Debug, PartialEq)]
    struct Triple {
        a: u32,
        b: u32,
        c: u32,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    enum Command {
        Q,
        Help,
        Size(usize, usize),
        Color(u8),
    }

    fn main() {
        let s = "1 2 3";

        let a: [u32; 3] = serde_scan::from_str(s).unwrap();
        assert_eq!(a, [1, 2, 3]);

        let b: (u32, u32, u32) = serde_scan::from_str(s).unwrap();
        assert_eq!(b, (1, 2, 3));

        let c: Triple = serde_scan::from_str(s).unwrap();
        assert_eq!(c, Triple { a: 1, b: 2, c: 3 });

        let s = "Size 1 2";
        let size = serde_scan::from_str(s).unwrap();
        assert_eq!(c, Command::Size(1, 2));
    }
```
