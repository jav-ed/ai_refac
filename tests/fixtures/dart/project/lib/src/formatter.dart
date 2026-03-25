// THIS FILE IS THE MOVE TARGET in tests/dart_move.rs
// Move: lib/src/formatter.dart -> lib/src/core/formatter.dart

import 'dart:convert';

class Formatter {
  String toJson(Map<String, dynamic> data) {
    return jsonEncode(data);
  }

  String formatValue(double v) {
    return v.toStringAsFixed(2);
  }
}
