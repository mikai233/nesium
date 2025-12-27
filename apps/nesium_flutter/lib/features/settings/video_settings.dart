import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/api/palette.dart' as nes_palette;
import '../../logging/app_logger.dart';
import '../../platform/platform_capabilities.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';

enum PaletteMode { builtin, custom }

@immutable
class VideoSettings {
  static const Object _unset = Object();

  const VideoSettings({
    required this.paletteMode,
    required this.builtinPreset,
    required this.integerScaling,
    this.customPaletteName,
  });

  final PaletteMode paletteMode;
  final nes_palette.PaletteKind builtinPreset;
  final bool integerScaling;
  final String? customPaletteName;

  VideoSettings copyWith({
    PaletteMode? paletteMode,
    nes_palette.PaletteKind? builtinPreset,
    bool? integerScaling,
    Object? customPaletteName = _unset,
  }) {
    return VideoSettings(
      paletteMode: paletteMode ?? this.paletteMode,
      builtinPreset: builtinPreset ?? this.builtinPreset,
      integerScaling: integerScaling ?? this.integerScaling,
      customPaletteName: identical(customPaletteName, _unset)
          ? this.customPaletteName
          : customPaletteName as String?,
    );
  }

  static VideoSettings defaults() {
    return VideoSettings(
      paletteMode: PaletteMode.builtin,
      builtinPreset: nes_palette.PaletteKind.nesdevNtsc,
      integerScaling: isNativeMobile,
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
    scheduleMicrotask(() {
      if (settings.paletteMode == PaletteMode.custom &&
          customBytes is Uint8List) {
        unawaitedLogged(
          nes_palette.setPalettePalData(data: customBytes),
          message: 'setPalettePalData (init)',
          logger: 'video_settings',
        );
        return;
      }
      unawaitedLogged(
        nes_palette.setPalettePreset(kind: settings.builtinPreset),
        message: 'setPalettePreset (init)',
        logger: 'video_settings',
      );
    });

    if (settings.paletteMode == PaletteMode.custom &&
        customBytes is! Uint8List) {
      return settings.copyWith(paletteMode: PaletteMode.builtin);
    }
    return settings;
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
    } catch (_) {}
    await _persist(state);
    await nes_palette.setPalettePalData(data: data);
  }

  Future<void> setIntegerScaling(bool value) async {
    if (value == state.integerScaling) return;
    state = state.copyWith(integerScaling: value);
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
    } catch (_) {}
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
    } catch (_) {}
  }

  nes_palette.PaletteKind builtinPreset = defaults.builtinPreset;
  if (map['builtinPreset'] is String) {
    try {
      builtinPreset = nes_palette.PaletteKind.values.byName(
        map['builtinPreset'] as String,
      );
    } catch (_) {}
  }

  final customPaletteName = map['customPaletteName'] is String
      ? map['customPaletteName'] as String
      : null;

  final integerScaling = map['integerScaling'] is bool
      ? map['integerScaling'] as bool
      : defaults.integerScaling;

  return defaults.copyWith(
    paletteMode: paletteMode,
    builtinPreset: builtinPreset,
    integerScaling: integerScaling,
    customPaletteName: customPaletteName,
  );
}
