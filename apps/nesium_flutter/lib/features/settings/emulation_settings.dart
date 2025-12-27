import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/api/emulation.dart' as nes_emulation;
import '../../platform/platform_capabilities.dart';

@immutable
class EmulationSettings {
  const EmulationSettings({
    required this.integerFpsMode,
    required this.pauseInBackground,
  });

  final bool integerFpsMode;
  final bool pauseInBackground;

  EmulationSettings copyWith({bool? integerFpsMode, bool? pauseInBackground}) {
    return EmulationSettings(
      integerFpsMode: integerFpsMode ?? this.integerFpsMode,
      pauseInBackground: pauseInBackground ?? this.pauseInBackground,
    );
  }

  static EmulationSettings defaults() {
    return EmulationSettings(
      integerFpsMode: false,
      pauseInBackground: isNativeMobile,
    );
  }
}

class EmulationSettingsController extends Notifier<EmulationSettings> {
  @override
  EmulationSettings build() => EmulationSettings.defaults();

  void setIntegerFpsMode(bool enabled) {
    if (enabled == state.integerFpsMode) return;
    state = state.copyWith(integerFpsMode: enabled);
    nes_emulation.setIntegerFpsMode(enabled: enabled).catchError((_) {});
  }

  void setPauseInBackground(bool enabled) {
    if (enabled == state.pauseInBackground) return;
    state = state.copyWith(pauseInBackground: enabled);
  }
}

final emulationSettingsProvider =
    NotifierProvider<EmulationSettingsController, EmulationSettings>(
      EmulationSettingsController.new,
    );
