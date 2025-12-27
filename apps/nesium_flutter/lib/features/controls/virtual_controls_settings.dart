import 'dart:async';
import 'dart:ui';

import 'package:nesium_flutter/bridge/api/input.dart' as nes_input;
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';

@immutable
class VirtualControlsSettings {
  const VirtualControlsSettings({
    required this.buttonSize,
    required this.gap,
    required this.opacity,
    required this.hitboxScale,
    required this.hapticsEnabled,
    required this.dpadDeadzoneRatio,
    required this.turboFramesPerToggle,
    required this.portraitDpadOffset,
    required this.portraitButtonsOffset,
    required this.portraitSystemOffset,
    required this.portraitAOffset,
    required this.portraitBOffset,
    required this.portraitTurboAOffset,
    required this.portraitTurboBOffset,
    required this.portraitSelectOffset,
    required this.portraitStartOffset,
    required this.landscapeDpadOffset,
    required this.landscapeButtonsOffset,
    required this.landscapeSystemOffset,
    required this.landscapeAOffset,
    required this.landscapeBOffset,
    required this.landscapeTurboAOffset,
    required this.landscapeTurboBOffset,
    required this.landscapeSelectOffset,
    required this.landscapeStartOffset,
    required this.portraitDpadScale,
    required this.portraitButtonsScale,
    required this.portraitSystemScale,
    required this.portraitAScale,
    required this.portraitBScale,
    required this.portraitTurboAScale,
    required this.portraitTurboBScale,
    required this.portraitSelectScale,
    required this.portraitStartScale,
    required this.landscapeDpadScale,
    required this.landscapeButtonsScale,
    required this.landscapeSystemScale,
    required this.landscapeAScale,
    required this.landscapeBScale,
    required this.landscapeTurboAScale,
    required this.landscapeTurboBScale,
    required this.landscapeSelectScale,
    required this.landscapeStartScale,
  });

  final double buttonSize;
  final double gap;
  final double opacity;
  final double hitboxScale;
  final bool hapticsEnabled;
  final double dpadDeadzoneRatio;
  final int turboFramesPerToggle;

  final Offset portraitDpadOffset;
  final Offset portraitButtonsOffset;
  final Offset portraitSystemOffset;
  final Offset portraitAOffset;
  final Offset portraitBOffset;
  final Offset portraitTurboAOffset;
  final Offset portraitTurboBOffset;
  final Offset portraitSelectOffset;
  final Offset portraitStartOffset;
  final Offset landscapeDpadOffset;
  final Offset landscapeButtonsOffset;
  final Offset landscapeSystemOffset;
  final Offset landscapeAOffset;
  final Offset landscapeBOffset;
  final Offset landscapeTurboAOffset;
  final Offset landscapeTurboBOffset;
  final Offset landscapeSelectOffset;
  final Offset landscapeStartOffset;

  final double portraitDpadScale;
  final double portraitButtonsScale;
  final double portraitSystemScale;
  final double portraitAScale;
  final double portraitBScale;
  final double portraitTurboAScale;
  final double portraitTurboBScale;
  final double portraitSelectScale;
  final double portraitStartScale;
  final double landscapeDpadScale;
  final double landscapeButtonsScale;
  final double landscapeSystemScale;
  final double landscapeAScale;
  final double landscapeBScale;
  final double landscapeTurboAScale;
  final double landscapeTurboBScale;
  final double landscapeSelectScale;
  final double landscapeStartScale;

