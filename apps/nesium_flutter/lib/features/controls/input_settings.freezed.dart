// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'input_settings.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$InputSettings implements DiagnosticableTreeMixin {

 InputDevice get device; KeyboardPreset get keyboardPreset; LogicalKeyboardKey? get customUp; LogicalKeyboardKey? get customDown; LogicalKeyboardKey? get customLeft; LogicalKeyboardKey? get customRight; LogicalKeyboardKey? get customA; LogicalKeyboardKey? get customB; LogicalKeyboardKey? get customSelect; LogicalKeyboardKey? get customStart; LogicalKeyboardKey? get customTurboA; LogicalKeyboardKey? get customTurboB; LogicalKeyboardKey? get customRewind; LogicalKeyboardKey? get customFastForward; LogicalKeyboardKey? get customSaveState; LogicalKeyboardKey? get customLoadState; LogicalKeyboardKey? get customPause;
/// Create a copy of InputSettings
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InputSettingsCopyWith<InputSettings> get copyWith => _$InputSettingsCopyWithImpl<InputSettings>(this as InputSettings, _$identity);

  /// Serializes this InputSettings to a JSON map.
  Map<String, dynamic> toJson();

@override
void debugFillProperties(DiagnosticPropertiesBuilder properties) {
  properties
    ..add(DiagnosticsProperty('type', 'InputSettings'))
    ..add(DiagnosticsProperty('device', device))..add(DiagnosticsProperty('keyboardPreset', keyboardPreset))..add(DiagnosticsProperty('customUp', customUp))..add(DiagnosticsProperty('customDown', customDown))..add(DiagnosticsProperty('customLeft', customLeft))..add(DiagnosticsProperty('customRight', customRight))..add(DiagnosticsProperty('customA', customA))..add(DiagnosticsProperty('customB', customB))..add(DiagnosticsProperty('customSelect', customSelect))..add(DiagnosticsProperty('customStart', customStart))..add(DiagnosticsProperty('customTurboA', customTurboA))..add(DiagnosticsProperty('customTurboB', customTurboB))..add(DiagnosticsProperty('customRewind', customRewind))..add(DiagnosticsProperty('customFastForward', customFastForward))..add(DiagnosticsProperty('customSaveState', customSaveState))..add(DiagnosticsProperty('customLoadState', customLoadState))..add(DiagnosticsProperty('customPause', customPause));
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InputSettings&&(identical(other.device, device) || other.device == device)&&(identical(other.keyboardPreset, keyboardPreset) || other.keyboardPreset == keyboardPreset)&&(identical(other.customUp, customUp) || other.customUp == customUp)&&(identical(other.customDown, customDown) || other.customDown == customDown)&&(identical(other.customLeft, customLeft) || other.customLeft == customLeft)&&(identical(other.customRight, customRight) || other.customRight == customRight)&&(identical(other.customA, customA) || other.customA == customA)&&(identical(other.customB, customB) || other.customB == customB)&&(identical(other.customSelect, customSelect) || other.customSelect == customSelect)&&(identical(other.customStart, customStart) || other.customStart == customStart)&&(identical(other.customTurboA, customTurboA) || other.customTurboA == customTurboA)&&(identical(other.customTurboB, customTurboB) || other.customTurboB == customTurboB)&&(identical(other.customRewind, customRewind) || other.customRewind == customRewind)&&(identical(other.customFastForward, customFastForward) || other.customFastForward == customFastForward)&&(identical(other.customSaveState, customSaveState) || other.customSaveState == customSaveState)&&(identical(other.customLoadState, customLoadState) || other.customLoadState == customLoadState)&&(identical(other.customPause, customPause) || other.customPause == customPause));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,device,keyboardPreset,customUp,customDown,customLeft,customRight,customA,customB,customSelect,customStart,customTurboA,customTurboB,customRewind,customFastForward,customSaveState,customLoadState,customPause);

@override
String toString({ DiagnosticLevel minLevel = DiagnosticLevel.info }) {
  return 'InputSettings(device: $device, keyboardPreset: $keyboardPreset, customUp: $customUp, customDown: $customDown, customLeft: $customLeft, customRight: $customRight, customA: $customA, customB: $customB, customSelect: $customSelect, customStart: $customStart, customTurboA: $customTurboA, customTurboB: $customTurboB, customRewind: $customRewind, customFastForward: $customFastForward, customSaveState: $customSaveState, customLoadState: $customLoadState, customPause: $customPause)';
}


}

