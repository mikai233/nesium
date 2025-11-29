import 'package:flutter_riverpod/flutter_riverpod.dart';

enum NesPane { console, debugger, tools }

class NesPaneController extends Notifier<NesPane> {
  @override
  NesPane build() => NesPane.console;
}

final selectedPaneProvider = NotifierProvider<NesPaneController, NesPane>(
  NesPaneController.new,
);
