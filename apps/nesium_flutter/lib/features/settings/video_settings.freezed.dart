// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'video_settings.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$VideoSettings implements DiagnosticableTreeMixin {

@JsonKey(unknownEnumValue: PaletteMode.builtin) PaletteMode get paletteMode;@JsonKey(unknownEnumValue: PaletteKind.nesdevNtsc) PaletteKind get builtinPreset;@JsonKey(unknownEnumValue: VideoFilter.none) VideoFilter get videoFilter; bool get integerScaling;@JsonKey(unknownEnumValue: NesAspectRatio.square) NesAspectRatio get aspectRatio; double get screenVerticalOffset;@NtscOptionsConverter() NtscOptions get ntscOptions;@NtscBisqwitOptionsConverter() NtscBisqwitOptions get ntscBisqwitOptions;/// LCD grid strength in `0.0..=1.0`.
 double get lcdGridStrength;/// Scanline intensity in `0.0..=1.0`.
 double get scanlineIntensity; String? get customPaletteName; bool get fullScreen;
/// Create a copy of VideoSettings
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$VideoSettingsCopyWith<VideoSettings> get copyWith => _$VideoSettingsCopyWithImpl<VideoSettings>(this as VideoSettings, _$identity);

  /// Serializes this VideoSettings to a JSON map.
  Map<String, dynamic> toJson();

@override
void debugFillProperties(DiagnosticPropertiesBuilder properties) {
  properties
    ..add(DiagnosticsProperty('type', 'VideoSettings'))
    ..add(DiagnosticsProperty('paletteMode', paletteMode))..add(DiagnosticsProperty('builtinPreset', builtinPreset))..add(DiagnosticsProperty('videoFilter', videoFilter))..add(DiagnosticsProperty('integerScaling', integerScaling))..add(DiagnosticsProperty('aspectRatio', aspectRatio))..add(DiagnosticsProperty('screenVerticalOffset', screenVerticalOffset))..add(DiagnosticsProperty('ntscOptions', ntscOptions))..add(DiagnosticsProperty('ntscBisqwitOptions', ntscBisqwitOptions))..add(DiagnosticsProperty('lcdGridStrength', lcdGridStrength))..add(DiagnosticsProperty('scanlineIntensity', scanlineIntensity))..add(DiagnosticsProperty('customPaletteName', customPaletteName))..add(DiagnosticsProperty('fullScreen', fullScreen));
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is VideoSettings&&(identical(other.paletteMode, paletteMode) || other.paletteMode == paletteMode)&&(identical(other.builtinPreset, builtinPreset) || other.builtinPreset == builtinPreset)&&(identical(other.videoFilter, videoFilter) || other.videoFilter == videoFilter)&&(identical(other.integerScaling, integerScaling) || other.integerScaling == integerScaling)&&(identical(other.aspectRatio, aspectRatio) || other.aspectRatio == aspectRatio)&&(identical(other.screenVerticalOffset, screenVerticalOffset) || other.screenVerticalOffset == screenVerticalOffset)&&(identical(other.ntscOptions, ntscOptions) || other.ntscOptions == ntscOptions)&&(identical(other.ntscBisqwitOptions, ntscBisqwitOptions) || other.ntscBisqwitOptions == ntscBisqwitOptions)&&(identical(other.lcdGridStrength, lcdGridStrength) || other.lcdGridStrength == lcdGridStrength)&&(identical(other.scanlineIntensity, scanlineIntensity) || other.scanlineIntensity == scanlineIntensity)&&(identical(other.customPaletteName, customPaletteName) || other.customPaletteName == customPaletteName)&&(identical(other.fullScreen, fullScreen) || other.fullScreen == fullScreen));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,paletteMode,builtinPreset,videoFilter,integerScaling,aspectRatio,screenVerticalOffset,ntscOptions,ntscBisqwitOptions,lcdGridStrength,scanlineIntensity,customPaletteName,fullScreen);

@override
String toString({ DiagnosticLevel minLevel = DiagnosticLevel.info }) {
  return 'VideoSettings(paletteMode: $paletteMode, builtinPreset: $builtinPreset, videoFilter: $videoFilter, integerScaling: $integerScaling, aspectRatio: $aspectRatio, screenVerticalOffset: $screenVerticalOffset, ntscOptions: $ntscOptions, ntscBisqwitOptions: $ntscBisqwitOptions, lcdGridStrength: $lcdGridStrength, scanlineIntensity: $scanlineIntensity, customPaletteName: $customPaletteName, fullScreen: $fullScreen)';
}


}

