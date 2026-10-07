from shop import Circle, Rect, total_area
from shop.report import describe, summary, unit_square

figures = [Rect(2, 3), Circle(1), unit_square()]
for item in figures:
    print(describe(item))
print(summary(figures))
print(f"{total_area(figures, scale=0.5):.2f}")