  VirtualControlsSettings copyWith({
    double? buttonSize,
    double? gap,
    double? opacity,
    double? hitboxScale,
    bool? hapticsEnabled,
    double? dpadDeadzoneRatio,
    int? turboFramesPerToggle,
    Offset? portraitDpadOffset,
    Offset? portraitButtonsOffset,
    Offset? portraitSystemOffset,
    Offset? portraitAOffset,
    Offset? portraitBOffset,
    Offset? portraitTurboAOffset,
    Offset? portraitTurboBOffset,
    Offset? portraitSelectOffset,
    Offset? portraitStartOffset,
    Offset? landscapeDpadOffset,
    Offset? landscapeButtonsOffset,
    Offset? landscapeSystemOffset,
    Offset? landscapeAOffset,
    Offset? landscapeBOffset,
    Offset? landscapeTurboAOffset,
    Offset? landscapeTurboBOffset,
    Offset? landscapeSelectOffset,
    Offset? landscapeStartOffset,
    double? portraitDpadScale,
    double? portraitButtonsScale,
    double? portraitSystemScale,
    double? portraitAScale,
    double? portraitBScale,
    double? portraitTurboAScale,
    double? portraitTurboBScale,
    double? portraitSelectScale,
    double? portraitStartScale,
    double? landscapeDpadScale,
    double? landscapeButtonsScale,
    double? landscapeSystemScale,
    double? landscapeAScale,
    double? landscapeBScale,
    double? landscapeTurboAScale,
    double? landscapeTurboBScale,
    double? landscapeSelectScale,
    double? landscapeStartScale,
  }) {
    return VirtualControlsSettings(
      buttonSize: buttonSize ?? this.buttonSize,
      gap: gap ?? this.gap,
      opacity: opacity ?? this.opacity,
      hitboxScale: hitboxScale ?? this.hitboxScale,
      hapticsEnabled: hapticsEnabled ?? this.hapticsEnabled,
      dpadDeadzoneRatio: dpadDeadzoneRatio ?? this.dpadDeadzoneRatio,
      turboFramesPerToggle: turboFramesPerToggle ?? this.turboFramesPerToggle,
      portraitDpadOffset: portraitDpadOffset ?? this.portraitDpadOffset,
      portraitButtonsOffset:
          portraitButtonsOffset ?? this.portraitButtonsOffset,
      portraitSystemOffset: portraitSystemOffset ?? this.portraitSystemOffset,
      portraitAOffset: portraitAOffset ?? this.portraitAOffset,
      portraitBOffset: portraitBOffset ?? this.portraitBOffset,
      portraitTurboAOffset: portraitTurboAOffset ?? this.portraitTurboAOffset,
      portraitTurboBOffset: portraitTurboBOffset ?? this.portraitTurboBOffset,
      portraitSelectOffset: portraitSelectOffset ?? this.portraitSelectOffset,
      portraitStartOffset: portraitStartOffset ?? this.portraitStartOffset,
      landscapeDpadOffset: landscapeDpadOffset ?? this.landscapeDpadOffset,
      landscapeButtonsOffset:
          landscapeButtonsOffset ?? this.landscapeButtonsOffset,
      landscapeSystemOffset:
          landscapeSystemOffset ?? this.landscapeSystemOffset,
      landscapeAOffset: landscapeAOffset ?? this.landscapeAOffset,
      landscapeBOffset: landscapeBOffset ?? this.landscapeBOffset,
      landscapeTurboAOffset:
          landscapeTurboAOffset ?? this.landscapeTurboAOffset,
      landscapeTurboBOffset:
          landscapeTurboBOffset ?? this.landscapeTurboBOffset,
      landscapeSelectOffset:
          landscapeSelectOffset ?? this.landscapeSelectOffset,
      landscapeStartOffset: landscapeStartOffset ?? this.landscapeStartOffset,
      portraitDpadScale: portraitDpadScale ?? this.portraitDpadScale,
      portraitButtonsScale: portraitButtonsScale ?? this.portraitButtonsScale,
      portraitSystemScale: portraitSystemScale ?? this.portraitSystemScale,
      portraitAScale: portraitAScale ?? this.portraitAScale,
      portraitBScale: portraitBScale ?? this.portraitBScale,
      portraitTurboAScale: portraitTurboAScale ?? this.portraitTurboAScale,
      portraitTurboBScale: portraitTurboBScale ?? this.portraitTurboBScale,
      portraitSelectScale: portraitSelectScale ?? this.portraitSelectScale,
      portraitStartScale: portraitStartScale ?? this.portraitStartScale,
      landscapeDpadScale: landscapeDpadScale ?? this.landscapeDpadScale,
      landscapeButtonsScale:
          landscapeButtonsScale ?? this.landscapeButtonsScale,
      landscapeSystemScale: landscapeSystemScale ?? this.landscapeSystemScale,
      landscapeAScale: landscapeAScale ?? this.landscapeAScale,
      landscapeBScale: landscapeBScale ?? this.landscapeBScale,
      landscapeTurboAScale: landscapeTurboAScale ?? this.landscapeTurboAScale,
      landscapeTurboBScale: landscapeTurboBScale ?? this.landscapeTurboBScale,
      landscapeSelectScale: landscapeSelectScale ?? this.landscapeSelectScale,
      landscapeStartScale: landscapeStartScale ?? this.landscapeStartScale,
    );
  }

