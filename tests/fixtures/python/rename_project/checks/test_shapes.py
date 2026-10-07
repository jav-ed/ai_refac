from shop.report import untyped
from shop.shapes import Rect, total_area


def test_total_area() -> None:
    assert total_area([Rect(1, 2), Rect(width=3, height=1)]) == 5


def test_untyped_receiver() -> None:
    assert untyped(Rect(2, 2)) == 4


if __name__ == "__main__":
    test_total_area()
    test_untyped_receiver()
    print("ok")
