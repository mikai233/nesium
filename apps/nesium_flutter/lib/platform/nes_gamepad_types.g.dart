// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'nes_gamepad_types.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_GamepadMapping _$GamepadMappingFromJson(
  Map<String, dynamic> json,
) => _GamepadMapping(
  a: $enumDecodeNullable(_$GamepadButtonEnumMap, json['a']),
  b: $enumDecodeNullable(_$GamepadButtonEnumMap, json['b']),
  select: $enumDecodeNullable(_$GamepadButtonEnumMap, json['select']),
  start: $enumDecodeNullable(_$GamepadButtonEnumMap, json['start']),
  up: $enumDecodeNullable(_$GamepadButtonEnumMap, json['up']),
  down: $enumDecodeNullable(_$GamepadButtonEnumMap, json['down']),
  left: $enumDecodeNullable(_$GamepadButtonEnumMap, json['left']),
  right: $enumDecodeNullable(_$GamepadButtonEnumMap, json['right']),
  turboA: $enumDecodeNullable(_$GamepadButtonEnumMap, json['turboA']),
  turboB: $enumDecodeNullable(_$GamepadButtonEnumMap, json['turboB']),
  rewind: $enumDecodeNullable(_$GamepadButtonEnumMap, json['rewind']),
  fastForward: $enumDecodeNullable(_$GamepadButtonEnumMap, json['fastForward']),
  saveState: $enumDecodeNullable(_$GamepadButtonEnumMap, json['saveState']),
  loadState: $enumDecodeNullable(_$GamepadButtonEnumMap, json['loadState']),
  pause: $enumDecodeNullable(_$GamepadButtonEnumMap, json['pause']),
  fullScreen: $enumDecodeNullable(_$GamepadButtonEnumMap, json['fullScreen']),
);

Map<String, dynamic> _$GamepadMappingToJson(_GamepadMapping instance) =>
    <String, dynamic>{
      'a': instance.a,
      'b': instance.b,
      'select': instance.select,
      'start': instance.start,
      'up': instance.up,
      'down': instance.down,
      'left': instance.left,
      'right': instance.right,
      'turboA': instance.turboA,
      'turboB': instance.turboB,
      'rewind': instance.rewind,
      'fastForward': instance.fastForward,
      'saveState': instance.saveState,
      'loadState': instance.loadState,
      'pause': instance.pause,
      'fullScreen': instance.fullScreen,
    };

const _$GamepadButtonEnumMap = {
  GamepadButton.south: 'south',
  GamepadButton.east: 'east',
  GamepadButton.north: 'north',
  GamepadButton.west: 'west',
  GamepadButton.c: 'c',
  GamepadButton.z: 'z',
  GamepadButton.leftTrigger: 'leftTrigger',
  GamepadButton.leftTrigger2: 'leftTrigger2',
  GamepadButton.rightTrigger: 'rightTrigger',
  GamepadButton.rightTrigger2: 'rightTrigger2',
  GamepadButton.select: 'select',
  GamepadButton.start: 'start',
  GamepadButton.mode: 'mode',
  GamepadButton.leftThumb: 'leftThumb',
  GamepadButton.rightThumb: 'rightThumb',
  GamepadButton.dpadUp: 'dpadUp',
  GamepadButton.dpadDown: 'dpadDown',
  GamepadButton.dpadLeft: 'dpadLeft',
  GamepadButton.dpadRight: 'dpadRight',
  GamepadButton.unknown: 'unknown',
};
