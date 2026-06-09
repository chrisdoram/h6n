use h6n::{Flat, Hex, Point};
fn main() {
    let point: Point = (1, 0).into();
    let hex: Hex<Flat> = point.into();
    println!("{:?}", hex);
}
