// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'nes_gamepad_types.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$GamepadMapping {

 GamepadButton? get a; GamepadButton? get b; GamepadButton? get select; GamepadButton? get start; GamepadButton? get up; GamepadButton? get down; GamepadButton? get left; GamepadButton? get right; GamepadButton? get turboA; GamepadButton? get turboB;// Extended actions
 GamepadButton? get rewind; GamepadButton? get fastForward; GamepadButton? get saveState; GamepadButton? get loadState; GamepadButton? get pause; GamepadButton? get fullScreen;
/// Create a copy of GamepadMapping
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$GamepadMappingCopyWith<GamepadMapping> get copyWith => _$GamepadMappingCopyWithImpl<GamepadMapping>(this as GamepadMapping, _$identity);

  /// Serializes this GamepadMapping to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GamepadMapping&&(identical(other.a, a) || other.a == a)&&(identical(other.b, b) || other.b == b)&&(identical(other.select, select) || other.select == select)&&(identical(other.start, start) || other.start == start)&&(identical(other.up, up) || other.up == up)&&(identical(other.down, down) || other.down == down)&&(identical(other.left, left) || other.left == left)&&(identical(other.right, right) || other.right == right)&&(identical(other.turboA, turboA) || other.turboA == turboA)&&(identical(other.turboB, turboB) || other.turboB == turboB)&&(identical(other.rewind, rewind) || other.rewind == rewind)&&(identical(other.fastForward, fastForward) || other.fastForward == fastForward)&&(identical(other.saveState, saveState) || other.saveState == saveState)&&(identical(other.loadState, loadState) || other.loadState == loadState)&&(identical(other.pause, pause) || other.pause == pause)&&(identical(other.fullScreen, fullScreen) || other.fullScreen == fullScreen));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,a,b,select,start,up,down,left,right,turboA,turboB,rewind,fastForward,saveState,loadState,pause,fullScreen);

@override
String toString() {
  return 'GamepadMapping(a: $a, b: $b, select: $select, start: $start, up: $up, down: $down, left: $left, right: $right, turboA: $turboA, turboB: $turboB, rewind: $rewind, fastForward: $fastForward, saveState: $saveState, loadState: $loadState, pause: $pause, fullScreen: $fullScreen)';
}


}

/// @nodoc
abstract mixin class $GamepadMappingCopyWith<$Res>  {
  factory $GamepadMappingCopyWith(GamepadMapping value, $Res Function(GamepadMapping) _then) = _$GamepadMappingCopyWithImpl;
@useResult
$Res call({
 GamepadButton? a, GamepadButton? b, GamepadButton? select, GamepadButton? start, GamepadButton? up, GamepadButton? down, GamepadButton? left, GamepadButton? right, GamepadButton? turboA, GamepadButton? turboB, GamepadButton? rewind, GamepadButton? fastForward, GamepadButton? saveState, GamepadButton? loadState, GamepadButton? pause, GamepadButton? fullScreen
});




}
/// @nodoc
class _$GamepadMappingCopyWithImpl<$Res>
    implements $GamepadMappingCopyWith<$Res> {
  _$GamepadMappingCopyWithImpl(this._self, this._then);

  final GamepadMapping _self;
  final $Res Function(GamepadMapping) _then;

/// Create a copy of GamepadMapping
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? a = freezed,Object? b = freezed,Object? select = freezed,Object? start = freezed,Object? up = freezed,Object? down = freezed,Object? left = freezed,Object? right = freezed,Object? turboA = freezed,Object? turboB = freezed,Object? rewind = freezed,Object? fastForward = freezed,Object? saveState = freezed,Object? loadState = freezed,Object? pause = freezed,Object? fullScreen = freezed,}) {
  return _then(_self.copyWith(
a: freezed == a ? _self.a : a // ignore: cast_nullable_to_non_nullable
as GamepadButton?,b: freezed == b ? _self.b : b // ignore: cast_nullable_to_non_nullable
as GamepadButton?,select: freezed == select ? _self.select : select // ignore: cast_nullable_to_non_nullable
as GamepadButton?,start: freezed == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as GamepadButton?,up: freezed == up ? _self.up : up // ignore: cast_nullable_to_non_nullable
as GamepadButton?,down: freezed == down ? _self.down : down // ignore: cast_nullable_to_non_nullable
as GamepadButton?,left: freezed == left ? _self.left : left // ignore: cast_nullable_to_non_nullable
as GamepadButton?,right: freezed == right ? _self.right : right // ignore: cast_nullable_to_non_nullable
as GamepadButton?,turboA: freezed == turboA ? _self.turboA : turboA // ignore: cast_nullable_to_non_nullable
as GamepadButton?,turboB: freezed == turboB ? _self.turboB : turboB // ignore: cast_nullable_to_non_nullable
as GamepadButton?,rewind: freezed == rewind ? _self.rewind : rewind // ignore: cast_nullable_to_non_nullable
as GamepadButton?,fastForward: freezed == fastForward ? _self.fastForward : fastForward // ignore: cast_nullable_to_non_nullable
as GamepadButton?,saveState: freezed == saveState ? _self.saveState : saveState // ignore: cast_nullable_to_non_nullable
as GamepadButton?,loadState: freezed == loadState ? _self.loadState : loadState // ignore: cast_nullable_to_non_nullable
as GamepadButton?,pause: freezed == pause ? _self.pause : pause // ignore: cast_nullable_to_non_nullable
as GamepadButton?,fullScreen: freezed == fullScreen ? _self.fullScreen : fullScreen // ignore: cast_nullable_to_non_nullable
as GamepadButton?,
  ));
}

}


