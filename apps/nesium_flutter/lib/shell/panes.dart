import 'package:flutter_riverpod/flutter_riverpod.dart';

enum NesPane { console, debugger, tools }

final selectedPaneProvider = StateProvider<NesPane>((ref) {
  return NesPane.console;
});
