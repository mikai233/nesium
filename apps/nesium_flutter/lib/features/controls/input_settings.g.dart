// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'input_settings.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_InputSettings _$InputSettingsFromJson(Map<String, dynamic> json) =>
    _InputSettings(
      device: $enumDecode(_$InputDeviceEnumMap, json['device']),
      keyboardPreset: $enumDecode(
        _$KeyboardPresetEnumMap,
        json['keyboardPreset'],
      ),
      customUp: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customUp'] as num?)?.toInt(),
      ),
      customDown: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customDown'] as num?)?.toInt(),
      ),
      customLeft: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customLeft'] as num?)?.toInt(),
      ),
      customRight: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customRight'] as num?)?.toInt(),
      ),
      customA: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customA'] as num?)?.toInt(),
      ),
      customB: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customB'] as num?)?.toInt(),
      ),
      customSelect: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customSelect'] as num?)?.toInt(),
      ),
      customStart: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customStart'] as num?)?.toInt(),
      ),
      customTurboA: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customTurboA'] as num?)?.toInt(),
      ),
      customTurboB: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customTurboB'] as num?)?.toInt(),
      ),
      customRewind: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customRewind'] as num?)?.toInt(),
      ),
      customFastForward: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customFastForward'] as num?)?.toInt(),
      ),
      customSaveState: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customSaveState'] as num?)?.toInt(),
      ),
      customLoadState: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customLoadState'] as num?)?.toInt(),
      ),
      customPause: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customPause'] as num?)?.toInt(),
      ),
      customFullScreen: const LogicalKeyboardKeyNullableConverter().fromJson(
        (json['customFullScreen'] as num?)?.toInt(),
      ),
    );

Map<String, dynamic> _$InputSettingsToJson(_InputSettings instance) =>
    <String, dynamic>{
      'device': _$InputDeviceEnumMap[instance.device]!,
      'keyboardPreset': _$KeyboardPresetEnumMap[instance.keyboardPreset]!,
      'customUp': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customUp,
      ),
      'customDown': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customDown,
      ),
      'customLeft': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customLeft,
      ),
      'customRight': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customRight,
      ),
      'customA': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customA,
      ),
      'customB': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customB,
      ),
      'customSelect': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customSelect,
      ),
      'customStart': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customStart,
      ),
      'customTurboA': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customTurboA,
      ),
      'customTurboB': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customTurboB,
      ),
      'customRewind': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customRewind,
      ),
      'customFastForward': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customFastForward,
      ),
      'customSaveState': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customSaveState,
      ),
      'customLoadState': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customLoadState,
      ),
      'customPause': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customPause,
      ),
      'customFullScreen': const LogicalKeyboardKeyNullableConverter().toJson(
        instance.customFullScreen,
      ),
    };

const _$InputDeviceEnumMap = {
  InputDevice.keyboard: 'keyboard',
  InputDevice.gamepad: 'gamepad',
  InputDevice.virtualController: 'virtualController',
};

const _$KeyboardPresetEnumMap = {
  KeyboardPreset.none: 'none',
  KeyboardPreset.nesStandard: 'nesStandard',
  KeyboardPreset.fightStick: 'fightStick',
  KeyboardPreset.arcadeLayout: 'arcadeLayout',
  KeyboardPreset.custom: 'custom',
};

_InputSettingsState _$InputSettingsStateFromJson(Map<String, dynamic> json) =>
    _InputSettingsState(
      ports: (json['ports'] as Map<String, dynamic>).map(
        (k, e) => MapEntry(
          int.parse(k),
          InputSettings.fromJson(e as Map<String, dynamic>),
        ),
      ),
      selectedPort: (json['selectedPort'] as num).toInt(),
    );

Map<String, dynamic> _$InputSettingsStateToJson(_InputSettingsState instance) =>
    <String, dynamic>{
      'ports': instance.ports.map((k, e) => MapEntry(k.toString(), e)),
      'selectedPort': instance.selectedPort,
    };
