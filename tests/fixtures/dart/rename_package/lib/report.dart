import 'package:shop/shapes.dart';

String describe(Shape shape) =>
    '${shape.runtimeType} with area ${shape.area().toStringAsFixed(2)}';

String summary(List<Shape> items) =>
    'total ${totalArea(items, scale: 2.0).toStringAsFixed(2)} over ${items.length} shapes';

String untyped(dynamic thing) => '${thing.area()}';

Rect unitSquare() => Rect(width: unit, height: unit);
