use shop::shapes::{double, kind_of, Kind, Rect};
use shop::Shape;

#[test]
fn a_square_is_not_wide() {
    let rect = Rect::new(2.0, 2.0);
    assert_eq!(kind_of(&rect), Kind::Square);
    assert_eq!(rect.area(), 4.0);
    assert_eq!(double(2.0), 4.0);
}
