import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/api/emulation.dart' as nes_emulation;

@immutable
class EmulationSettings {
  const EmulationSettings({required this.integerFpsMode});

  final bool integerFpsMode;

  EmulationSettings copyWith({bool? integerFpsMode}) {
    return EmulationSettings(
      integerFpsMode: integerFpsMode ?? this.integerFpsMode,
    );
  }

  static const defaults = EmulationSettings(integerFpsMode: false);
}

class EmulationSettingsController extends Notifier<EmulationSettings> {
  @override
  EmulationSettings build() => EmulationSettings.defaults;

  void setIntegerFpsMode(bool enabled) {
    if (enabled == state.integerFpsMode) return;
    state = state.copyWith(integerFpsMode: enabled);
    nes_emulation.setIntegerFpsMode(enabled: enabled).catchError((_) {});
  }
}

final emulationSettingsProvider =
    NotifierProvider<EmulationSettingsController, EmulationSettings>(
      EmulationSettingsController.new,
    );
