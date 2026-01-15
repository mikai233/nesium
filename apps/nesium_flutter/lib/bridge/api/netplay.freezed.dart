// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'netplay.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NetplayGameEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetplayGameEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NetplayGameEvent()';
}


}

/// @nodoc
class $NetplayGameEventCopyWith<$Res>  {
$NetplayGameEventCopyWith(NetplayGameEvent _, $Res Function(NetplayGameEvent) __);
}


/// Adds pattern-matching-related methods to [NetplayGameEvent].
extension NetplayGameEventPatterns on NetplayGameEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NetplayGameEvent_LoadRom value)?  loadRom,TResult Function( NetplayGameEvent_StartGame value)?  startGame,TResult Function( NetplayGameEvent_PauseSync value)?  pauseSync,TResult Function( NetplayGameEvent_ResetSync value)?  resetSync,TResult Function( NetplayGameEvent_SyncState value)?  syncState,TResult Function( NetplayGameEvent_PlayerLeft value)?  playerLeft,TResult Function( NetplayGameEvent_Error value)?  error,TResult Function( NetplayGameEvent_FallbackToRelay value)?  fallbackToRelay,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NetplayGameEvent_LoadRom() when loadRom != null:
return loadRom(_that);case NetplayGameEvent_StartGame() when startGame != null:
return startGame(_that);case NetplayGameEvent_PauseSync() when pauseSync != null:
return pauseSync(_that);case NetplayGameEvent_ResetSync() when resetSync != null:
return resetSync(_that);case NetplayGameEvent_SyncState() when syncState != null:
return syncState(_that);case NetplayGameEvent_PlayerLeft() when playerLeft != null:
return playerLeft(_that);case NetplayGameEvent_Error() when error != null:
return error(_that);case NetplayGameEvent_FallbackToRelay() when fallbackToRelay != null:
return fallbackToRelay(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NetplayGameEvent_LoadRom value)  loadRom,required TResult Function( NetplayGameEvent_StartGame value)  startGame,required TResult Function( NetplayGameEvent_PauseSync value)  pauseSync,required TResult Function( NetplayGameEvent_ResetSync value)  resetSync,required TResult Function( NetplayGameEvent_SyncState value)  syncState,required TResult Function( NetplayGameEvent_PlayerLeft value)  playerLeft,required TResult Function( NetplayGameEvent_Error value)  error,required TResult Function( NetplayGameEvent_FallbackToRelay value)  fallbackToRelay,}){
final _that = this;
switch (_that) {
case NetplayGameEvent_LoadRom():
return loadRom(_that);case NetplayGameEvent_StartGame():
return startGame(_that);case NetplayGameEvent_PauseSync():
return pauseSync(_that);case NetplayGameEvent_ResetSync():
return resetSync(_that);case NetplayGameEvent_SyncState():
return syncState(_that);case NetplayGameEvent_PlayerLeft():
return playerLeft(_that);case NetplayGameEvent_Error():
return error(_that);case NetplayGameEvent_FallbackToRelay():
return fallbackToRelay(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NetplayGameEvent_LoadRom value)?  loadRom,TResult? Function( NetplayGameEvent_StartGame value)?  startGame,TResult? Function( NetplayGameEvent_PauseSync value)?  pauseSync,TResult? Function( NetplayGameEvent_ResetSync value)?  resetSync,TResult? Function( NetplayGameEvent_SyncState value)?  syncState,TResult? Function( NetplayGameEvent_PlayerLeft value)?  playerLeft,TResult? Function( NetplayGameEvent_Error value)?  error,TResult? Function( NetplayGameEvent_FallbackToRelay value)?  fallbackToRelay,}){
final _that = this;
switch (_that) {
case NetplayGameEvent_LoadRom() when loadRom != null:
return loadRom(_that);case NetplayGameEvent_StartGame() when startGame != null:
return startGame(_that);case NetplayGameEvent_PauseSync() when pauseSync != null:
return pauseSync(_that);case NetplayGameEvent_ResetSync() when resetSync != null:
return resetSync(_that);case NetplayGameEvent_SyncState() when syncState != null:
return syncState(_that);case NetplayGameEvent_PlayerLeft() when playerLeft != null:
return playerLeft(_that);case NetplayGameEvent_Error() when error != null:
return error(_that);case NetplayGameEvent_FallbackToRelay() when fallbackToRelay != null:
return fallbackToRelay(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( Uint8List data)?  loadRom,TResult Function()?  startGame,TResult Function( bool paused)?  pauseSync,TResult Function( int kind)?  resetSync,TResult Function( int frame,  Uint8List data)?  syncState,TResult Function( int playerIndex)?  playerLeft,TResult Function( int errorCode)?  error,TResult Function( String relayAddr,  int relayRoomCode,  String reason)?  fallbackToRelay,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NetplayGameEvent_LoadRom() when loadRom != null:
return loadRom(_that.data);case NetplayGameEvent_StartGame() when startGame != null:
return startGame();case NetplayGameEvent_PauseSync() when pauseSync != null:
return pauseSync(_that.paused);case NetplayGameEvent_ResetSync() when resetSync != null:
return resetSync(_that.kind);case NetplayGameEvent_SyncState() when syncState != null:
return syncState(_that.frame,_that.data);case NetplayGameEvent_PlayerLeft() when playerLeft != null:
return playerLeft(_that.playerIndex);case NetplayGameEvent_Error() when error != null:
return error(_that.errorCode);case NetplayGameEvent_FallbackToRelay() when fallbackToRelay != null:
return fallbackToRelay(_that.relayAddr,_that.relayRoomCode,_that.reason);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( Uint8List data)  loadRom,required TResult Function()  startGame,required TResult Function( bool paused)  pauseSync,required TResult Function( int kind)  resetSync,required TResult Function( int frame,  Uint8List data)  syncState,required TResult Function( int playerIndex)  playerLeft,required TResult Function( int errorCode)  error,required TResult Function( String relayAddr,  int relayRoomCode,  String reason)  fallbackToRelay,}) {final _that = this;
switch (_that) {
case NetplayGameEvent_LoadRom():
return loadRom(_that.data);case NetplayGameEvent_StartGame():
return startGame();case NetplayGameEvent_PauseSync():
return pauseSync(_that.paused);case NetplayGameEvent_ResetSync():
return resetSync(_that.kind);case NetplayGameEvent_SyncState():
return syncState(_that.frame,_that.data);case NetplayGameEvent_PlayerLeft():
return playerLeft(_that.playerIndex);case NetplayGameEvent_Error():
return error(_that.errorCode);case NetplayGameEvent_FallbackToRelay():
return fallbackToRelay(_that.relayAddr,_that.relayRoomCode,_that.reason);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( Uint8List data)?  loadRom,TResult? Function()?  startGame,TResult? Function( bool paused)?  pauseSync,TResult? Function( int kind)?  resetSync,TResult? Function( int frame,  Uint8List data)?  syncState,TResult? Function( int playerIndex)?  playerLeft,TResult? Function( int errorCode)?  error,TResult? Function( String relayAddr,  int relayRoomCode,  String reason)?  fallbackToRelay,}) {final _that = this;
switch (_that) {
case NetplayGameEvent_LoadRom() when loadRom != null:
return loadRom(_that.data);case NetplayGameEvent_StartGame() when startGame != null:
return startGame();case NetplayGameEvent_PauseSync() when pauseSync != null:
return pauseSync(_that.paused);case NetplayGameEvent_ResetSync() when resetSync != null:
return resetSync(_that.kind);case NetplayGameEvent_SyncState() when syncState != null:
return syncState(_that.frame,_that.data);case NetplayGameEvent_PlayerLeft() when playerLeft != null:
return playerLeft(_that.playerIndex);case NetplayGameEvent_Error() when error != null:
return error(_that.errorCode);case NetplayGameEvent_FallbackToRelay() when fallbackToRelay != null:
return fallbackToRelay(_that.relayAddr,_that.relayRoomCode,_that.reason);case _:
  return null;

}
}

}

