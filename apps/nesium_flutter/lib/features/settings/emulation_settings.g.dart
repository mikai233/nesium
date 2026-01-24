// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'emulation_settings.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_EmulationSettings _$EmulationSettingsFromJson(Map<String, dynamic> json) =>
    _EmulationSettings(
      integerFpsMode: json['integerFpsMode'] as bool? ?? false,
      pauseInBackground: json['pauseInBackground'] as bool? ?? false,
      autoSaveEnabled: json['autoSaveEnabled'] as bool? ?? true,
      autoSaveIntervalInMinutes:
          (json['autoSaveIntervalInMinutes'] as num?)?.toInt() ?? 1,
      quickSaveSlot: (json['quickSaveSlot'] as num?)?.toInt() ?? 1,
      fastForwardSpeedPercent:
          (json['fastForwardSpeedPercent'] as num?)?.toInt() ?? 300,
      rewindEnabled: json['rewindEnabled'] as bool? ?? true,
      rewindSeconds: (json['rewindSeconds'] as num?)?.toInt() ?? 60,
      rewindSpeedPercent: (json['rewindSpeedPercent'] as num?)?.toInt() ?? 100,
      showEmulationStatusOverlay:
          json['showEmulationStatusOverlay'] as bool? ?? true,
    );

Map<String, dynamic> _$EmulationSettingsToJson(_EmulationSettings instance) =>
    <String, dynamic>{
      'integerFpsMode': instance.integerFpsMode,
      'pauseInBackground': instance.pauseInBackground,
      'autoSaveEnabled': instance.autoSaveEnabled,
      'autoSaveIntervalInMinutes': instance.autoSaveIntervalInMinutes,
      'quickSaveSlot': instance.quickSaveSlot,
      'fastForwardSpeedPercent': instance.fastForwardSpeedPercent,
      'rewindEnabled': instance.rewindEnabled,
      'rewindSeconds': instance.rewindSeconds,
      'rewindSpeedPercent': instance.rewindSpeedPercent,
      'showEmulationStatusOverlay': instance.showEmulationStatusOverlay,
    };
