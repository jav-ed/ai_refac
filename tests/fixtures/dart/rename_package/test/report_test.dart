import 'package:shop/shop.dart';

void main() {
  final area = totalArea([Rect(width: 1, height: 2), Rect(width: 3, height: 1)]);
  if (area != 5) {
    throw StateError('total area was $area');
  }
  if (untyped(Rect(width: 2, height: 2)) != '4.0') {
    throw StateError('untyped receiver broke');
  }
  print('ok');
}
