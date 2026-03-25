// Exercises: package: import + show combinator — URI updates, show combinator survives.
import 'package:acme_utils/src/formatter.dart' show Formatter;

class Item {
  final double price;

  const Item({required this.price});

  String formattedPrice() => Formatter().formatValue(price);
}