  static const defaults = VirtualControlsSettings(
    buttonSize: 64,
    gap: 10,
    opacity: 0.65,
    hitboxScale: 1.25,
    hapticsEnabled: false,
    dpadDeadzoneRatio: 0.16,
    turboFramesPerToggle: 2,
    portraitDpadOffset: Offset.zero,
    portraitButtonsOffset: Offset.zero,
    portraitSystemOffset: Offset.zero,
    portraitAOffset: Offset.zero,
    portraitBOffset: Offset.zero,
    portraitTurboAOffset: Offset.zero,
    portraitTurboBOffset: Offset.zero,
    portraitSelectOffset: Offset.zero,
    portraitStartOffset: Offset.zero,
    landscapeDpadOffset: Offset.zero,
    landscapeButtonsOffset: Offset.zero,
    landscapeSystemOffset: Offset.zero,
    landscapeAOffset: Offset.zero,
    landscapeBOffset: Offset.zero,
    landscapeTurboAOffset: Offset.zero,
    landscapeTurboBOffset: Offset.zero,
    landscapeSelectOffset: Offset.zero,
    landscapeStartOffset: Offset.zero,
    portraitDpadScale: 1.0,
    portraitButtonsScale: 1.0,
    portraitSystemScale: 1.0,
    portraitAScale: 1.0,
    portraitBScale: 1.0,
    portraitTurboAScale: 1.0,
    portraitTurboBScale: 1.0,
    portraitSelectScale: 1.0,
    portraitStartScale: 1.0,
    landscapeDpadScale: 1.0,
    landscapeButtonsScale: 1.0,
    landscapeSystemScale: 1.0,
    landscapeAScale: 1.0,
    landscapeBScale: 1.0,
    landscapeTurboAScale: 1.0,
    landscapeTurboBScale: 1.0,
    landscapeSelectScale: 1.0,
    landscapeStartScale: 1.0,
  );
}

class VirtualControlsSettingsController
    extends Notifier<VirtualControlsSettings> {
  @override
  VirtualControlsSettings build() {
    final loaded = _virtualControlsFromStorage(
      ref.read(appStorageProvider).get(StorageKeys.settingsVirtualControls),
    );
    final settings = loaded ?? VirtualControlsSettings.defaults;

    unawaitedLogged(
      nes_input.setTurboFramesPerToggle(frames: settings.turboFramesPerToggle),
      message: 'setTurboFramesPerToggle (init)',
      logger: 'virtual_controls_settings',
    );

    return settings;
  }

  void replace(VirtualControlsSettings value) => _set(value);

  void setButtonSize(double value) => _set(state.copyWith(buttonSize: value));
  void setGap(double value) => _set(state.copyWith(gap: value));
  void setOpacity(double value) => _set(state.copyWith(opacity: value));
  void setHitboxScale(double value) => _set(state.copyWith(hitboxScale: value));
  void setHapticsEnabled(bool value) =>
      _set(state.copyWith(hapticsEnabled: value));
  void setDpadDeadzoneRatio(double value) =>
      _set(state.copyWith(dpadDeadzoneRatio: value));
  void setTurboFramesPerToggle(int value) {
    final next = value.clamp(1, 255);
    _set(state.copyWith(turboFramesPerToggle: next));
    unawaitedLogged(
      nes_input.setTurboFramesPerToggle(frames: next),
      message: 'setTurboFramesPerToggle',
      logger: 'virtual_controls_settings',
    );
  }

  void setPortraitDpadOffset(Offset value) =>
      _set(state.copyWith(portraitDpadOffset: value));
  void setPortraitButtonsOffset(Offset value) =>
      _set(state.copyWith(portraitButtonsOffset: value));
  void setPortraitSystemOffset(Offset value) =>
      _set(state.copyWith(portraitSystemOffset: value));
  void setLandscapeDpadOffset(Offset value) =>
      _set(state.copyWith(landscapeDpadOffset: value));
  void setLandscapeButtonsOffset(Offset value) =>
      _set(state.copyWith(landscapeButtonsOffset: value));
  void setLandscapeSystemOffset(Offset value) =>
      _set(state.copyWith(landscapeSystemOffset: value));

  void setPortraitDpadScale(double value) =>
      _set(state.copyWith(portraitDpadScale: value));
  void setPortraitButtonsScale(double value) =>
      _set(state.copyWith(portraitButtonsScale: value));
  void setPortraitSystemScale(double value) =>
      _set(state.copyWith(portraitSystemScale: value));
  void setLandscapeDpadScale(double value) =>
      _set(state.copyWith(landscapeDpadScale: value));
  void setLandscapeButtonsScale(double value) =>
      _set(state.copyWith(landscapeButtonsScale: value));
  void setLandscapeSystemScale(double value) =>
      _set(state.copyWith(landscapeSystemScale: value));

  void _set(VirtualControlsSettings next) {
    state = next;
    _persist(next);
  }

  void _persist(VirtualControlsSettings value) {
    unawaitedLogged(
      Future<void>.sync(
        () => ref
            .read(appStorageProvider)
            .put(
              StorageKeys.settingsVirtualControls,
              _virtualControlsToStorage(value),
            ),
      ),
      message: 'Persist virtual controls settings',
      logger: 'virtual_controls_settings',
    );
  }
}

