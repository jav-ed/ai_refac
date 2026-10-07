use shop::report::lines;
use shop::{Rect, Shape};

fn main() {
    let rect = Rect::new(3.0, 4.0).scaled();
    println!("{}", rect.area());
    for line in lines(&[rect]) {
        println!("{line}");
    }
}
