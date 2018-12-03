extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_scan;

#[derive(Deserialize)]
struct Claim {
    id: u32,
    start: (u32, u32),
    dim: (u32, u32),
}

fn main() {
    let input = include_str!("scan.txt");

    for l in input.lines() {
        let c: Claim = scan!("#{} @ {},{}: {}x{}" <- l).unwrap();

        println!("claim no. {}. start: ({},{}), area: {}", c.id, c.start.0, c.start.1, c.dim.0 * c.dim.1);
    }
}