final virtualControlsSettingsProvider =
    NotifierProvider<
      VirtualControlsSettingsController,
      VirtualControlsSettings
    >(VirtualControlsSettingsController.new);

Map<String, Object?> _virtualControlsToStorage(VirtualControlsSettings value) {
  List<double> offset(Offset v) => <double>[v.dx, v.dy];

  return <String, Object?>{
    'buttonSize': value.buttonSize,
    'gap': value.gap,
    'opacity': value.opacity,
    'hitboxScale': value.hitboxScale,
    'hapticsEnabled': value.hapticsEnabled,
    'dpadDeadzoneRatio': value.dpadDeadzoneRatio,
    'turboFramesPerToggle': value.turboFramesPerToggle,
    'portraitDpadOffset': offset(value.portraitDpadOffset),
    'portraitButtonsOffset': offset(value.portraitButtonsOffset),
    'portraitSystemOffset': offset(value.portraitSystemOffset),
    'portraitAOffset': offset(value.portraitAOffset),
    'portraitBOffset': offset(value.portraitBOffset),
    'portraitTurboAOffset': offset(value.portraitTurboAOffset),
    'portraitTurboBOffset': offset(value.portraitTurboBOffset),
    'portraitSelectOffset': offset(value.portraitSelectOffset),
    'portraitStartOffset': offset(value.portraitStartOffset),
    'landscapeDpadOffset': offset(value.landscapeDpadOffset),
    'landscapeButtonsOffset': offset(value.landscapeButtonsOffset),
    'landscapeSystemOffset': offset(value.landscapeSystemOffset),
    'landscapeAOffset': offset(value.landscapeAOffset),
    'landscapeBOffset': offset(value.landscapeBOffset),
    'landscapeTurboAOffset': offset(value.landscapeTurboAOffset),
    'landscapeTurboBOffset': offset(value.landscapeTurboBOffset),
    'landscapeSelectOffset': offset(value.landscapeSelectOffset),
    'landscapeStartOffset': offset(value.landscapeStartOffset),
    'portraitDpadScale': value.portraitDpadScale,
    'portraitButtonsScale': value.portraitButtonsScale,
    'portraitSystemScale': value.portraitSystemScale,
    'portraitAScale': value.portraitAScale,
    'portraitBScale': value.portraitBScale,
    'portraitTurboAScale': value.portraitTurboAScale,
    'portraitTurboBScale': value.portraitTurboBScale,
    'portraitSelectScale': value.portraitSelectScale,
    'portraitStartScale': value.portraitStartScale,
    'landscapeDpadScale': value.landscapeDpadScale,
    'landscapeButtonsScale': value.landscapeButtonsScale,
    'landscapeSystemScale': value.landscapeSystemScale,
    'landscapeAScale': value.landscapeAScale,
    'landscapeBScale': value.landscapeBScale,
    'landscapeTurboAScale': value.landscapeTurboAScale,
    'landscapeTurboBScale': value.landscapeTurboBScale,
    'landscapeSelectScale': value.landscapeSelectScale,
    'landscapeStartScale': value.landscapeStartScale,
  };
}

