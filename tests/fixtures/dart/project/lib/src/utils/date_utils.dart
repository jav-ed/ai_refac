// Exercises: relative import of formatter from utils/ subdirectory.
import '../formatter.dart';

String formatDate(DateTime d) => Formatter().toJson({'date': d.toIso8601String()});
