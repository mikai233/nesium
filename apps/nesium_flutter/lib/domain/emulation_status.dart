import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

enum EmulationOverlayMode { none, paused, rewinding, fastForwarding }

@immutable
class EmulationStatus {
  const EmulationStatus({
    required this.paused,
    required this.rewinding,
    required this.fastForwarding,
  });

  const EmulationStatus.idle()
    : paused = false,
      rewinding = false,
      fastForwarding = false;

  final bool paused;
  final bool rewinding;
  final bool fastForwarding;

  EmulationOverlayMode get overlayMode {
    if (rewinding) return EmulationOverlayMode.rewinding;
    if (fastForwarding) return EmulationOverlayMode.fastForwarding;
    if (paused) return EmulationOverlayMode.paused;
    return EmulationOverlayMode.none;
  }

  EmulationStatus copyWith({
    bool? paused,
    bool? rewinding,
    bool? fastForwarding,
  }) {
    return EmulationStatus(
      paused: paused ?? this.paused,
      rewinding: rewinding ?? this.rewinding,
      fastForwarding: fastForwarding ?? this.fastForwarding,
    );
  }
}

class EmulationStatusController extends Notifier<EmulationStatus> {
  @override
  EmulationStatus build() => const EmulationStatus.idle();

  void setPaused(bool value) {
    if (state.paused == value) return;
    state = state.copyWith(paused: value);
  }

  void setRewinding(bool value) {
    if (state.rewinding == value) return;
    state = state.copyWith(rewinding: value);
  }

  void setFastForwarding(bool value) {
    if (state.fastForwarding == value) return;
    state = state.copyWith(fastForwarding: value);
  }
}

final emulationStatusProvider =
    NotifierProvider<EmulationStatusController, EmulationStatus>(
      EmulationStatusController.new,
    );
