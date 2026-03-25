// Exercises: relative import + aliased import (as fmt).
import 'dart:io';
import 'formatter.dart' as fmt;

class Service {
  final fmt.Formatter _formatter = fmt.Formatter();

  Future<String> process(Map<String, dynamic> data) async {
    return _formatter.toJson(data);
  }
}
