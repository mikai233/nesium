// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'video_settings.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_VideoSettings _$VideoSettingsFromJson(
  Map<String, dynamic> json,
) => _VideoSettings(
  paletteMode:
      $enumDecodeNullable(
        _$PaletteModeEnumMap,
        json['paletteMode'],
        unknownValue: PaletteMode.builtin,
      ) ??
      PaletteMode.builtin,
  builtinPreset:
      $enumDecodeNullable(
        _$PaletteKindEnumMap,
        json['builtinPreset'],
        unknownValue: PaletteKind.nesdevNtsc,
      ) ??
      PaletteKind.nesdevNtsc,
  videoFilter:
      $enumDecodeNullable(
        _$VideoFilterEnumMap,
        json['videoFilter'],
        unknownValue: VideoFilter.none,
      ) ??
      VideoFilter.none,
  integerScaling: json['integerScaling'] as bool? ?? false,
  aspectRatio:
      $enumDecodeNullable(
        _$NesAspectRatioEnumMap,
        json['aspectRatio'],
        unknownValue: NesAspectRatio.square,
      ) ??
      NesAspectRatio.square,
  screenVerticalOffset: (json['screenVerticalOffset'] as num?)?.toDouble() ?? 0,
  ntscOptions: json['ntscOptions'] == null
      ? NtscOptionsConverter._fallback
      : const NtscOptionsConverter().fromJson(
          json['ntscOptions'] as Map<String, dynamic>,
        ),
  ntscBisqwitOptions: json['ntscBisqwitOptions'] == null
      ? NtscBisqwitOptionsConverter._fallback
      : const NtscBisqwitOptionsConverter().fromJson(
          json['ntscBisqwitOptions'] as Map<String, dynamic>,
        ),
  lcdGridStrength: (json['lcdGridStrength'] as num?)?.toDouble() ?? 1.0,
  scanlineIntensity: (json['scanlineIntensity'] as num?)?.toDouble() ?? 0.30,
  customPaletteName: json['customPaletteName'] as String?,
);

Map<String, dynamic> _$VideoSettingsToJson(_VideoSettings instance) =>
    <String, dynamic>{
      'paletteMode': _$PaletteModeEnumMap[instance.paletteMode]!,
      'builtinPreset': _$PaletteKindEnumMap[instance.builtinPreset]!,
      'videoFilter': _$VideoFilterEnumMap[instance.videoFilter]!,
      'integerScaling': instance.integerScaling,
      'aspectRatio': _$NesAspectRatioEnumMap[instance.aspectRatio]!,
      'screenVerticalOffset': instance.screenVerticalOffset,
      'ntscOptions': const NtscOptionsConverter().toJson(instance.ntscOptions),
      'ntscBisqwitOptions': const NtscBisqwitOptionsConverter().toJson(
        instance.ntscBisqwitOptions,
      ),
      'lcdGridStrength': instance.lcdGridStrength,
      'scanlineIntensity': instance.scanlineIntensity,
      'customPaletteName': instance.customPaletteName,
    };

const _$PaletteModeEnumMap = {
  PaletteMode.builtin: 'builtin',
  PaletteMode.custom: 'custom',
};

const _$PaletteKindEnumMap = {
  PaletteKind.nesdevNtsc: 'nesdevNtsc',
  PaletteKind.fbxCompositeDirect: 'fbxCompositeDirect',
  PaletteKind.sonyCxa2025AsUs: 'sonyCxa2025AsUs',
  PaletteKind.pal2C07: 'pal2C07',
  PaletteKind.rawLinear: 'rawLinear',
};

const _$VideoFilterEnumMap = {
  VideoFilter.none: 'none',
  VideoFilter.prescale2X: 'prescale2X',
  VideoFilter.prescale3X: 'prescale3X',
  VideoFilter.prescale4X: 'prescale4X',
  VideoFilter.hq2X: 'hq2X',
  VideoFilter.hq3X: 'hq3X',
  VideoFilter.hq4X: 'hq4X',
  VideoFilter.sai2X: 'sai2X',
  VideoFilter.super2XSai: 'super2XSai',
  VideoFilter.superEagle: 'superEagle',
  VideoFilter.ntscComposite: 'ntscComposite',
  VideoFilter.ntscSVideo: 'ntscSVideo',
  VideoFilter.ntscRgb: 'ntscRgb',
  VideoFilter.ntscMonochrome: 'ntscMonochrome',
  VideoFilter.lcdGrid: 'lcdGrid',
  VideoFilter.scanlines: 'scanlines',
  VideoFilter.xbrz2X: 'xbrz2X',
  VideoFilter.xbrz3X: 'xbrz3X',
  VideoFilter.xbrz4X: 'xbrz4X',
  VideoFilter.xbrz5X: 'xbrz5X',
  VideoFilter.xbrz6X: 'xbrz6X',
  VideoFilter.ntscBisqwit2X: 'ntscBisqwit2X',
  VideoFilter.ntscBisqwit4X: 'ntscBisqwit4X',
  VideoFilter.ntscBisqwit8X: 'ntscBisqwit8X',
};

const _$NesAspectRatioEnumMap = {
  NesAspectRatio.square: 'square',
  NesAspectRatio.ntsc: 'ntsc',
  NesAspectRatio.stretch: 'stretch',
};
