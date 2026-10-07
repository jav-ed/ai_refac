import 'package:shop/shop.dart';

void main() {
  final figures = <Shape>[Rect(width: 2, height: 3), Circle(1), unitSquare()];
  for (final item in figures) {
    print(describe(item));
  }
  print(summary(figures));
  print(totalArea(figures, scale: 0.5).toStringAsFixed(2));
}
