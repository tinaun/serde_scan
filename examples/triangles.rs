extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_scan;

#[derive(Debug, Deserialize)]
struct Triangle {
    a: u64,
    b: u64,
    c: u64,
}

impl Triangle {
    fn is_valid(&self) -> bool {
        self.a + self.b > self.c && self.a + self.c > self.b && self.b + self.c > self.a
    }
}

fn main() {
    let n: usize = serde_scan::next_line().unwrap();
    let mut valid = 0;

    for _ in 0..n {
        let t: Triangle = serde_scan::next_line().unwrap();

        if t.is_valid() {
            valid += 1;
        }
    }

    println!("{} out of {} triangles are valid.", valid, n);
}