/// @nodoc


class NetplayGameEvent_LoadRom extends NetplayGameEvent {
  const NetplayGameEvent_LoadRom({required this.data}): super._();
  

 final  Uint8List data;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetplayGameEvent_LoadRomCopyWith<NetplayGameEvent_LoadRom> get copyWith => _$NetplayGameEvent_LoadRomCopyWithImpl<NetplayGameEvent_LoadRom>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetplayGameEvent_LoadRom&&const DeepCollectionEquality().equals(other.data, data));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(data));

@override
String toString() {
  return 'NetplayGameEvent.loadRom(data: $data)';
}


}

/// @nodoc
abstract mixin class $NetplayGameEvent_LoadRomCopyWith<$Res> implements $NetplayGameEventCopyWith<$Res> {
  factory $NetplayGameEvent_LoadRomCopyWith(NetplayGameEvent_LoadRom value, $Res Function(NetplayGameEvent_LoadRom) _then) = _$NetplayGameEvent_LoadRomCopyWithImpl;
@useResult
$Res call({
 Uint8List data
});




}
/// @nodoc
class _$NetplayGameEvent_LoadRomCopyWithImpl<$Res>
    implements $NetplayGameEvent_LoadRomCopyWith<$Res> {
  _$NetplayGameEvent_LoadRomCopyWithImpl(this._self, this._then);

  final NetplayGameEvent_LoadRom _self;
  final $Res Function(NetplayGameEvent_LoadRom) _then;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? data = null,}) {
  return _then(NetplayGameEvent_LoadRom(
data: null == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as Uint8List,
  ));
}


}

