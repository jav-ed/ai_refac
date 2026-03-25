// Exercises: package: import in test/ — must be rewritten when formatter moves.
import 'package:acme_utils/src/formatter.dart';
import 'package:acme_utils/src/validator.dart';

void main() {
  final f = Formatter();
  final v = Validator();
  assert(v.validate(f.formatValue(1.0)));
}
