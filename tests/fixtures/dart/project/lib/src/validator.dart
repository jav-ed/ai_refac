// Exercises: package: import of formatter — should be rewritten when formatter moves.
import 'package:acme_utils/src/formatter.dart';

class Validator {
  final Formatter _formatter = Formatter();

  bool validate(String input) => input.isNotEmpty;

  String format(Map<String, dynamic> data) => _formatter.toJson(data);
}