/// @nodoc


class NetplayGameEvent_StartGame extends NetplayGameEvent {
  const NetplayGameEvent_StartGame(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetplayGameEvent_StartGame);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NetplayGameEvent.startGame()';
}


}




/// @nodoc


class NetplayGameEvent_PauseSync extends NetplayGameEvent {
  const NetplayGameEvent_PauseSync({required this.paused}): super._();
  

 final  bool paused;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetplayGameEvent_PauseSyncCopyWith<NetplayGameEvent_PauseSync> get copyWith => _$NetplayGameEvent_PauseSyncCopyWithImpl<NetplayGameEvent_PauseSync>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetplayGameEvent_PauseSync&&(identical(other.paused, paused) || other.paused == paused));
}


@override
int get hashCode => Object.hash(runtimeType,paused);

@override
String toString() {
  return 'NetplayGameEvent.pauseSync(paused: $paused)';
}


}

/// @nodoc
abstract mixin class $NetplayGameEvent_PauseSyncCopyWith<$Res> implements $NetplayGameEventCopyWith<$Res> {
  factory $NetplayGameEvent_PauseSyncCopyWith(NetplayGameEvent_PauseSync value, $Res Function(NetplayGameEvent_PauseSync) _then) = _$NetplayGameEvent_PauseSyncCopyWithImpl;
@useResult
$Res call({
 bool paused
});




}
/// @nodoc
class _$NetplayGameEvent_PauseSyncCopyWithImpl<$Res>
    implements $NetplayGameEvent_PauseSyncCopyWith<$Res> {
  _$NetplayGameEvent_PauseSyncCopyWithImpl(this._self, this._then);

  final NetplayGameEvent_PauseSync _self;
  final $Res Function(NetplayGameEvent_PauseSync) _then;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? paused = null,}) {
  return _then(NetplayGameEvent_PauseSync(
paused: null == paused ? _self.paused : paused // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class NetplayGameEvent_ResetSync extends NetplayGameEvent {
  const NetplayGameEvent_ResetSync({required this.kind}): super._();
  

 final  int kind;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetplayGameEvent_ResetSyncCopyWith<NetplayGameEvent_ResetSync> get copyWith => _$NetplayGameEvent_ResetSyncCopyWithImpl<NetplayGameEvent_ResetSync>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetplayGameEvent_ResetSync&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,kind);

@override
String toString() {
  return 'NetplayGameEvent.resetSync(kind: $kind)';
}


}

/// @nodoc
abstract mixin class $NetplayGameEvent_ResetSyncCopyWith<$Res> implements $NetplayGameEventCopyWith<$Res> {
  factory $NetplayGameEvent_ResetSyncCopyWith(NetplayGameEvent_ResetSync value, $Res Function(NetplayGameEvent_ResetSync) _then) = _$NetplayGameEvent_ResetSyncCopyWithImpl;
@useResult
$Res call({
 int kind
});




}
/// @nodoc
class _$NetplayGameEvent_ResetSyncCopyWithImpl<$Res>
    implements $NetplayGameEvent_ResetSyncCopyWith<$Res> {
  _$NetplayGameEvent_ResetSyncCopyWithImpl(this._self, this._then);

  final NetplayGameEvent_ResetSync _self;
  final $Res Function(NetplayGameEvent_ResetSync) _then;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? kind = null,}) {
  return _then(NetplayGameEvent_ResetSync(
kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class NetplayGameEvent_SyncState extends NetplayGameEvent {
  const NetplayGameEvent_SyncState({required this.frame, required this.data}): super._();
  

 final  int frame;
 final  Uint8List data;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetplayGameEvent_SyncStateCopyWith<NetplayGameEvent_SyncState> get copyWith => _$NetplayGameEvent_SyncStateCopyWithImpl<NetplayGameEvent_SyncState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetplayGameEvent_SyncState&&(identical(other.frame, frame) || other.frame == frame)&&const DeepCollectionEquality().equals(other.data, data));
}


@override
int get hashCode => Object.hash(runtimeType,frame,const DeepCollectionEquality().hash(data));

@override
String toString() {
  return 'NetplayGameEvent.syncState(frame: $frame, data: $data)';
}


}

/// @nodoc
abstract mixin class $NetplayGameEvent_SyncStateCopyWith<$Res> implements $NetplayGameEventCopyWith<$Res> {
  factory $NetplayGameEvent_SyncStateCopyWith(NetplayGameEvent_SyncState value, $Res Function(NetplayGameEvent_SyncState) _then) = _$NetplayGameEvent_SyncStateCopyWithImpl;
@useResult
$Res call({
 int frame, Uint8List data
});




}
/// @nodoc
class _$NetplayGameEvent_SyncStateCopyWithImpl<$Res>
    implements $NetplayGameEvent_SyncStateCopyWith<$Res> {
  _$NetplayGameEvent_SyncStateCopyWithImpl(this._self, this._then);

  final NetplayGameEvent_SyncState _self;
  final $Res Function(NetplayGameEvent_SyncState) _then;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? frame = null,Object? data = null,}) {
  return _then(NetplayGameEvent_SyncState(
frame: null == frame ? _self.frame : frame // ignore: cast_nullable_to_non_nullable
as int,data: null == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as Uint8List,
  ));
}


}

