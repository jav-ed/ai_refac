"""Shapes with an area."""

import math
from dataclasses import dataclass

UNIT = 1.0


class Shape:
    """Anything with an area."""

    def area(self) -> float:
        raise NotImplementedError


@dataclass
class Rect(Shape):
    width: float
    height: float

    def area(self) -> float:
        return self.width * self.height * UNIT


@dataclass
class Circle(Shape):
    radius: float

    def area(self) -> float:
        return math.pi * self.radius**2


def total_area(shapes: list[Shape], scale: float = 1.0) -> float:
    total = 0.0
    for shape in shapes:
        total += shape.area()
    return total * scale


def largest(shapes: list[Shape]) -> Shape:
    total = max(shapes, key=lambda shape: shape.area())
    return total
