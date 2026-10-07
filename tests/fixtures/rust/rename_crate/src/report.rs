use crate::shapes::{kind_of, total_area, Kind, Rect, Shape};

macro_rules! describe {
    ($shape:expr) => {
        format!("area={}", $shape.area())
    };
}

pub fn lines(rects: &[Rect]) -> Vec<String> {
    let mut lines: Vec<String> = rects
        .iter()
        .map(|rect| {
            let kind = match kind_of(rect) {
                Kind::Square => "square",
                Kind::Wide => "wide",
            };
            format!("{} ({kind})", describe!(rect))
        })
        .collect();
    let shapes: Vec<&dyn Shape> = rects.iter().map(|rect| rect as &dyn Shape).collect();
    lines.push(format!("total={}", total_area(&shapes)));
    lines
}