/// @nodoc


class NetplayGameEvent_PlayerLeft extends NetplayGameEvent {
  const NetplayGameEvent_PlayerLeft({required this.playerIndex}): super._();
  

 final  int playerIndex;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetplayGameEvent_PlayerLeftCopyWith<NetplayGameEvent_PlayerLeft> get copyWith => _$NetplayGameEvent_PlayerLeftCopyWithImpl<NetplayGameEvent_PlayerLeft>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetplayGameEvent_PlayerLeft&&(identical(other.playerIndex, playerIndex) || other.playerIndex == playerIndex));
}


@override
int get hashCode => Object.hash(runtimeType,playerIndex);

@override
String toString() {
  return 'NetplayGameEvent.playerLeft(playerIndex: $playerIndex)';
}


}

/// @nodoc
abstract mixin class $NetplayGameEvent_PlayerLeftCopyWith<$Res> implements $NetplayGameEventCopyWith<$Res> {
  factory $NetplayGameEvent_PlayerLeftCopyWith(NetplayGameEvent_PlayerLeft value, $Res Function(NetplayGameEvent_PlayerLeft) _then) = _$NetplayGameEvent_PlayerLeftCopyWithImpl;
@useResult
$Res call({
 int playerIndex
});




}
/// @nodoc
class _$NetplayGameEvent_PlayerLeftCopyWithImpl<$Res>
    implements $NetplayGameEvent_PlayerLeftCopyWith<$Res> {
  _$NetplayGameEvent_PlayerLeftCopyWithImpl(this._self, this._then);

  final NetplayGameEvent_PlayerLeft _self;
  final $Res Function(NetplayGameEvent_PlayerLeft) _then;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? playerIndex = null,}) {
  return _then(NetplayGameEvent_PlayerLeft(
playerIndex: null == playerIndex ? _self.playerIndex : playerIndex // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class NetplayGameEvent_Error extends NetplayGameEvent {
  const NetplayGameEvent_Error({required this.errorCode}): super._();
  

 final  int errorCode;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetplayGameEvent_ErrorCopyWith<NetplayGameEvent_Error> get copyWith => _$NetplayGameEvent_ErrorCopyWithImpl<NetplayGameEvent_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetplayGameEvent_Error&&(identical(other.errorCode, errorCode) || other.errorCode == errorCode));
}


@override
int get hashCode => Object.hash(runtimeType,errorCode);

@override
String toString() {
  return 'NetplayGameEvent.error(errorCode: $errorCode)';
}


}

