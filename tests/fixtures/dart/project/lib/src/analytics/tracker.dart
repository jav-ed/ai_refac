// Exercises: package: import — should be rewritten when formatter moves.
import 'package:acme_utils/src/formatter.dart';

class Tracker {
  final Formatter _formatter = Formatter();

  String track(double value) => _formatter.formatValue(value);
}
