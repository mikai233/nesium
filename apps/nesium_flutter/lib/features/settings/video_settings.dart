// ignore_for_file: invalid_annotation_target

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import '../../platform/nes_palette.dart' as nes_palette;
import '../../platform/nes_palette.dart' show PaletteKind;
import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../platform/nes_video.dart' as nes_video;
import '../../platform/nes_video.dart' show NtscOptions, VideoFilter;
import '../../windows/settings_sync.dart';

part 'video_settings.freezed.dart';
part 'video_settings.g.dart';

enum PaletteMode { builtin, custom }

enum NesAspectRatio { square, ntsc, stretch }

class NtscOptionsConverter
    implements JsonConverter<NtscOptions, Map<String, dynamic>> {
  const NtscOptionsConverter();

  static const NtscOptions _fallback = NtscOptions(
    hue: 0,
    saturation: 0,
    contrast: 0,
    brightness: 0,
    sharpness: 0,
    gamma: 0,
    resolution: 0,
    artifacts: 0,
    fringing: 0,
    bleed: 0,
    mergeFields: true,
  );

  @override
  NtscOptions fromJson(Map<String, dynamic> json) {
    double readDouble(String key, double fallback) {
      final value = json[key];
      return value is num ? value.toDouble() : fallback;
    }

    final mergeFieldsValue = json['mergeFields'];
    final mergeFields = mergeFieldsValue is bool
        ? mergeFieldsValue
        : _fallback.mergeFields;

    return NtscOptions(
      hue: readDouble('hue', _fallback.hue),
      saturation: readDouble('saturation', _fallback.saturation),
      contrast: readDouble('contrast', _fallback.contrast),
      brightness: readDouble('brightness', _fallback.brightness),
      sharpness: readDouble('sharpness', _fallback.sharpness),
      gamma: readDouble('gamma', _fallback.gamma),
      resolution: readDouble('resolution', _fallback.resolution),
      artifacts: readDouble('artifacts', _fallback.artifacts),
      fringing: readDouble('fringing', _fallback.fringing),
      bleed: readDouble('bleed', _fallback.bleed),
      mergeFields: mergeFields,
    );
  }

  @override
  Map<String, dynamic> toJson(NtscOptions object) => <String, dynamic>{
    'hue': object.hue,
    'saturation': object.saturation,
    'contrast': object.contrast,
    'brightness': object.brightness,
    'sharpness': object.sharpness,
    'gamma': object.gamma,
    'resolution': object.resolution,
    'artifacts': object.artifacts,
    'fringing': object.fringing,
    'bleed': object.bleed,
    'mergeFields': object.mergeFields,
  };
}

@freezed
sealed class VideoSettings with _$VideoSettings {
  const VideoSettings._();

  const factory VideoSettings({
    @JsonKey(unknownEnumValue: PaletteMode.builtin)
    @Default(PaletteMode.builtin)
    PaletteMode paletteMode,
    @JsonKey(unknownEnumValue: PaletteKind.nesdevNtsc)
    @Default(PaletteKind.nesdevNtsc)
    PaletteKind builtinPreset,
    @JsonKey(unknownEnumValue: VideoFilter.none)
    @Default(VideoFilter.none)
    VideoFilter videoFilter,
    @Default(false) bool integerScaling,
    @JsonKey(unknownEnumValue: NesAspectRatio.square)
    @Default(NesAspectRatio.square)
    NesAspectRatio aspectRatio,
    @Default(0) double screenVerticalOffset,
    @NtscOptionsConverter()
    @Default(NtscOptionsConverter._fallback)
    NtscOptions ntscOptions,
    String? customPaletteName,
  }) = _VideoSettings;

  factory VideoSettings.fromJson(Map<String, dynamic> json) =>
      _$VideoSettingsFromJson(json);
}

class VideoSettingsController extends Notifier<VideoSettings> {
  @override
  VideoSettings build() {
    final settings = _loadSettingsFromStorage();

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

  Future<void> reloadFromStorage({bool applyPalette = true}) async {
    final settings = _loadSettingsFromStorage();

    final customBytes = ref
        .read(appStorageProvider)
        .get(StorageKeys.settingsVideoCustomPaletteBytes);
    final next =
        (settings.paletteMode == PaletteMode.custom &&
            customBytes is! Uint8List)
        ? settings.copyWith(paletteMode: PaletteMode.builtin)
        : settings;
    if (next != state) {
      state = next;
    }

    if (applyPalette) {
      await applyToRuntime();
    }
  }

  void applySynced(VideoSettings next) {
    if (next == state) return;
    state = next;
  }

  VideoSettings _loadSettingsFromStorage() {
    final stored = ref.read(appStorageProvider).get(StorageKeys.settingsVideo);
    if (stored is Map) {
      try {
        return VideoSettings.fromJson(Map<String, dynamic>.from(stored));
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'Failed to load video settings',
          logger: 'video_settings',
        );
      }
    }
    return const VideoSettings();
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
    await _persist(state, broadcastFields: const ['integerScaling']);
  }

  Future<void> setVideoFilter(nes_video.VideoFilter filter) async {
    if (filter == state.videoFilter) return;
    state = state.copyWith(videoFilter: filter);
    await _persist(state, broadcastFields: const ['videoFilter']);
  }

  Future<void> setAspectRatio(NesAspectRatio value) async {
    if (value == state.aspectRatio) return;
    state = state.copyWith(aspectRatio: value);
    await _persist(state, broadcastFields: const ['aspectRatio']);
  }

  Future<void> setScreenVerticalOffset(double value) async {
    final clamped = value.clamp(-240.0, 240.0).toDouble();
    if (clamped == state.screenVerticalOffset) return;
    state = state.copyWith(screenVerticalOffset: clamped);
    await _persist(state, broadcastFields: const ['screenVerticalOffset']);
  }

  Future<void> setNtscOptions(nes_video.NtscOptions value) async {
    if (value == state.ntscOptions) return;
    state = state.copyWith(ntscOptions: value);
    // NTSC tuning parameters are applied to the shared Rust pipeline directly
    // (see settings UI debounce). Other windows don't depend on these values.
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

  Future<void> _persist(
    VideoSettings value, {
    List<String> broadcastFields = const <String>[],
  }) async {
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsVideo, value.toJson());
      if (broadcastFields.isNotEmpty) {
        unawaited(
          SettingsSync.broadcast(
            group: 'video',
            fields: broadcastFields,
            payload: value.toJson(),
          ),
        );
      }
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