/// @nodoc
abstract mixin class $InputSettingsCopyWith<$Res>  {
  factory $InputSettingsCopyWith(InputSettings value, $Res Function(InputSettings) _then) = _$InputSettingsCopyWithImpl;
@useResult
$Res call({
 InputDevice device, KeyboardPreset keyboardPreset, LogicalKeyboardKey? customUp, LogicalKeyboardKey? customDown, LogicalKeyboardKey? customLeft, LogicalKeyboardKey? customRight, LogicalKeyboardKey? customA, LogicalKeyboardKey? customB, LogicalKeyboardKey? customSelect, LogicalKeyboardKey? customStart, LogicalKeyboardKey? customTurboA, LogicalKeyboardKey? customTurboB, LogicalKeyboardKey? customRewind, LogicalKeyboardKey? customFastForward, LogicalKeyboardKey? customSaveState, LogicalKeyboardKey? customLoadState, LogicalKeyboardKey? customPause
});




}
/// @nodoc
class _$InputSettingsCopyWithImpl<$Res>
    implements $InputSettingsCopyWith<$Res> {
  _$InputSettingsCopyWithImpl(this._self, this._then);

  final InputSettings _self;
  final $Res Function(InputSettings) _then;

/// Create a copy of InputSettings
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? device = null,Object? keyboardPreset = null,Object? customUp = freezed,Object? customDown = freezed,Object? customLeft = freezed,Object? customRight = freezed,Object? customA = freezed,Object? customB = freezed,Object? customSelect = freezed,Object? customStart = freezed,Object? customTurboA = freezed,Object? customTurboB = freezed,Object? customRewind = freezed,Object? customFastForward = freezed,Object? customSaveState = freezed,Object? customLoadState = freezed,Object? customPause = freezed,}) {
  return _then(_self.copyWith(
device: null == device ? _self.device : device // ignore: cast_nullable_to_non_nullable
as InputDevice,keyboardPreset: null == keyboardPreset ? _self.keyboardPreset : keyboardPreset // ignore: cast_nullable_to_non_nullable
as KeyboardPreset,customUp: freezed == customUp ? _self.customUp : customUp // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customDown: freezed == customDown ? _self.customDown : customDown // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customLeft: freezed == customLeft ? _self.customLeft : customLeft // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customRight: freezed == customRight ? _self.customRight : customRight // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customA: freezed == customA ? _self.customA : customA // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customB: freezed == customB ? _self.customB : customB // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customSelect: freezed == customSelect ? _self.customSelect : customSelect // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customStart: freezed == customStart ? _self.customStart : customStart // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customTurboA: freezed == customTurboA ? _self.customTurboA : customTurboA // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customTurboB: freezed == customTurboB ? _self.customTurboB : customTurboB // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customRewind: freezed == customRewind ? _self.customRewind : customRewind // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customFastForward: freezed == customFastForward ? _self.customFastForward : customFastForward // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customSaveState: freezed == customSaveState ? _self.customSaveState : customSaveState // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customLoadState: freezed == customLoadState ? _self.customLoadState : customLoadState // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customPause: freezed == customPause ? _self.customPause : customPause // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,
  ));
}

}


