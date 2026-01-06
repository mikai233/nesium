import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../platform/nes_palette.dart' as nes_palette;
import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';

enum PaletteMode { builtin, custom }

enum NesAspectRatio { square, ntsc, stretch }

@immutable
class VideoSettings {
  static const Object _unset = Object();

  const VideoSettings({
    required this.paletteMode,
    required this.builtinPreset,
    required this.integerScaling,
    required this.aspectRatio,
    required this.screenVerticalOffset,
    this.customPaletteName,
  });

  final PaletteMode paletteMode;
  final nes_palette.PaletteKind builtinPreset;
  final bool integerScaling;
  final NesAspectRatio aspectRatio;
  final double screenVerticalOffset;
  final String? customPaletteName;

  VideoSettings copyWith({
    PaletteMode? paletteMode,
    nes_palette.PaletteKind? builtinPreset,
    bool? integerScaling,
    NesAspectRatio? aspectRatio,
    double? screenVerticalOffset,
    Object? customPaletteName = _unset,
  }) {
    return VideoSettings(
      paletteMode: paletteMode ?? this.paletteMode,
      builtinPreset: builtinPreset ?? this.builtinPreset,
      integerScaling: integerScaling ?? this.integerScaling,
      aspectRatio: aspectRatio ?? this.aspectRatio,
      screenVerticalOffset: screenVerticalOffset ?? this.screenVerticalOffset,
      customPaletteName: identical(customPaletteName, _unset)
          ? this.customPaletteName
          : customPaletteName as String?,
    );
  }

  static VideoSettings defaults() {
    return VideoSettings(
      paletteMode: PaletteMode.builtin,
      builtinPreset: nes_palette.PaletteKind.nesdevNtsc,
      integerScaling: false,
      aspectRatio: NesAspectRatio.square,
      screenVerticalOffset: 0,
      customPaletteName: null,
    );
  }
}

class VideoSettingsController extends Notifier<VideoSettings> {
  @override
  VideoSettings build() {
    final defaults = VideoSettings.defaults();
    final loaded = _videoSettingsFromStorage(
      ref.read(appStorageProvider).get(StorageKeys.settingsVideo),
      defaults: defaults,
    );
    final settings = loaded ?? defaults;

    final customBytes = ref
        .read(appStorageProvider)
        .get(StorageKeys.settingsVideoCustomPaletteBytes);
    final next =
        (settings.paletteMode == PaletteMode.custom &&
            customBytes is! Uint8List)
        ? settings.copyWith(paletteMode: PaletteMode.builtin)
        : settings;
    scheduleMicrotask(() {
      unawaitedLogged(
        applyToRuntime(),
        message: 'applyToRuntime (init)',
        logger: 'video_settings',
      );
    });
    return next;
  }

  Future<void> applyToRuntime() async {
    final storage = ref.read(appStorageProvider);
    if (state.paletteMode == PaletteMode.custom) {
      final customBytes = storage.get(
        StorageKeys.settingsVideoCustomPaletteBytes,
      );
      if (customBytes is Uint8List) {
        await nes_palette.setPalettePalData(data: customBytes);
        return;
      }
    }
    await nes_palette.setPalettePreset(kind: state.builtinPreset);
  }

  Future<void> setPaletteMode(PaletteMode mode) async {
    if (mode == state.paletteMode) return;
    state = state.copyWith(paletteMode: mode);
    await _persist(state);
    await applyToRuntime();
  }

  Future<void> setBuiltinPreset(nes_palette.PaletteKind preset) async {
    if (preset == state.builtinPreset &&
        state.paletteMode == PaletteMode.builtin) {
      return;
    }
    state = state.copyWith(
      paletteMode: PaletteMode.builtin,
      builtinPreset: preset,
    );
    await _persist(state);
    await nes_palette.setPalettePreset(kind: preset);
  }

