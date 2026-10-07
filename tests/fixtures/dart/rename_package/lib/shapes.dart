import 'dart:math' as math;

const double unit = 1.0;

/// Anything with an area; see [Shape.area].
abstract class Shape {
  double area();
}

class Rect extends Shape {
  Rect({required this.width, required this.height});

  final double width;
  final double height;

  @override
  double area() => width * height * unit;
}

class Circle extends Shape {
  Circle(this.radius);

  final double radius;

  @override
  double area() => math.pi * radius * radius;
}

double totalArea(List<Shape> shapes, {double scale = 1.0}) {
  var total = 0.0;
  for (final shape in shapes) {
    total += shape.area();
  }
  return total * scale;
}

Shape largest(List<Shape> shapes) {
  var total = shapes.first;
  for (final shape in shapes) {
    if (shape.area() > total.area()) {
      total = shape;
    }
  }
  return total;
}
