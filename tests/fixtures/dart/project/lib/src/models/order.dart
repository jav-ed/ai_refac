// Exercises: package: import of formatter — should be rewritten when formatter moves.
import 'package:acme_utils/src/formatter.dart';

class Order {
  final double amount;

  const Order({required this.amount});

  String formattedAmount() => Formatter().formatValue(amount);
}
