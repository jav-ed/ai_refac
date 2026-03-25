// Exercises: relative import of formatter from utils/ subdirectory.
// Before: '../formatter.dart'  After: '../core/formatter.dart'
import '../formatter.dart';

String formatDate(DateTime d) => Formatter().toJson({'date': d.toIso8601String()});