/// @nodoc
abstract mixin class $NetplayGameEvent_ErrorCopyWith<$Res> implements $NetplayGameEventCopyWith<$Res> {
  factory $NetplayGameEvent_ErrorCopyWith(NetplayGameEvent_Error value, $Res Function(NetplayGameEvent_Error) _then) = _$NetplayGameEvent_ErrorCopyWithImpl;
@useResult
$Res call({
 int errorCode
});




}
/// @nodoc
class _$NetplayGameEvent_ErrorCopyWithImpl<$Res>
    implements $NetplayGameEvent_ErrorCopyWith<$Res> {
  _$NetplayGameEvent_ErrorCopyWithImpl(this._self, this._then);

  final NetplayGameEvent_Error _self;
  final $Res Function(NetplayGameEvent_Error) _then;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? errorCode = null,}) {
  return _then(NetplayGameEvent_Error(
errorCode: null == errorCode ? _self.errorCode : errorCode // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class NetplayGameEvent_FallbackToRelay extends NetplayGameEvent {
  const NetplayGameEvent_FallbackToRelay({required this.relayAddr, required this.relayRoomCode, required this.reason}): super._();
  

 final  String relayAddr;
 final  int relayRoomCode;
 final  String reason;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetplayGameEvent_FallbackToRelayCopyWith<NetplayGameEvent_FallbackToRelay> get copyWith => _$NetplayGameEvent_FallbackToRelayCopyWithImpl<NetplayGameEvent_FallbackToRelay>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetplayGameEvent_FallbackToRelay&&(identical(other.relayAddr, relayAddr) || other.relayAddr == relayAddr)&&(identical(other.relayRoomCode, relayRoomCode) || other.relayRoomCode == relayRoomCode)&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,relayAddr,relayRoomCode,reason);

@override
String toString() {
  return 'NetplayGameEvent.fallbackToRelay(relayAddr: $relayAddr, relayRoomCode: $relayRoomCode, reason: $reason)';
}


}

/// @nodoc
abstract mixin class $NetplayGameEvent_FallbackToRelayCopyWith<$Res> implements $NetplayGameEventCopyWith<$Res> {
  factory $NetplayGameEvent_FallbackToRelayCopyWith(NetplayGameEvent_FallbackToRelay value, $Res Function(NetplayGameEvent_FallbackToRelay) _then) = _$NetplayGameEvent_FallbackToRelayCopyWithImpl;
@useResult
$Res call({
 String relayAddr, int relayRoomCode, String reason
});




}
/// @nodoc
class _$NetplayGameEvent_FallbackToRelayCopyWithImpl<$Res>
    implements $NetplayGameEvent_FallbackToRelayCopyWith<$Res> {
  _$NetplayGameEvent_FallbackToRelayCopyWithImpl(this._self, this._then);

  final NetplayGameEvent_FallbackToRelay _self;
  final $Res Function(NetplayGameEvent_FallbackToRelay) _then;

/// Create a copy of NetplayGameEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? relayAddr = null,Object? relayRoomCode = null,Object? reason = null,}) {
  return _then(NetplayGameEvent_FallbackToRelay(
relayAddr: null == relayAddr ? _self.relayAddr : relayAddr // ignore: cast_nullable_to_non_nullable
as String,relayRoomCode: null == relayRoomCode ? _self.relayRoomCode : relayRoomCode // ignore: cast_nullable_to_non_nullable
as int,reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
