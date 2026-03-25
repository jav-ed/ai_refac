// Exercises: package: import from test/ (must use package: style, not relative).
// show combinator must survive the rewrite.
import 'package:acme_utils/src/formatter.dart' show Formatter;

void main() {
  final f = Formatter();
  assert(f.formatValue(3.14159).isNotEmpty);
}
