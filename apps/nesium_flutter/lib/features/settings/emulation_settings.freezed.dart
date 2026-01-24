// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'emulation_settings.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$EmulationSettings implements DiagnosticableTreeMixin {

 bool get integerFpsMode; bool get pauseInBackground; bool get autoSaveEnabled; int get autoSaveIntervalInMinutes; int get quickSaveSlot; int get fastForwardSpeedPercent; bool get rewindEnabled; int get rewindSeconds; int get rewindSpeedPercent; bool get showEmulationStatusOverlay;
/// Create a copy of EmulationSettings
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EmulationSettingsCopyWith<EmulationSettings> get copyWith => _$EmulationSettingsCopyWithImpl<EmulationSettings>(this as EmulationSettings, _$identity);

  /// Serializes this EmulationSettings to a JSON map.
  Map<String, dynamic> toJson();

@override
void debugFillProperties(DiagnosticPropertiesBuilder properties) {
  properties
    ..add(DiagnosticsProperty('type', 'EmulationSettings'))
    ..add(DiagnosticsProperty('integerFpsMode', integerFpsMode))..add(DiagnosticsProperty('pauseInBackground', pauseInBackground))..add(DiagnosticsProperty('autoSaveEnabled', autoSaveEnabled))..add(DiagnosticsProperty('autoSaveIntervalInMinutes', autoSaveIntervalInMinutes))..add(DiagnosticsProperty('quickSaveSlot', quickSaveSlot))..add(DiagnosticsProperty('fastForwardSpeedPercent', fastForwardSpeedPercent))..add(DiagnosticsProperty('rewindEnabled', rewindEnabled))..add(DiagnosticsProperty('rewindSeconds', rewindSeconds))..add(DiagnosticsProperty('rewindSpeedPercent', rewindSpeedPercent))..add(DiagnosticsProperty('showEmulationStatusOverlay', showEmulationStatusOverlay));
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EmulationSettings&&(identical(other.integerFpsMode, integerFpsMode) || other.integerFpsMode == integerFpsMode)&&(identical(other.pauseInBackground, pauseInBackground) || other.pauseInBackground == pauseInBackground)&&(identical(other.autoSaveEnabled, autoSaveEnabled) || other.autoSaveEnabled == autoSaveEnabled)&&(identical(other.autoSaveIntervalInMinutes, autoSaveIntervalInMinutes) || other.autoSaveIntervalInMinutes == autoSaveIntervalInMinutes)&&(identical(other.quickSaveSlot, quickSaveSlot) || other.quickSaveSlot == quickSaveSlot)&&(identical(other.fastForwardSpeedPercent, fastForwardSpeedPercent) || other.fastForwardSpeedPercent == fastForwardSpeedPercent)&&(identical(other.rewindEnabled, rewindEnabled) || other.rewindEnabled == rewindEnabled)&&(identical(other.rewindSeconds, rewindSeconds) || other.rewindSeconds == rewindSeconds)&&(identical(other.rewindSpeedPercent, rewindSpeedPercent) || other.rewindSpeedPercent == rewindSpeedPercent)&&(identical(other.showEmulationStatusOverlay, showEmulationStatusOverlay) || other.showEmulationStatusOverlay == showEmulationStatusOverlay));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,integerFpsMode,pauseInBackground,autoSaveEnabled,autoSaveIntervalInMinutes,quickSaveSlot,fastForwardSpeedPercent,rewindEnabled,rewindSeconds,rewindSpeedPercent,showEmulationStatusOverlay);

@override
String toString({ DiagnosticLevel minLevel = DiagnosticLevel.info }) {
  return 'EmulationSettings(integerFpsMode: $integerFpsMode, pauseInBackground: $pauseInBackground, autoSaveEnabled: $autoSaveEnabled, autoSaveIntervalInMinutes: $autoSaveIntervalInMinutes, quickSaveSlot: $quickSaveSlot, fastForwardSpeedPercent: $fastForwardSpeedPercent, rewindEnabled: $rewindEnabled, rewindSeconds: $rewindSeconds, rewindSpeedPercent: $rewindSpeedPercent, showEmulationStatusOverlay: $showEmulationStatusOverlay)';
}


}

