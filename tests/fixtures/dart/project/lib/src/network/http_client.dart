// Exercises: package: import + dart:io SDK import.
// The dart:io import must NOT be rewritten; only the package: URI changes.
import 'dart:io';
import 'package:acme_utils/src/formatter.dart';

class HttpClient {
  final Formatter _formatter = Formatter();

  Future<String> get(Uri uri) async {
    final client = HttpClient();
    final request = await client.getUrl(uri);
    final response = await request.close();
    return _formatter.formatValue(response.statusCode.toDouble());
  }
}
