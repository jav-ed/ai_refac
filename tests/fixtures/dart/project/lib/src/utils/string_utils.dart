// Exercises: relative import of formatter from utils/ subdirectory.
import '../formatter.dart';

String formatString(String s) => Formatter().toJson({'value': s});
