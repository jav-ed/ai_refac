import shop.shapes as shapes
from shop.shapes import Rect, Shape, total_area


def describe(shape: Shape) -> str:
    return f"{type(shape).__name__} with area {shape.area():.2f}"


def summary(items: list[Shape]) -> str:
    return f"total {total_area(items, scale=2.0):.2f} over {len(items)} shapes"


def untyped(thing):
    # No annotation: the server cannot know that `thing` is a Shape.
    return thing.area()


def unit() -> float:
    return shapes.UNIT


def unit_square() -> Rect:
    return Rect(width=unit(), height=unit())
