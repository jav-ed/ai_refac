// Control file: no dependency on formatter.dart — must be byte-identical after move.
class Config {
  final String host;
  final int port;

  const Config({this.host = 'localhost', this.port = 8080});
}