/// @nodoc
abstract mixin class $EmulationSettingsCopyWith<$Res>  {
  factory $EmulationSettingsCopyWith(EmulationSettings value, $Res Function(EmulationSettings) _then) = _$EmulationSettingsCopyWithImpl;
@useResult
$Res call({
 bool integerFpsMode, bool pauseInBackground, bool autoSaveEnabled, int autoSaveIntervalInMinutes, int quickSaveSlot, int fastForwardSpeedPercent, bool rewindEnabled, int rewindSeconds, int rewindSpeedPercent, bool showEmulationStatusOverlay
});




}
/// @nodoc
class _$EmulationSettingsCopyWithImpl<$Res>
    implements $EmulationSettingsCopyWith<$Res> {
  _$EmulationSettingsCopyWithImpl(this._self, this._then);

  final EmulationSettings _self;
  final $Res Function(EmulationSettings) _then;

/// Create a copy of EmulationSettings
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? integerFpsMode = null,Object? pauseInBackground = null,Object? autoSaveEnabled = null,Object? autoSaveIntervalInMinutes = null,Object? quickSaveSlot = null,Object? fastForwardSpeedPercent = null,Object? rewindEnabled = null,Object? rewindSeconds = null,Object? rewindSpeedPercent = null,Object? showEmulationStatusOverlay = null,}) {
  return _then(_self.copyWith(
integerFpsMode: null == integerFpsMode ? _self.integerFpsMode : integerFpsMode // ignore: cast_nullable_to_non_nullable
as bool,pauseInBackground: null == pauseInBackground ? _self.pauseInBackground : pauseInBackground // ignore: cast_nullable_to_non_nullable
as bool,autoSaveEnabled: null == autoSaveEnabled ? _self.autoSaveEnabled : autoSaveEnabled // ignore: cast_nullable_to_non_nullable
as bool,autoSaveIntervalInMinutes: null == autoSaveIntervalInMinutes ? _self.autoSaveIntervalInMinutes : autoSaveIntervalInMinutes // ignore: cast_nullable_to_non_nullable
as int,quickSaveSlot: null == quickSaveSlot ? _self.quickSaveSlot : quickSaveSlot // ignore: cast_nullable_to_non_nullable
as int,fastForwardSpeedPercent: null == fastForwardSpeedPercent ? _self.fastForwardSpeedPercent : fastForwardSpeedPercent // ignore: cast_nullable_to_non_nullable
as int,rewindEnabled: null == rewindEnabled ? _self.rewindEnabled : rewindEnabled // ignore: cast_nullable_to_non_nullable
as bool,rewindSeconds: null == rewindSeconds ? _self.rewindSeconds : rewindSeconds // ignore: cast_nullable_to_non_nullable
as int,rewindSpeedPercent: null == rewindSpeedPercent ? _self.rewindSpeedPercent : rewindSpeedPercent // ignore: cast_nullable_to_non_nullable
as int,showEmulationStatusOverlay: null == showEmulationStatusOverlay ? _self.showEmulationStatusOverlay : showEmulationStatusOverlay // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [EmulationSettings].
extension EmulationSettingsPatterns on EmulationSettings {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _EmulationSettings value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _EmulationSettings() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _EmulationSettings value)  $default,){
final _that = this;
switch (_that) {
case _EmulationSettings():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _EmulationSettings value)?  $default,){
final _that = this;
switch (_that) {
case _EmulationSettings() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool integerFpsMode,  bool pauseInBackground,  bool autoSaveEnabled,  int autoSaveIntervalInMinutes,  int quickSaveSlot,  int fastForwardSpeedPercent,  bool rewindEnabled,  int rewindSeconds,  int rewindSpeedPercent,  bool showEmulationStatusOverlay)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _EmulationSettings() when $default != null:
return $default(_that.integerFpsMode,_that.pauseInBackground,_that.autoSaveEnabled,_that.autoSaveIntervalInMinutes,_that.quickSaveSlot,_that.fastForwardSpeedPercent,_that.rewindEnabled,_that.rewindSeconds,_that.rewindSpeedPercent,_that.showEmulationStatusOverlay);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool integerFpsMode,  bool pauseInBackground,  bool autoSaveEnabled,  int autoSaveIntervalInMinutes,  int quickSaveSlot,  int fastForwardSpeedPercent,  bool rewindEnabled,  int rewindSeconds,  int rewindSpeedPercent,  bool showEmulationStatusOverlay)  $default,) {final _that = this;
switch (_that) {
case _EmulationSettings():
return $default(_that.integerFpsMode,_that.pauseInBackground,_that.autoSaveEnabled,_that.autoSaveIntervalInMinutes,_that.quickSaveSlot,_that.fastForwardSpeedPercent,_that.rewindEnabled,_that.rewindSeconds,_that.rewindSpeedPercent,_that.showEmulationStatusOverlay);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool integerFpsMode,  bool pauseInBackground,  bool autoSaveEnabled,  int autoSaveIntervalInMinutes,  int quickSaveSlot,  int fastForwardSpeedPercent,  bool rewindEnabled,  int rewindSeconds,  int rewindSpeedPercent,  bool showEmulationStatusOverlay)?  $default,) {final _that = this;
switch (_that) {
case _EmulationSettings() when $default != null:
return $default(_that.integerFpsMode,_that.pauseInBackground,_that.autoSaveEnabled,_that.autoSaveIntervalInMinutes,_that.quickSaveSlot,_that.fastForwardSpeedPercent,_that.rewindEnabled,_that.rewindSeconds,_that.rewindSpeedPercent,_that.showEmulationStatusOverlay);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _EmulationSettings extends EmulationSettings with DiagnosticableTreeMixin {
  const _EmulationSettings({this.integerFpsMode = false, this.pauseInBackground = false, this.autoSaveEnabled = true, this.autoSaveIntervalInMinutes = 1, this.quickSaveSlot = 1, this.fastForwardSpeedPercent = 300, this.rewindEnabled = true, this.rewindSeconds = 60, this.rewindSpeedPercent = 100, this.showEmulationStatusOverlay = true}): super._();
  factory _EmulationSettings.fromJson(Map<String, dynamic> json) => _$EmulationSettingsFromJson(json);

@override@JsonKey() final  bool integerFpsMode;
@override@JsonKey() final  bool pauseInBackground;
@override@JsonKey() final  bool autoSaveEnabled;
@override@JsonKey() final  int autoSaveIntervalInMinutes;
@override@JsonKey() final  int quickSaveSlot;
@override@JsonKey() final  int fastForwardSpeedPercent;
@override@JsonKey() final  bool rewindEnabled;
@override@JsonKey() final  int rewindSeconds;
@override@JsonKey() final  int rewindSpeedPercent;
@override@JsonKey() final  bool showEmulationStatusOverlay;

/// Create a copy of EmulationSettings
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$EmulationSettingsCopyWith<_EmulationSettings> get copyWith => __$EmulationSettingsCopyWithImpl<_EmulationSettings>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$EmulationSettingsToJson(this, );
}
@override
void debugFillProperties(DiagnosticPropertiesBuilder properties) {
  properties
    ..add(DiagnosticsProperty('type', 'EmulationSettings'))
    ..add(DiagnosticsProperty('integerFpsMode', integerFpsMode))..add(DiagnosticsProperty('pauseInBackground', pauseInBackground))..add(DiagnosticsProperty('autoSaveEnabled', autoSaveEnabled))..add(DiagnosticsProperty('autoSaveIntervalInMinutes', autoSaveIntervalInMinutes))..add(DiagnosticsProperty('quickSaveSlot', quickSaveSlot))..add(DiagnosticsProperty('fastForwardSpeedPercent', fastForwardSpeedPercent))..add(DiagnosticsProperty('rewindEnabled', rewindEnabled))..add(DiagnosticsProperty('rewindSeconds', rewindSeconds))..add(DiagnosticsProperty('rewindSpeedPercent', rewindSpeedPercent))..add(DiagnosticsProperty('showEmulationStatusOverlay', showEmulationStatusOverlay));
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _EmulationSettings&&(identical(other.integerFpsMode, integerFpsMode) || other.integerFpsMode == integerFpsMode)&&(identical(other.pauseInBackground, pauseInBackground) || other.pauseInBackground == pauseInBackground)&&(identical(other.autoSaveEnabled, autoSaveEnabled) || other.autoSaveEnabled == autoSaveEnabled)&&(identical(other.autoSaveIntervalInMinutes, autoSaveIntervalInMinutes) || other.autoSaveIntervalInMinutes == autoSaveIntervalInMinutes)&&(identical(other.quickSaveSlot, quickSaveSlot) || other.quickSaveSlot == quickSaveSlot)&&(identical(other.fastForwardSpeedPercent, fastForwardSpeedPercent) || other.fastForwardSpeedPercent == fastForwardSpeedPercent)&&(identical(other.rewindEnabled, rewindEnabled) || other.rewindEnabled == rewindEnabled)&&(identical(other.rewindSeconds, rewindSeconds) || other.rewindSeconds == rewindSeconds)&&(identical(other.rewindSpeedPercent, rewindSpeedPercent) || other.rewindSpeedPercent == rewindSpeedPercent)&&(identical(other.showEmulationStatusOverlay, showEmulationStatusOverlay) || other.showEmulationStatusOverlay == showEmulationStatusOverlay));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,integerFpsMode,pauseInBackground,autoSaveEnabled,autoSaveIntervalInMinutes,quickSaveSlot,fastForwardSpeedPercent,rewindEnabled,rewindSeconds,rewindSpeedPercent,showEmulationStatusOverlay);

@override
String toString({ DiagnosticLevel minLevel = DiagnosticLevel.info }) {
  return 'EmulationSettings(integerFpsMode: $integerFpsMode, pauseInBackground: $pauseInBackground, autoSaveEnabled: $autoSaveEnabled, autoSaveIntervalInMinutes: $autoSaveIntervalInMinutes, quickSaveSlot: $quickSaveSlot, fastForwardSpeedPercent: $fastForwardSpeedPercent, rewindEnabled: $rewindEnabled, rewindSeconds: $rewindSeconds, rewindSpeedPercent: $rewindSpeedPercent, showEmulationStatusOverlay: $showEmulationStatusOverlay)';
}


}

/// @nodoc
abstract mixin class _$EmulationSettingsCopyWith<$Res> implements $EmulationSettingsCopyWith<$Res> {
  factory _$EmulationSettingsCopyWith(_EmulationSettings value, $Res Function(_EmulationSettings) _then) = __$EmulationSettingsCopyWithImpl;
@override @useResult
$Res call({
 bool integerFpsMode, bool pauseInBackground, bool autoSaveEnabled, int autoSaveIntervalInMinutes, int quickSaveSlot, int fastForwardSpeedPercent, bool rewindEnabled, int rewindSeconds, int rewindSpeedPercent, bool showEmulationStatusOverlay
});




}
/// @nodoc
class __$EmulationSettingsCopyWithImpl<$Res>
    implements _$EmulationSettingsCopyWith<$Res> {
  __$EmulationSettingsCopyWithImpl(this._self, this._then);

  final _EmulationSettings _self;
  final $Res Function(_EmulationSettings) _then;

/// Create a copy of EmulationSettings
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? integerFpsMode = null,Object? pauseInBackground = null,Object? autoSaveEnabled = null,Object? autoSaveIntervalInMinutes = null,Object? quickSaveSlot = null,Object? fastForwardSpeedPercent = null,Object? rewindEnabled = null,Object? rewindSeconds = null,Object? rewindSpeedPercent = null,Object? showEmulationStatusOverlay = null,}) {
  return _then(_EmulationSettings(
integerFpsMode: null == integerFpsMode ? _self.integerFpsMode : integerFpsMode // ignore: cast_nullable_to_non_nullable
as bool,pauseInBackground: null == pauseInBackground ? _self.pauseInBackground : pauseInBackground // ignore: cast_nullable_to_non_nullable
as bool,autoSaveEnabled: null == autoSaveEnabled ? _self.autoSaveEnabled : autoSaveEnabled // ignore: cast_nullable_to_non_nullable
as bool,autoSaveIntervalInMinutes: null == autoSaveIntervalInMinutes ? _self.autoSaveIntervalInMinutes : autoSaveIntervalInMinutes // ignore: cast_nullable_to_non_nullable
as int,quickSaveSlot: null == quickSaveSlot ? _self.quickSaveSlot : quickSaveSlot // ignore: cast_nullable_to_non_nullable
as int,fastForwardSpeedPercent: null == fastForwardSpeedPercent ? _self.fastForwardSpeedPercent : fastForwardSpeedPercent // ignore: cast_nullable_to_non_nullable
as int,rewindEnabled: null == rewindEnabled ? _self.rewindEnabled : rewindEnabled // ignore: cast_nullable_to_non_nullable
as bool,rewindSeconds: null == rewindSeconds ? _self.rewindSeconds : rewindSeconds // ignore: cast_nullable_to_non_nullable
as int,rewindSpeedPercent: null == rewindSpeedPercent ? _self.rewindSpeedPercent : rewindSpeedPercent // ignore: cast_nullable_to_non_nullable
as int,showEmulationStatusOverlay: null == showEmulationStatusOverlay ? _self.showEmulationStatusOverlay : showEmulationStatusOverlay // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
