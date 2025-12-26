import 'dart:ui';

import 'package:nesium_flutter/src/rust/api/input.dart' as nes_input;
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

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
    required this.landscapeDpadOffset,
    required this.landscapeButtonsOffset,
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
  final Offset landscapeDpadOffset;
  final Offset landscapeButtonsOffset;

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
    Offset? landscapeDpadOffset,
    Offset? landscapeButtonsOffset,
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
      landscapeDpadOffset: landscapeDpadOffset ?? this.landscapeDpadOffset,
      landscapeButtonsOffset:
          landscapeButtonsOffset ?? this.landscapeButtonsOffset,
    );
  }

  static const defaults = VirtualControlsSettings(
    buttonSize: 64,
    gap: 10,
    opacity: 0.65,
    hitboxScale: 1.25,
    hapticsEnabled: true,
    dpadDeadzoneRatio: 0.16,
    turboFramesPerToggle: 2,
    portraitDpadOffset: Offset.zero,
    portraitButtonsOffset: Offset.zero,
    landscapeDpadOffset: Offset.zero,
    landscapeButtonsOffset: Offset.zero,
  );
}

class VirtualControlsSettingsController
    extends Notifier<VirtualControlsSettings> {
  @override
  VirtualControlsSettings build() => VirtualControlsSettings.defaults;

  void setButtonSize(double value) => state = state.copyWith(buttonSize: value);
  void setGap(double value) => state = state.copyWith(gap: value);
  void setOpacity(double value) => state = state.copyWith(opacity: value);
  void setHitboxScale(double value) =>
      state = state.copyWith(hitboxScale: value);
  void setHapticsEnabled(bool value) =>
      state = state.copyWith(hapticsEnabled: value);
  void setDpadDeadzoneRatio(double value) =>
      state = state.copyWith(dpadDeadzoneRatio: value);
  void setTurboFramesPerToggle(int value) {
    final next = value.clamp(1, 255);
    state = state.copyWith(turboFramesPerToggle: next);
    nes_input.setTurboFramesPerToggle(frames: next).catchError((_) {});
  }

  void setPortraitDpadOffset(Offset value) =>
      state = state.copyWith(portraitDpadOffset: value);
  void setPortraitButtonsOffset(Offset value) =>
      state = state.copyWith(portraitButtonsOffset: value);
  void setLandscapeDpadOffset(Offset value) =>
      state = state.copyWith(landscapeDpadOffset: value);
  void setLandscapeButtonsOffset(Offset value) =>
      state = state.copyWith(landscapeButtonsOffset: value);
}

final virtualControlsSettingsProvider =
    NotifierProvider<
      VirtualControlsSettingsController,
      VirtualControlsSettings
    >(VirtualControlsSettingsController.new);
