import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/api/input.dart' as nes_input;
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';

@immutable
class TurboSettings {
  const TurboSettings({
    required this.onFrames,
    required this.offFrames,
    required this.linked,
  });

  final int onFrames;
  final int offFrames;
  final bool linked;

  TurboSettings copyWith({int? onFrames, int? offFrames, bool? linked}) =>
      TurboSettings(
        onFrames: onFrames ?? this.onFrames,
        offFrames: offFrames ?? this.offFrames,
        linked: linked ?? this.linked,
      );

  static const TurboSettings defaults = TurboSettings(
    onFrames: 2,
    offFrames: 2,
    linked: true,
  );
}

class TurboSettingsController extends Notifier<TurboSettings> {
  @override
  TurboSettings build() {
    final defaults = TurboSettings.defaults;
    final loaded = _turboFromStorage(
      ref.read(appStorageProvider).get(StorageKeys.settingsTurbo),
      defaults: defaults,
    );
    final settings = loaded ?? defaults;

    // Best-effort apply; callers may invoke before the runtime is initialized.
    unawaited(
      nes_input
          .setTurboTiming(
            onFrames: settings.onFrames,
            offFrames: settings.offFrames,
          )
          .catchError((_) {}),
    );

    return settings;
  }

  void setOnFrames(int value) {
    final next = value.clamp(1, 255);
    if (next == state.onFrames) return;
    state = state.copyWith(
      onFrames: next,
      offFrames: state.linked ? next : null,
    );
    _persistAndApply(state);
  }

  void setOffFrames(int value) {
    final next = value.clamp(1, 255);
    if (next == state.offFrames) return;
    state = state.copyWith(
      offFrames: next,
      onFrames: state.linked ? next : null,
    );
    _persistAndApply(state);
  }

  void setLinked(bool value) {
    if (value == state.linked) return;
    state = state.copyWith(linked: value);
    if (value) {
      state = state.copyWith(offFrames: state.onFrames);
    }
    _persistAndApply(state);
  }

  void _persistAndApply(TurboSettings value) {
    unawaited(
      Future.wait([
        ref
            .read(appStorageProvider)
            .put(StorageKeys.settingsTurbo, _turboToStorage(value))
            .catchError((_) {}),
        nes_input
            .setTurboTiming(
              onFrames: value.onFrames,
              offFrames: value.offFrames,
            )
            .catchError((_) {}),
      ]),
    );
  }
}

final turboSettingsProvider =
    NotifierProvider<TurboSettingsController, TurboSettings>(
      TurboSettingsController.new,
    );

Map<String, Object?> _turboToStorage(TurboSettings value) => <String, Object?>{
  'onFrames': value.onFrames,
  'offFrames': value.offFrames,
  'linked': value.linked,
};

TurboSettings? _turboFromStorage(
  Object? value, {
  required TurboSettings defaults,
}) {
  if (value is! Map) return null;
  final map = value.cast<String, Object?>();

  int i(Object? v, int fallback) => v is num ? v.toInt() : fallback;
  bool b(Object? v, bool fallback) => v is bool ? v : fallback;

  final onFrames = i(map['onFrames'], defaults.onFrames).clamp(1, 255);
  final offFrames = i(map['offFrames'], defaults.offFrames).clamp(1, 255);
  final linked = b(map['linked'], defaults.linked);
  return defaults.copyWith(
    onFrames: onFrames,
    offFrames: offFrames,
    linked: linked,
  );
}
