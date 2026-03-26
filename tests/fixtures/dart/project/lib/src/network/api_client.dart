// Exercises: relative import + as alias from network/ subdirectory.
import '../formatter.dart' as f;

class ApiClient {
  String formatResponse(double v) => f.Formatter().formatValue(v);
}