VirtualControlsSettings? _virtualControlsFromStorage(Object? value) {
  if (value is! Map) return null;
  final map = value.cast<String, Object?>();
  final defaults = VirtualControlsSettings.defaults;

  double d(Object? v, double fallback) => v is num ? v.toDouble() : fallback;
  int i(Object? v, int fallback) => v is num ? v.toInt() : fallback;
  bool b(Object? v, bool fallback) => v is bool ? v : fallback;
  Offset o(Object? v, Offset fallback) {
    if (v is List && v.length == 2 && v[0] is num && v[1] is num) {
      return Offset((v[0] as num).toDouble(), (v[1] as num).toDouble());
    }
    return fallback;
  }

  return defaults.copyWith(
    buttonSize: d(map['buttonSize'], defaults.buttonSize),
    gap: d(map['gap'], defaults.gap),
    opacity: d(map['opacity'], defaults.opacity),
    hitboxScale: d(map['hitboxScale'], defaults.hitboxScale),
    hapticsEnabled: b(map['hapticsEnabled'], defaults.hapticsEnabled),
    dpadDeadzoneRatio: d(map['dpadDeadzoneRatio'], defaults.dpadDeadzoneRatio),
    turboFramesPerToggle: i(
      map['turboFramesPerToggle'],
      defaults.turboFramesPerToggle,
    ).clamp(1, 255),
    portraitDpadOffset: o(
      map['portraitDpadOffset'],
      defaults.portraitDpadOffset,
    ),
    portraitButtonsOffset: o(
      map['portraitButtonsOffset'],
      defaults.portraitButtonsOffset,
    ),
    portraitSystemOffset: o(
      map['portraitSystemOffset'],
      defaults.portraitSystemOffset,
    ),
    portraitAOffset: o(map['portraitAOffset'], defaults.portraitAOffset),
    portraitBOffset: o(map['portraitBOffset'], defaults.portraitBOffset),
    portraitTurboAOffset: o(
      map['portraitTurboAOffset'],
      defaults.portraitTurboAOffset,
    ),
    portraitTurboBOffset: o(
      map['portraitTurboBOffset'],
      defaults.portraitTurboBOffset,
    ),
    portraitSelectOffset: o(
      map['portraitSelectOffset'],
      defaults.portraitSelectOffset,
    ),
    portraitStartOffset: o(
      map['portraitStartOffset'],
      defaults.portraitStartOffset,
    ),
    landscapeDpadOffset: o(
      map['landscapeDpadOffset'],
      defaults.landscapeDpadOffset,
    ),
    landscapeButtonsOffset: o(
      map['landscapeButtonsOffset'],
      defaults.landscapeButtonsOffset,
    ),
    landscapeSystemOffset: o(
      map['landscapeSystemOffset'],
      defaults.landscapeSystemOffset,
    ),
    landscapeAOffset: o(map['landscapeAOffset'], defaults.landscapeAOffset),
    landscapeBOffset: o(map['landscapeBOffset'], defaults.landscapeBOffset),
    landscapeTurboAOffset: o(
      map['landscapeTurboAOffset'],
      defaults.landscapeTurboAOffset,
    ),
    landscapeTurboBOffset: o(
      map['landscapeTurboBOffset'],
      defaults.landscapeTurboBOffset,
    ),
    landscapeSelectOffset: o(
      map['landscapeSelectOffset'],
      defaults.landscapeSelectOffset,
    ),
    landscapeStartOffset: o(
      map['landscapeStartOffset'],
      defaults.landscapeStartOffset,
    ),
    portraitDpadScale: d(map['portraitDpadScale'], defaults.portraitDpadScale),
    portraitButtonsScale: d(
      map['portraitButtonsScale'],
      defaults.portraitButtonsScale,
    ),
    portraitSystemScale: d(
      map['portraitSystemScale'],
      defaults.portraitSystemScale,
    ),
    portraitAScale: d(map['portraitAScale'], defaults.portraitAScale),
    portraitBScale: d(map['portraitBScale'], defaults.portraitBScale),
    portraitTurboAScale: d(
      map['portraitTurboAScale'],
      defaults.portraitTurboAScale,
    ),
    portraitTurboBScale: d(
      map['portraitTurboBScale'],
      defaults.portraitTurboBScale,
    ),
    portraitSelectScale: d(
      map['portraitSelectScale'],
      defaults.portraitSelectScale,
    ),
    portraitStartScale: d(
      map['portraitStartScale'],
      defaults.portraitStartScale,
    ),
    landscapeDpadScale: d(
      map['landscapeDpadScale'],
      defaults.landscapeDpadScale,
    ),
    landscapeButtonsScale: d(
      map['landscapeButtonsScale'],
      defaults.landscapeButtonsScale,
    ),
    landscapeSystemScale: d(
      map['landscapeSystemScale'],
      defaults.landscapeSystemScale,
    ),
    landscapeAScale: d(map['landscapeAScale'], defaults.landscapeAScale),
    landscapeBScale: d(map['landscapeBScale'], defaults.landscapeBScale),
    landscapeTurboAScale: d(
      map['landscapeTurboAScale'],
      defaults.landscapeTurboAScale,
    ),
    landscapeTurboBScale: d(
      map['landscapeTurboBScale'],
      defaults.landscapeTurboBScale,
    ),
    landscapeSelectScale: d(
      map['landscapeSelectScale'],
      defaults.landscapeSelectScale,
    ),
    landscapeStartScale: d(
      map['landscapeStartScale'],
      defaults.landscapeStartScale,
    ),
  );
}