/// Adds pattern-matching-related methods to [InputSettings].
extension InputSettingsPatterns on InputSettings {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _InputSettings value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _InputSettings() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _InputSettings value)  $default,){
final _that = this;
switch (_that) {
case _InputSettings():
return $default(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _InputSettings value)?  $default,){
final _that = this;
switch (_that) {
case _InputSettings() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( InputDevice device,  KeyboardPreset keyboardPreset,  LogicalKeyboardKey? customUp,  LogicalKeyboardKey? customDown,  LogicalKeyboardKey? customLeft,  LogicalKeyboardKey? customRight,  LogicalKeyboardKey? customA,  LogicalKeyboardKey? customB,  LogicalKeyboardKey? customSelect,  LogicalKeyboardKey? customStart,  LogicalKeyboardKey? customTurboA,  LogicalKeyboardKey? customTurboB,  LogicalKeyboardKey? customRewind,  LogicalKeyboardKey? customFastForward,  LogicalKeyboardKey? customSaveState,  LogicalKeyboardKey? customLoadState,  LogicalKeyboardKey? customPause)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _InputSettings() when $default != null:
return $default(_that.device,_that.keyboardPreset,_that.customUp,_that.customDown,_that.customLeft,_that.customRight,_that.customA,_that.customB,_that.customSelect,_that.customStart,_that.customTurboA,_that.customTurboB,_that.customRewind,_that.customFastForward,_that.customSaveState,_that.customLoadState,_that.customPause);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( InputDevice device,  KeyboardPreset keyboardPreset,  LogicalKeyboardKey? customUp,  LogicalKeyboardKey? customDown,  LogicalKeyboardKey? customLeft,  LogicalKeyboardKey? customRight,  LogicalKeyboardKey? customA,  LogicalKeyboardKey? customB,  LogicalKeyboardKey? customSelect,  LogicalKeyboardKey? customStart,  LogicalKeyboardKey? customTurboA,  LogicalKeyboardKey? customTurboB,  LogicalKeyboardKey? customRewind,  LogicalKeyboardKey? customFastForward,  LogicalKeyboardKey? customSaveState,  LogicalKeyboardKey? customLoadState,  LogicalKeyboardKey? customPause)  $default,) {final _that = this;
switch (_that) {
case _InputSettings():
return $default(_that.device,_that.keyboardPreset,_that.customUp,_that.customDown,_that.customLeft,_that.customRight,_that.customA,_that.customB,_that.customSelect,_that.customStart,_that.customTurboA,_that.customTurboB,_that.customRewind,_that.customFastForward,_that.customSaveState,_that.customLoadState,_that.customPause);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( InputDevice device,  KeyboardPreset keyboardPreset,  LogicalKeyboardKey? customUp,  LogicalKeyboardKey? customDown,  LogicalKeyboardKey? customLeft,  LogicalKeyboardKey? customRight,  LogicalKeyboardKey? customA,  LogicalKeyboardKey? customB,  LogicalKeyboardKey? customSelect,  LogicalKeyboardKey? customStart,  LogicalKeyboardKey? customTurboA,  LogicalKeyboardKey? customTurboB,  LogicalKeyboardKey? customRewind,  LogicalKeyboardKey? customFastForward,  LogicalKeyboardKey? customSaveState,  LogicalKeyboardKey? customLoadState,  LogicalKeyboardKey? customPause)?  $default,) {final _that = this;
switch (_that) {
case _InputSettings() when $default != null:
return $default(_that.device,_that.keyboardPreset,_that.customUp,_that.customDown,_that.customLeft,_that.customRight,_that.customA,_that.customB,_that.customSelect,_that.customStart,_that.customTurboA,_that.customTurboB,_that.customRewind,_that.customFastForward,_that.customSaveState,_that.customLoadState,_that.customPause);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()
@LogicalKeyboardKeyNullableConverter()
class _InputSettings extends InputSettings with DiagnosticableTreeMixin {
  const _InputSettings({required this.device, required this.keyboardPreset, this.customUp, this.customDown, this.customLeft, this.customRight, this.customA, this.customB, this.customSelect, this.customStart, this.customTurboA, this.customTurboB, this.customRewind, this.customFastForward, this.customSaveState, this.customLoadState, this.customPause}): super._();
  factory _InputSettings.fromJson(Map<String, dynamic> json) => _$InputSettingsFromJson(json);

@override final  InputDevice device;
@override final  KeyboardPreset keyboardPreset;
@override final  LogicalKeyboardKey? customUp;
@override final  LogicalKeyboardKey? customDown;
@override final  LogicalKeyboardKey? customLeft;
@override final  LogicalKeyboardKey? customRight;
@override final  LogicalKeyboardKey? customA;
@override final  LogicalKeyboardKey? customB;
@override final  LogicalKeyboardKey? customSelect;
@override final  LogicalKeyboardKey? customStart;
@override final  LogicalKeyboardKey? customTurboA;
@override final  LogicalKeyboardKey? customTurboB;
@override final  LogicalKeyboardKey? customRewind;
@override final  LogicalKeyboardKey? customFastForward;
@override final  LogicalKeyboardKey? customSaveState;
@override final  LogicalKeyboardKey? customLoadState;
@override final  LogicalKeyboardKey? customPause;

/// Create a copy of InputSettings
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$InputSettingsCopyWith<_InputSettings> get copyWith => __$InputSettingsCopyWithImpl<_InputSettings>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$InputSettingsToJson(this, );
}
@override
void debugFillProperties(DiagnosticPropertiesBuilder properties) {
  properties
    ..add(DiagnosticsProperty('type', 'InputSettings'))
    ..add(DiagnosticsProperty('device', device))..add(DiagnosticsProperty('keyboardPreset', keyboardPreset))..add(DiagnosticsProperty('customUp', customUp))..add(DiagnosticsProperty('customDown', customDown))..add(DiagnosticsProperty('customLeft', customLeft))..add(DiagnosticsProperty('customRight', customRight))..add(DiagnosticsProperty('customA', customA))..add(DiagnosticsProperty('customB', customB))..add(DiagnosticsProperty('customSelect', customSelect))..add(DiagnosticsProperty('customStart', customStart))..add(DiagnosticsProperty('customTurboA', customTurboA))..add(DiagnosticsProperty('customTurboB', customTurboB))..add(DiagnosticsProperty('customRewind', customRewind))..add(DiagnosticsProperty('customFastForward', customFastForward))..add(DiagnosticsProperty('customSaveState', customSaveState))..add(DiagnosticsProperty('customLoadState', customLoadState))..add(DiagnosticsProperty('customPause', customPause));
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _InputSettings&&(identical(other.device, device) || other.device == device)&&(identical(other.keyboardPreset, keyboardPreset) || other.keyboardPreset == keyboardPreset)&&(identical(other.customUp, customUp) || other.customUp == customUp)&&(identical(other.customDown, customDown) || other.customDown == customDown)&&(identical(other.customLeft, customLeft) || other.customLeft == customLeft)&&(identical(other.customRight, customRight) || other.customRight == customRight)&&(identical(other.customA, customA) || other.customA == customA)&&(identical(other.customB, customB) || other.customB == customB)&&(identical(other.customSelect, customSelect) || other.customSelect == customSelect)&&(identical(other.customStart, customStart) || other.customStart == customStart)&&(identical(other.customTurboA, customTurboA) || other.customTurboA == customTurboA)&&(identical(other.customTurboB, customTurboB) || other.customTurboB == customTurboB)&&(identical(other.customRewind, customRewind) || other.customRewind == customRewind)&&(identical(other.customFastForward, customFastForward) || other.customFastForward == customFastForward)&&(identical(other.customSaveState, customSaveState) || other.customSaveState == customSaveState)&&(identical(other.customLoadState, customLoadState) || other.customLoadState == customLoadState)&&(identical(other.customPause, customPause) || other.customPause == customPause));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,device,keyboardPreset,customUp,customDown,customLeft,customRight,customA,customB,customSelect,customStart,customTurboA,customTurboB,customRewind,customFastForward,customSaveState,customLoadState,customPause);

@override
String toString({ DiagnosticLevel minLevel = DiagnosticLevel.info }) {
  return 'InputSettings(device: $device, keyboardPreset: $keyboardPreset, customUp: $customUp, customDown: $customDown, customLeft: $customLeft, customRight: $customRight, customA: $customA, customB: $customB, customSelect: $customSelect, customStart: $customStart, customTurboA: $customTurboA, customTurboB: $customTurboB, customRewind: $customRewind, customFastForward: $customFastForward, customSaveState: $customSaveState, customLoadState: $customLoadState, customPause: $customPause)';
}


}

/// @nodoc
abstract mixin class _$InputSettingsCopyWith<$Res> implements $InputSettingsCopyWith<$Res> {
  factory _$InputSettingsCopyWith(_InputSettings value, $Res Function(_InputSettings) _then) = __$InputSettingsCopyWithImpl;
@override @useResult
$Res call({
 InputDevice device, KeyboardPreset keyboardPreset, LogicalKeyboardKey? customUp, LogicalKeyboardKey? customDown, LogicalKeyboardKey? customLeft, LogicalKeyboardKey? customRight, LogicalKeyboardKey? customA, LogicalKeyboardKey? customB, LogicalKeyboardKey? customSelect, LogicalKeyboardKey? customStart, LogicalKeyboardKey? customTurboA, LogicalKeyboardKey? customTurboB, LogicalKeyboardKey? customRewind, LogicalKeyboardKey? customFastForward, LogicalKeyboardKey? customSaveState, LogicalKeyboardKey? customLoadState, LogicalKeyboardKey? customPause
});




}
/// @nodoc
class __$InputSettingsCopyWithImpl<$Res>
    implements _$InputSettingsCopyWith<$Res> {
  __$InputSettingsCopyWithImpl(this._self, this._then);

  final _InputSettings _self;
  final $Res Function(_InputSettings) _then;

/// Create a copy of InputSettings
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? device = null,Object? keyboardPreset = null,Object? customUp = freezed,Object? customDown = freezed,Object? customLeft = freezed,Object? customRight = freezed,Object? customA = freezed,Object? customB = freezed,Object? customSelect = freezed,Object? customStart = freezed,Object? customTurboA = freezed,Object? customTurboB = freezed,Object? customRewind = freezed,Object? customFastForward = freezed,Object? customSaveState = freezed,Object? customLoadState = freezed,Object? customPause = freezed,}) {
  return _then(_InputSettings(
device: null == device ? _self.device : device // ignore: cast_nullable_to_non_nullable
as InputDevice,keyboardPreset: null == keyboardPreset ? _self.keyboardPreset : keyboardPreset // ignore: cast_nullable_to_non_nullable
as KeyboardPreset,customUp: freezed == customUp ? _self.customUp : customUp // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customDown: freezed == customDown ? _self.customDown : customDown // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customLeft: freezed == customLeft ? _self.customLeft : customLeft // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customRight: freezed == customRight ? _self.customRight : customRight // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customA: freezed == customA ? _self.customA : customA // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customB: freezed == customB ? _self.customB : customB // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customSelect: freezed == customSelect ? _self.customSelect : customSelect // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customStart: freezed == customStart ? _self.customStart : customStart // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customTurboA: freezed == customTurboA ? _self.customTurboA : customTurboA // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customTurboB: freezed == customTurboB ? _self.customTurboB : customTurboB // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customRewind: freezed == customRewind ? _self.customRewind : customRewind // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customFastForward: freezed == customFastForward ? _self.customFastForward : customFastForward // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customSaveState: freezed == customSaveState ? _self.customSaveState : customSaveState // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customLoadState: freezed == customLoadState ? _self.customLoadState : customLoadState // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,customPause: freezed == customPause ? _self.customPause : customPause // ignore: cast_nullable_to_non_nullable
as LogicalKeyboardKey?,
  ));
}


}


/// @nodoc
mixin _$InputSettingsState implements DiagnosticableTreeMixin {

 Map<int, InputSettings> get ports; int get selectedPort;
/// Create a copy of InputSettingsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InputSettingsStateCopyWith<InputSettingsState> get copyWith => _$InputSettingsStateCopyWithImpl<InputSettingsState>(this as InputSettingsState, _$identity);

  /// Serializes this InputSettingsState to a JSON map.
  Map<String, dynamic> toJson();

@override
void debugFillProperties(DiagnosticPropertiesBuilder properties) {
  properties
    ..add(DiagnosticsProperty('type', 'InputSettingsState'))
    ..add(DiagnosticsProperty('ports', ports))..add(DiagnosticsProperty('selectedPort', selectedPort));
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InputSettingsState&&const DeepCollectionEquality().equals(other.ports, ports)&&(identical(other.selectedPort, selectedPort) || other.selectedPort == selectedPort));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(ports),selectedPort);

@override
String toString({ DiagnosticLevel minLevel = DiagnosticLevel.info }) {
  return 'InputSettingsState(ports: $ports, selectedPort: $selectedPort)';
}


}

