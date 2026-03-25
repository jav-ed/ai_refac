// Exercises: package: import in test/ — must be rewritten when formatter moves.
import 'package:acme_utils/src/formatter.dart';
import 'package:acme_utils/src/service.dart';

void main() {
  final f = Formatter();
  final s = Service();
  assert(f.formatValue(0.0).isNotEmpty);
  assert(s != null);
}
