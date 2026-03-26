// Exercises: relative import of formatter from a subdirectory of lib/src/.
import '../formatter.dart';

class Cache {
  final _store = <String, String>{};

  void put(String key, double value) {
    _store[key] = Formatter().formatValue(value);
  }

  String? get(String key) => _store[key];
}