/// @nodoc
abstract mixin class $VideoSettingsCopyWith<$Res>  {
  factory $VideoSettingsCopyWith(VideoSettings value, $Res Function(VideoSettings) _then) = _$VideoSettingsCopyWithImpl;
@useResult
$Res call({
@JsonKey(unknownEnumValue: PaletteMode.builtin) PaletteMode paletteMode,@JsonKey(unknownEnumValue: PaletteKind.nesdevNtsc) PaletteKind builtinPreset,@JsonKey(unknownEnumValue: VideoFilter.none) VideoFilter videoFilter, bool integerScaling,@JsonKey(unknownEnumValue: NesAspectRatio.square) NesAspectRatio aspectRatio, double screenVerticalOffset,@NtscOptionsConverter() NtscOptions ntscOptions,@NtscBisqwitOptionsConverter() NtscBisqwitOptions ntscBisqwitOptions, double lcdGridStrength, double scanlineIntensity, String? customPaletteName, bool fullScreen
});




}
/// @nodoc
class _$VideoSettingsCopyWithImpl<$Res>
    implements $VideoSettingsCopyWith<$Res> {
  _$VideoSettingsCopyWithImpl(this._self, this._then);

  final VideoSettings _self;
  final $Res Function(VideoSettings) _then;

/// Create a copy of VideoSettings
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? paletteMode = null,Object? builtinPreset = null,Object? videoFilter = null,Object? integerScaling = null,Object? aspectRatio = null,Object? screenVerticalOffset = null,Object? ntscOptions = null,Object? ntscBisqwitOptions = null,Object? lcdGridStrength = null,Object? scanlineIntensity = null,Object? customPaletteName = freezed,Object? fullScreen = null,}) {
  return _then(_self.copyWith(
paletteMode: null == paletteMode ? _self.paletteMode : paletteMode // ignore: cast_nullable_to_non_nullable
as PaletteMode,builtinPreset: null == builtinPreset ? _self.builtinPreset : builtinPreset // ignore: cast_nullable_to_non_nullable
as PaletteKind,videoFilter: null == videoFilter ? _self.videoFilter : videoFilter // ignore: cast_nullable_to_non_nullable
as VideoFilter,integerScaling: null == integerScaling ? _self.integerScaling : integerScaling // ignore: cast_nullable_to_non_nullable
as bool,aspectRatio: null == aspectRatio ? _self.aspectRatio : aspectRatio // ignore: cast_nullable_to_non_nullable
as NesAspectRatio,screenVerticalOffset: null == screenVerticalOffset ? _self.screenVerticalOffset : screenVerticalOffset // ignore: cast_nullable_to_non_nullable
as double,ntscOptions: null == ntscOptions ? _self.ntscOptions : ntscOptions // ignore: cast_nullable_to_non_nullable
as NtscOptions,ntscBisqwitOptions: null == ntscBisqwitOptions ? _self.ntscBisqwitOptions : ntscBisqwitOptions // ignore: cast_nullable_to_non_nullable
as NtscBisqwitOptions,lcdGridStrength: null == lcdGridStrength ? _self.lcdGridStrength : lcdGridStrength // ignore: cast_nullable_to_non_nullable
as double,scanlineIntensity: null == scanlineIntensity ? _self.scanlineIntensity : scanlineIntensity // ignore: cast_nullable_to_non_nullable
as double,customPaletteName: freezed == customPaletteName ? _self.customPaletteName : customPaletteName // ignore: cast_nullable_to_non_nullable
as String?,fullScreen: null == fullScreen ? _self.fullScreen : fullScreen // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [VideoSettings].
extension VideoSettingsPatterns on VideoSettings {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _VideoSettings value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _VideoSettings() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _VideoSettings value)  $default,){
final _that = this;
switch (_that) {
case _VideoSettings():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _VideoSettings value)?  $default,){
final _that = this;
switch (_that) {
case _VideoSettings() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@JsonKey(unknownEnumValue: PaletteMode.builtin)  PaletteMode paletteMode, @JsonKey(unknownEnumValue: PaletteKind.nesdevNtsc)  PaletteKind builtinPreset, @JsonKey(unknownEnumValue: VideoFilter.none)  VideoFilter videoFilter,  bool integerScaling, @JsonKey(unknownEnumValue: NesAspectRatio.square)  NesAspectRatio aspectRatio,  double screenVerticalOffset, @NtscOptionsConverter()  NtscOptions ntscOptions, @NtscBisqwitOptionsConverter()  NtscBisqwitOptions ntscBisqwitOptions,  double lcdGridStrength,  double scanlineIntensity,  String? customPaletteName,  bool fullScreen)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _VideoSettings() when $default != null:
return $default(_that.paletteMode,_that.builtinPreset,_that.videoFilter,_that.integerScaling,_that.aspectRatio,_that.screenVerticalOffset,_that.ntscOptions,_that.ntscBisqwitOptions,_that.lcdGridStrength,_that.scanlineIntensity,_that.customPaletteName,_that.fullScreen);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@JsonKey(unknownEnumValue: PaletteMode.builtin)  PaletteMode paletteMode, @JsonKey(unknownEnumValue: PaletteKind.nesdevNtsc)  PaletteKind builtinPreset, @JsonKey(unknownEnumValue: VideoFilter.none)  VideoFilter videoFilter,  bool integerScaling, @JsonKey(unknownEnumValue: NesAspectRatio.square)  NesAspectRatio aspectRatio,  double screenVerticalOffset, @NtscOptionsConverter()  NtscOptions ntscOptions, @NtscBisqwitOptionsConverter()  NtscBisqwitOptions ntscBisqwitOptions,  double lcdGridStrength,  double scanlineIntensity,  String? customPaletteName,  bool fullScreen)  $default,) {final _that = this;
switch (_that) {
case _VideoSettings():
return $default(_that.paletteMode,_that.builtinPreset,_that.videoFilter,_that.integerScaling,_that.aspectRatio,_that.screenVerticalOffset,_that.ntscOptions,_that.ntscBisqwitOptions,_that.lcdGridStrength,_that.scanlineIntensity,_that.customPaletteName,_that.fullScreen);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@JsonKey(unknownEnumValue: PaletteMode.builtin)  PaletteMode paletteMode, @JsonKey(unknownEnumValue: PaletteKind.nesdevNtsc)  PaletteKind builtinPreset, @JsonKey(unknownEnumValue: VideoFilter.none)  VideoFilter videoFilter,  bool integerScaling, @JsonKey(unknownEnumValue: NesAspectRatio.square)  NesAspectRatio aspectRatio,  double screenVerticalOffset, @NtscOptionsConverter()  NtscOptions ntscOptions, @NtscBisqwitOptionsConverter()  NtscBisqwitOptions ntscBisqwitOptions,  double lcdGridStrength,  double scanlineIntensity,  String? customPaletteName,  bool fullScreen)?  $default,) {final _that = this;
switch (_that) {
case _VideoSettings() when $default != null:
return $default(_that.paletteMode,_that.builtinPreset,_that.videoFilter,_that.integerScaling,_that.aspectRatio,_that.screenVerticalOffset,_that.ntscOptions,_that.ntscBisqwitOptions,_that.lcdGridStrength,_that.scanlineIntensity,_that.customPaletteName,_that.fullScreen);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _VideoSettings extends VideoSettings with DiagnosticableTreeMixin {
  const _VideoSettings({@JsonKey(unknownEnumValue: PaletteMode.builtin) this.paletteMode = PaletteMode.builtin, @JsonKey(unknownEnumValue: PaletteKind.nesdevNtsc) this.builtinPreset = PaletteKind.nesdevNtsc, @JsonKey(unknownEnumValue: VideoFilter.none) this.videoFilter = VideoFilter.none, this.integerScaling = false, @JsonKey(unknownEnumValue: NesAspectRatio.square) this.aspectRatio = NesAspectRatio.square, this.screenVerticalOffset = 0, @NtscOptionsConverter() this.ntscOptions = NtscOptionsConverter._fallback, @NtscBisqwitOptionsConverter() this.ntscBisqwitOptions = NtscBisqwitOptionsConverter._fallback, this.lcdGridStrength = 1.0, this.scanlineIntensity = 0.30, this.customPaletteName, this.fullScreen = false}): super._();
  factory _VideoSettings.fromJson(Map<String, dynamic> json) => _$VideoSettingsFromJson(json);

@override@JsonKey(unknownEnumValue: PaletteMode.builtin) final  PaletteMode paletteMode;
@override@JsonKey(unknownEnumValue: PaletteKind.nesdevNtsc) final  PaletteKind builtinPreset;
@override@JsonKey(unknownEnumValue: VideoFilter.none) final  VideoFilter videoFilter;
@override@JsonKey() final  bool integerScaling;
@override@JsonKey(unknownEnumValue: NesAspectRatio.square) final  NesAspectRatio aspectRatio;
@override@JsonKey() final  double screenVerticalOffset;
@override@JsonKey()@NtscOptionsConverter() final  NtscOptions ntscOptions;
@override@JsonKey()@NtscBisqwitOptionsConverter() final  NtscBisqwitOptions ntscBisqwitOptions;
/// LCD grid strength in `0.0..=1.0`.
@override@JsonKey() final  double lcdGridStrength;
/// Scanline intensity in `0.0..=1.0`.
@override@JsonKey() final  double scanlineIntensity;
@override final  String? customPaletteName;
@override@JsonKey() final  bool fullScreen;

/// Create a copy of VideoSettings
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$VideoSettingsCopyWith<_VideoSettings> get copyWith => __$VideoSettingsCopyWithImpl<_VideoSettings>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$VideoSettingsToJson(this, );
}
@override
void debugFillProperties(DiagnosticPropertiesBuilder properties) {
  properties
    ..add(DiagnosticsProperty('type', 'VideoSettings'))
    ..add(DiagnosticsProperty('paletteMode', paletteMode))..add(DiagnosticsProperty('builtinPreset', builtinPreset))..add(DiagnosticsProperty('videoFilter', videoFilter))..add(DiagnosticsProperty('integerScaling', integerScaling))..add(DiagnosticsProperty('aspectRatio', aspectRatio))..add(DiagnosticsProperty('screenVerticalOffset', screenVerticalOffset))..add(DiagnosticsProperty('ntscOptions', ntscOptions))..add(DiagnosticsProperty('ntscBisqwitOptions', ntscBisqwitOptions))..add(DiagnosticsProperty('lcdGridStrength', lcdGridStrength))..add(DiagnosticsProperty('scanlineIntensity', scanlineIntensity))..add(DiagnosticsProperty('customPaletteName', customPaletteName))..add(DiagnosticsProperty('fullScreen', fullScreen));
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _VideoSettings&&(identical(other.paletteMode, paletteMode) || other.paletteMode == paletteMode)&&(identical(other.builtinPreset, builtinPreset) || other.builtinPreset == builtinPreset)&&(identical(other.videoFilter, videoFilter) || other.videoFilter == videoFilter)&&(identical(other.integerScaling, integerScaling) || other.integerScaling == integerScaling)&&(identical(other.aspectRatio, aspectRatio) || other.aspectRatio == aspectRatio)&&(identical(other.screenVerticalOffset, screenVerticalOffset) || other.screenVerticalOffset == screenVerticalOffset)&&(identical(other.ntscOptions, ntscOptions) || other.ntscOptions == ntscOptions)&&(identical(other.ntscBisqwitOptions, ntscBisqwitOptions) || other.ntscBisqwitOptions == ntscBisqwitOptions)&&(identical(other.lcdGridStrength, lcdGridStrength) || other.lcdGridStrength == lcdGridStrength)&&(identical(other.scanlineIntensity, scanlineIntensity) || other.scanlineIntensity == scanlineIntensity)&&(identical(other.customPaletteName, customPaletteName) || other.customPaletteName == customPaletteName)&&(identical(other.fullScreen, fullScreen) || other.fullScreen == fullScreen));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,paletteMode,builtinPreset,videoFilter,integerScaling,aspectRatio,screenVerticalOffset,ntscOptions,ntscBisqwitOptions,lcdGridStrength,scanlineIntensity,customPaletteName,fullScreen);

@override
String toString({ DiagnosticLevel minLevel = DiagnosticLevel.info }) {
  return 'VideoSettings(paletteMode: $paletteMode, builtinPreset: $builtinPreset, videoFilter: $videoFilter, integerScaling: $integerScaling, aspectRatio: $aspectRatio, screenVerticalOffset: $screenVerticalOffset, ntscOptions: $ntscOptions, ntscBisqwitOptions: $ntscBisqwitOptions, lcdGridStrength: $lcdGridStrength, scanlineIntensity: $scanlineIntensity, customPaletteName: $customPaletteName, fullScreen: $fullScreen)';
}


}

/// @nodoc
abstract mixin class _$VideoSettingsCopyWith<$Res> implements $VideoSettingsCopyWith<$Res> {
  factory _$VideoSettingsCopyWith(_VideoSettings value, $Res Function(_VideoSettings) _then) = __$VideoSettingsCopyWithImpl;
@override @useResult
$Res call({
@JsonKey(unknownEnumValue: PaletteMode.builtin) PaletteMode paletteMode,@JsonKey(unknownEnumValue: PaletteKind.nesdevNtsc) PaletteKind builtinPreset,@JsonKey(unknownEnumValue: VideoFilter.none) VideoFilter videoFilter, bool integerScaling,@JsonKey(unknownEnumValue: NesAspectRatio.square) NesAspectRatio aspectRatio, double screenVerticalOffset,@NtscOptionsConverter() NtscOptions ntscOptions,@NtscBisqwitOptionsConverter() NtscBisqwitOptions ntscBisqwitOptions, double lcdGridStrength, double scanlineIntensity, String? customPaletteName, bool fullScreen
});




}
/// @nodoc
class __$VideoSettingsCopyWithImpl<$Res>
    implements _$VideoSettingsCopyWith<$Res> {
  __$VideoSettingsCopyWithImpl(this._self, this._then);

  final _VideoSettings _self;
  final $Res Function(_VideoSettings) _then;

/// Create a copy of VideoSettings
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? paletteMode = null,Object? builtinPreset = null,Object? videoFilter = null,Object? integerScaling = null,Object? aspectRatio = null,Object? screenVerticalOffset = null,Object? ntscOptions = null,Object? ntscBisqwitOptions = null,Object? lcdGridStrength = null,Object? scanlineIntensity = null,Object? customPaletteName = freezed,Object? fullScreen = null,}) {
  return _then(_VideoSettings(
paletteMode: null == paletteMode ? _self.paletteMode : paletteMode // ignore: cast_nullable_to_non_nullable
as PaletteMode,builtinPreset: null == builtinPreset ? _self.builtinPreset : builtinPreset // ignore: cast_nullable_to_non_nullable
as PaletteKind,videoFilter: null == videoFilter ? _self.videoFilter : videoFilter // ignore: cast_nullable_to_non_nullable
as VideoFilter,integerScaling: null == integerScaling ? _self.integerScaling : integerScaling // ignore: cast_nullable_to_non_nullable
as bool,aspectRatio: null == aspectRatio ? _self.aspectRatio : aspectRatio // ignore: cast_nullable_to_non_nullable
as NesAspectRatio,screenVerticalOffset: null == screenVerticalOffset ? _self.screenVerticalOffset : screenVerticalOffset // ignore: cast_nullable_to_non_nullable
as double,ntscOptions: null == ntscOptions ? _self.ntscOptions : ntscOptions // ignore: cast_nullable_to_non_nullable
as NtscOptions,ntscBisqwitOptions: null == ntscBisqwitOptions ? _self.ntscBisqwitOptions : ntscBisqwitOptions // ignore: cast_nullable_to_non_nullable
as NtscBisqwitOptions,lcdGridStrength: null == lcdGridStrength ? _self.lcdGridStrength : lcdGridStrength // ignore: cast_nullable_to_non_nullable
as double,scanlineIntensity: null == scanlineIntensity ? _self.scanlineIntensity : scanlineIntensity // ignore: cast_nullable_to_non_nullable
as double,customPaletteName: freezed == customPaletteName ? _self.customPaletteName : customPaletteName // ignore: cast_nullable_to_non_nullable
as String?,fullScreen: null == fullScreen ? _self.fullScreen : fullScreen // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on