/// Adds pattern-matching-related methods to [GamepadMapping].
extension GamepadMappingPatterns on GamepadMapping {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _GamepadMapping value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _GamepadMapping() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _GamepadMapping value)  $default,){
final _that = this;
switch (_that) {
case _GamepadMapping():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _GamepadMapping value)?  $default,){
final _that = this;
switch (_that) {
case _GamepadMapping() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( GamepadButton? a,  GamepadButton? b,  GamepadButton? select,  GamepadButton? start,  GamepadButton? up,  GamepadButton? down,  GamepadButton? left,  GamepadButton? right,  GamepadButton? turboA,  GamepadButton? turboB,  GamepadButton? rewind,  GamepadButton? fastForward,  GamepadButton? saveState,  GamepadButton? loadState,  GamepadButton? pause,  GamepadButton? fullScreen)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _GamepadMapping() when $default != null:
return $default(_that.a,_that.b,_that.select,_that.start,_that.up,_that.down,_that.left,_that.right,_that.turboA,_that.turboB,_that.rewind,_that.fastForward,_that.saveState,_that.loadState,_that.pause,_that.fullScreen);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( GamepadButton? a,  GamepadButton? b,  GamepadButton? select,  GamepadButton? start,  GamepadButton? up,  GamepadButton? down,  GamepadButton? left,  GamepadButton? right,  GamepadButton? turboA,  GamepadButton? turboB,  GamepadButton? rewind,  GamepadButton? fastForward,  GamepadButton? saveState,  GamepadButton? loadState,  GamepadButton? pause,  GamepadButton? fullScreen)  $default,) {final _that = this;
switch (_that) {
case _GamepadMapping():
return $default(_that.a,_that.b,_that.select,_that.start,_that.up,_that.down,_that.left,_that.right,_that.turboA,_that.turboB,_that.rewind,_that.fastForward,_that.saveState,_that.loadState,_that.pause,_that.fullScreen);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( GamepadButton? a,  GamepadButton? b,  GamepadButton? select,  GamepadButton? start,  GamepadButton? up,  GamepadButton? down,  GamepadButton? left,  GamepadButton? right,  GamepadButton? turboA,  GamepadButton? turboB,  GamepadButton? rewind,  GamepadButton? fastForward,  GamepadButton? saveState,  GamepadButton? loadState,  GamepadButton? pause,  GamepadButton? fullScreen)?  $default,) {final _that = this;
switch (_that) {
case _GamepadMapping() when $default != null:
return $default(_that.a,_that.b,_that.select,_that.start,_that.up,_that.down,_that.left,_that.right,_that.turboA,_that.turboB,_that.rewind,_that.fastForward,_that.saveState,_that.loadState,_that.pause,_that.fullScreen);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _GamepadMapping implements GamepadMapping {
  const _GamepadMapping({required this.a, required this.b, required this.select, required this.start, required this.up, required this.down, required this.left, required this.right, required this.turboA, required this.turboB, this.rewind, this.fastForward, this.saveState, this.loadState, this.pause, this.fullScreen});
  factory _GamepadMapping.fromJson(Map<String, dynamic> json) => _$GamepadMappingFromJson(json);

@override final  GamepadButton? a;
@override final  GamepadButton? b;
@override final  GamepadButton? select;
@override final  GamepadButton? start;
@override final  GamepadButton? up;
@override final  GamepadButton? down;
@override final  GamepadButton? left;
@override final  GamepadButton? right;
@override final  GamepadButton? turboA;
@override final  GamepadButton? turboB;
// Extended actions
@override final  GamepadButton? rewind;
@override final  GamepadButton? fastForward;
@override final  GamepadButton? saveState;
@override final  GamepadButton? loadState;
@override final  GamepadButton? pause;
@override final  GamepadButton? fullScreen;

/// Create a copy of GamepadMapping
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$GamepadMappingCopyWith<_GamepadMapping> get copyWith => __$GamepadMappingCopyWithImpl<_GamepadMapping>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$GamepadMappingToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _GamepadMapping&&(identical(other.a, a) || other.a == a)&&(identical(other.b, b) || other.b == b)&&(identical(other.select, select) || other.select == select)&&(identical(other.start, start) || other.start == start)&&(identical(other.up, up) || other.up == up)&&(identical(other.down, down) || other.down == down)&&(identical(other.left, left) || other.left == left)&&(identical(other.right, right) || other.right == right)&&(identical(other.turboA, turboA) || other.turboA == turboA)&&(identical(other.turboB, turboB) || other.turboB == turboB)&&(identical(other.rewind, rewind) || other.rewind == rewind)&&(identical(other.fastForward, fastForward) || other.fastForward == fastForward)&&(identical(other.saveState, saveState) || other.saveState == saveState)&&(identical(other.loadState, loadState) || other.loadState == loadState)&&(identical(other.pause, pause) || other.pause == pause)&&(identical(other.fullScreen, fullScreen) || other.fullScreen == fullScreen));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,a,b,select,start,up,down,left,right,turboA,turboB,rewind,fastForward,saveState,loadState,pause,fullScreen);

@override
String toString() {
  return 'GamepadMapping(a: $a, b: $b, select: $select, start: $start, up: $up, down: $down, left: $left, right: $right, turboA: $turboA, turboB: $turboB, rewind: $rewind, fastForward: $fastForward, saveState: $saveState, loadState: $loadState, pause: $pause, fullScreen: $fullScreen)';
}


}

/// @nodoc
abstract mixin class _$GamepadMappingCopyWith<$Res> implements $GamepadMappingCopyWith<$Res> {
  factory _$GamepadMappingCopyWith(_GamepadMapping value, $Res Function(_GamepadMapping) _then) = __$GamepadMappingCopyWithImpl;
@override @useResult
$Res call({
 GamepadButton? a, GamepadButton? b, GamepadButton? select, GamepadButton? start, GamepadButton? up, GamepadButton? down, GamepadButton? left, GamepadButton? right, GamepadButton? turboA, GamepadButton? turboB, GamepadButton? rewind, GamepadButton? fastForward, GamepadButton? saveState, GamepadButton? loadState, GamepadButton? pause, GamepadButton? fullScreen
});




}
/// @nodoc
class __$GamepadMappingCopyWithImpl<$Res>
    implements _$GamepadMappingCopyWith<$Res> {
  __$GamepadMappingCopyWithImpl(this._self, this._then);

  final _GamepadMapping _self;
  final $Res Function(_GamepadMapping) _then;

/// Create a copy of GamepadMapping
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? a = freezed,Object? b = freezed,Object? select = freezed,Object? start = freezed,Object? up = freezed,Object? down = freezed,Object? left = freezed,Object? right = freezed,Object? turboA = freezed,Object? turboB = freezed,Object? rewind = freezed,Object? fastForward = freezed,Object? saveState = freezed,Object? loadState = freezed,Object? pause = freezed,Object? fullScreen = freezed,}) {
  return _then(_GamepadMapping(
a: freezed == a ? _self.a : a // ignore: cast_nullable_to_non_nullable
as GamepadButton?,b: freezed == b ? _self.b : b // ignore: cast_nullable_to_non_nullable
as GamepadButton?,select: freezed == select ? _self.select : select // ignore: cast_nullable_to_non_nullable
as GamepadButton?,start: freezed == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as GamepadButton?,up: freezed == up ? _self.up : up // ignore: cast_nullable_to_non_nullable
as GamepadButton?,down: freezed == down ? _self.down : down // ignore: cast_nullable_to_non_nullable
as GamepadButton?,left: freezed == left ? _self.left : left // ignore: cast_nullable_to_non_nullable
as GamepadButton?,right: freezed == right ? _self.right : right // ignore: cast_nullable_to_non_nullable
as GamepadButton?,turboA: freezed == turboA ? _self.turboA : turboA // ignore: cast_nullable_to_non_nullable
as GamepadButton?,turboB: freezed == turboB ? _self.turboB : turboB // ignore: cast_nullable_to_non_nullable
as GamepadButton?,rewind: freezed == rewind ? _self.rewind : rewind // ignore: cast_nullable_to_non_nullable
as GamepadButton?,fastForward: freezed == fastForward ? _self.fastForward : fastForward // ignore: cast_nullable_to_non_nullable
as GamepadButton?,saveState: freezed == saveState ? _self.saveState : saveState // ignore: cast_nullable_to_non_nullable
as GamepadButton?,loadState: freezed == loadState ? _self.loadState : loadState // ignore: cast_nullable_to_non_nullable
as GamepadButton?,pause: freezed == pause ? _self.pause : pause // ignore: cast_nullable_to_non_nullable
as GamepadButton?,fullScreen: freezed == fullScreen ? _self.fullScreen : fullScreen // ignore: cast_nullable_to_non_nullable
as GamepadButton?,
  ));
}


}

// dart format on