/// @nodoc
abstract mixin class $InputSettingsStateCopyWith<$Res>  {
  factory $InputSettingsStateCopyWith(InputSettingsState value, $Res Function(InputSettingsState) _then) = _$InputSettingsStateCopyWithImpl;
@useResult
$Res call({
 Map<int, InputSettings> ports, int selectedPort
});




}
/// @nodoc
class _$InputSettingsStateCopyWithImpl<$Res>
    implements $InputSettingsStateCopyWith<$Res> {
  _$InputSettingsStateCopyWithImpl(this._self, this._then);

  final InputSettingsState _self;
  final $Res Function(InputSettingsState) _then;

/// Create a copy of InputSettingsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? ports = null,Object? selectedPort = null,}) {
  return _then(_self.copyWith(
ports: null == ports ? _self.ports : ports // ignore: cast_nullable_to_non_nullable
as Map<int, InputSettings>,selectedPort: null == selectedPort ? _self.selectedPort : selectedPort // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [InputSettingsState].
extension InputSettingsStatePatterns on InputSettingsState {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _InputSettingsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _InputSettingsState() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _InputSettingsState value)  $default,){
final _that = this;
switch (_that) {
case _InputSettingsState():
return $default(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _InputSettingsState value)?  $default,){
final _that = this;
switch (_that) {
case _InputSettingsState() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( Map<int, InputSettings> ports,  int selectedPort)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _InputSettingsState() when $default != null:
return $default(_that.ports,_that.selectedPort);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( Map<int, InputSettings> ports,  int selectedPort)  $default,) {final _that = this;
switch (_that) {
case _InputSettingsState():
return $default(_that.ports,_that.selectedPort);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( Map<int, InputSettings> ports,  int selectedPort)?  $default,) {final _that = this;
switch (_that) {
case _InputSettingsState() when $default != null:
return $default(_that.ports,_that.selectedPort);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _InputSettingsState extends InputSettingsState with DiagnosticableTreeMixin {
  const _InputSettingsState({required final  Map<int, InputSettings> ports, required this.selectedPort}): _ports = ports,super._();
  factory _InputSettingsState.fromJson(Map<String, dynamic> json) => _$InputSettingsStateFromJson(json);

 final  Map<int, InputSettings> _ports;
@override Map<int, InputSettings> get ports {
  if (_ports is EqualUnmodifiableMapView) return _ports;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableMapView(_ports);
}

@override final  int selectedPort;

/// Create a copy of InputSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$InputSettingsStateCopyWith<_InputSettingsState> get copyWith => __$InputSettingsStateCopyWithImpl<_InputSettingsState>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$InputSettingsStateToJson(this, );
}
@override
void debugFillProperties(DiagnosticPropertiesBuilder properties) {
  properties
    ..add(DiagnosticsProperty('type', 'InputSettingsState'))
    ..add(DiagnosticsProperty('ports', ports))..add(DiagnosticsProperty('selectedPort', selectedPort));
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _InputSettingsState&&const DeepCollectionEquality().equals(other._ports, _ports)&&(identical(other.selectedPort, selectedPort) || other.selectedPort == selectedPort));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ports),selectedPort);

@override
String toString({ DiagnosticLevel minLevel = DiagnosticLevel.info }) {
  return 'InputSettingsState(ports: $ports, selectedPort: $selectedPort)';
}


}

/// @nodoc
abstract mixin class _$InputSettingsStateCopyWith<$Res> implements $InputSettingsStateCopyWith<$Res> {
  factory _$InputSettingsStateCopyWith(_InputSettingsState value, $Res Function(_InputSettingsState) _then) = __$InputSettingsStateCopyWithImpl;
@override @useResult
$Res call({
 Map<int, InputSettings> ports, int selectedPort
});




}
/// @nodoc
class __$InputSettingsStateCopyWithImpl<$Res>
    implements _$InputSettingsStateCopyWith<$Res> {
  __$InputSettingsStateCopyWithImpl(this._self, this._then);

  final _InputSettingsState _self;
  final $Res Function(_InputSettingsState) _then;

/// Create a copy of InputSettingsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? ports = null,Object? selectedPort = null,}) {
  return _then(_InputSettingsState(
ports: null == ports ? _self._ports : ports // ignore: cast_nullable_to_non_nullable
as Map<int, InputSettings>,selectedPort: null == selectedPort ? _self.selectedPort : selectedPort // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
