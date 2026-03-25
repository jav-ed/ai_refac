// Exercises: relative import + as alias from network/ subdirectory.
// Before: '../formatter.dart' as f   After: '../core/formatter.dart' as f
import '../formatter.dart' as f;

class ApiClient {
  String formatResponse(double v) => f.Formatter().formatValue(v);
}
