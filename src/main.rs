use h6n::{FLAT, Hex, Point};
fn main() {
    let point: Point = (1, 0).into();
    let hex: Hex<FLAT> = point.into();
    println!("{:?}", hex);
}
