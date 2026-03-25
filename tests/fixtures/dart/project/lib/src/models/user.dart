// Exercises: relative import of formatter from models/ subdirectory.
// Before: '../formatter.dart'  After: '../core/formatter.dart'
import '../formatter.dart';

class User {
  final String name;
  final double score;

  const User({required this.name, required this.score});

  String formattedScore() => Formatter().formatValue(score);
}
