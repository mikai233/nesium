import 'dart:ui';

import 'package:nesium_flutter/bridge/api/input.dart' as nes_input;
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
  VirtualControlsSettings build() => VirtualControlsSettings.defaults;

  void replace(VirtualControlsSettings value) => state = value;

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
  void setPortraitSystemOffset(Offset value) =>
      state = state.copyWith(portraitSystemOffset: value);
  void setLandscapeDpadOffset(Offset value) =>
      state = state.copyWith(landscapeDpadOffset: value);
  void setLandscapeButtonsOffset(Offset value) =>
      state = state.copyWith(landscapeButtonsOffset: value);
  void setLandscapeSystemOffset(Offset value) =>
      state = state.copyWith(landscapeSystemOffset: value);

  void setPortraitDpadScale(double value) =>
      state = state.copyWith(portraitDpadScale: value);
  void setPortraitButtonsScale(double value) =>
      state = state.copyWith(portraitButtonsScale: value);
  void setPortraitSystemScale(double value) =>
      state = state.copyWith(portraitSystemScale: value);
  void setLandscapeDpadScale(double value) =>
      state = state.copyWith(landscapeDpadScale: value);
  void setLandscapeButtonsScale(double value) =>
      state = state.copyWith(landscapeButtonsScale: value);
  void setLandscapeSystemScale(double value) =>
      state = state.copyWith(landscapeSystemScale: value);
}

final virtualControlsSettingsProvider =
    NotifierProvider<
      VirtualControlsSettingsController,
      VirtualControlsSettings
    >(VirtualControlsSettingsController.new);