  Future<void> setCustomPalette(Uint8List data, {String? name}) async {
    state = state.copyWith(
      paletteMode: PaletteMode.custom,
      customPaletteName: name ?? state.customPaletteName ?? 'custom',
    );
    final storage = ref.read(appStorageProvider);
    try {
      await storage.put(
        StorageKeys.settingsVideoCustomPaletteBytes,
        Uint8List.fromList(data),
      );
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist custom palette bytes',
        logger: 'video_settings',
      );
    }
    await _persist(state);
    await nes_palette.setPalettePalData(data: data);
  }

  Future<void> setIntegerScaling(bool value) async {
    if (value == state.integerScaling) return;
    state = state.copyWith(integerScaling: value);
    await _persist(state);
  }

  Future<void> setAspectRatio(NesAspectRatio value) async {
    if (value == state.aspectRatio) return;
    state = state.copyWith(aspectRatio: value);
    await _persist(state);
  }

  Future<void> setScreenVerticalOffset(double value) async {
    final clamped = value.clamp(-240.0, 240.0).toDouble();
    if (clamped == state.screenVerticalOffset) return;
    state = state.copyWith(screenVerticalOffset: clamped);
    await _persist(state);
  }

  void useCustomIfAvailable() {
    if (state.customPaletteName == null) return;
    state = state.copyWith(paletteMode: PaletteMode.custom);
    unawaited(_persist(state));
    final bytes = ref
        .read(appStorageProvider)
        .get(StorageKeys.settingsVideoCustomPaletteBytes);
    if (bytes is Uint8List) {
      unawaitedLogged(
        nes_palette.setPalettePalData(data: bytes),
        message: 'setPalettePalData (useCustomIfAvailable)',
        logger: 'video_settings',
      );
    }
  }

  Future<void> _persist(VideoSettings value) async {
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsVideo, _videoSettingsToStorage(value));
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist video settings',
        logger: 'video_settings',
      );
    }
  }
}

final videoSettingsProvider =
    NotifierProvider<VideoSettingsController, VideoSettings>(
      VideoSettingsController.new,
    );

Map<String, Object?> _videoSettingsToStorage(VideoSettings value) =>
    <String, Object?>{
      'paletteMode': value.paletteMode.name,
      'builtinPreset': value.builtinPreset.name,
      'integerScaling': value.integerScaling,
      'aspectRatio': value.aspectRatio.name,
      'screenVerticalOffset': value.screenVerticalOffset,
      'customPaletteName': value.customPaletteName,
    };

VideoSettings? _videoSettingsFromStorage(
  Object? value, {
  required VideoSettings defaults,
}) {
  if (value is! Map) return null;
  final map = value.cast<String, Object?>();

  PaletteMode paletteMode = defaults.paletteMode;
  if (map['paletteMode'] is String) {
    try {
      paletteMode = PaletteMode.values.byName(map['paletteMode'] as String);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to lookup PaletteMode by name',
        logger: 'video_settings',
      );
    }
  }

  nes_palette.PaletteKind builtinPreset = defaults.builtinPreset;
  if (map['builtinPreset'] is String) {
    try {
      builtinPreset = nes_palette.PaletteKind.values.byName(
        map['builtinPreset'] as String,
      );
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to lookup PaletteKind by name',
        logger: 'video_settings',
      );
    }
  }

  final customPaletteName = map['customPaletteName'] is String
      ? map['customPaletteName'] as String
      : null;

  final integerScaling = map['integerScaling'] is bool
      ? map['integerScaling'] as bool
      : defaults.integerScaling;

  NesAspectRatio aspectRatio = defaults.aspectRatio;
  if (map['aspectRatio'] is String) {
    try {
      aspectRatio = NesAspectRatio.values.byName(map['aspectRatio'] as String);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to lookup NesAspectRatio by name',
        logger: 'video_settings',
      );
    }
  }

  final screenVerticalOffset = map['screenVerticalOffset'] is num
      ? (map['screenVerticalOffset'] as num).toDouble()
      : defaults.screenVerticalOffset;

  return defaults.copyWith(
    paletteMode: paletteMode,
    builtinPreset: builtinPreset,
    integerScaling: integerScaling,
    aspectRatio: aspectRatio,
    screenVerticalOffset: screenVerticalOffset,
    customPaletteName: customPaletteName,
  );
}
