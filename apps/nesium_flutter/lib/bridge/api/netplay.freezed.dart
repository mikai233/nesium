// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'netplay.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

/// @nodoc
mixin _$NetplayGameEvent {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(Uint8List data) loadRom,
    required TResult Function() startGame,
    required TResult Function(bool paused) pauseSync,
    required TResult Function(int kind) resetSync,
    required TResult Function(int frame, Uint8List data) syncState,
    required TResult Function(int playerIndex) playerLeft,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(Uint8List data)? loadRom,
    TResult? Function()? startGame,
    TResult? Function(bool paused)? pauseSync,
    TResult? Function(int kind)? resetSync,
    TResult? Function(int frame, Uint8List data)? syncState,
    TResult? Function(int playerIndex)? playerLeft,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(Uint8List data)? loadRom,
    TResult Function()? startGame,
    TResult Function(bool paused)? pauseSync,
    TResult Function(int kind)? resetSync,
    TResult Function(int frame, Uint8List data)? syncState,
    TResult Function(int playerIndex)? playerLeft,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(NetplayGameEvent_LoadRom value) loadRom,
    required TResult Function(NetplayGameEvent_StartGame value) startGame,
    required TResult Function(NetplayGameEvent_PauseSync value) pauseSync,
    required TResult Function(NetplayGameEvent_ResetSync value) resetSync,
    required TResult Function(NetplayGameEvent_SyncState value) syncState,
    required TResult Function(NetplayGameEvent_PlayerLeft value) playerLeft,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult? Function(NetplayGameEvent_StartGame value)? startGame,
    TResult? Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult? Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult? Function(NetplayGameEvent_SyncState value)? syncState,
    TResult? Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult Function(NetplayGameEvent_StartGame value)? startGame,
    TResult Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult Function(NetplayGameEvent_SyncState value)? syncState,
    TResult Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $NetplayGameEventCopyWith<$Res> {
  factory $NetplayGameEventCopyWith(
    NetplayGameEvent value,
    $Res Function(NetplayGameEvent) then,
  ) = _$NetplayGameEventCopyWithImpl<$Res, NetplayGameEvent>;
}

/// @nodoc
class _$NetplayGameEventCopyWithImpl<$Res, $Val extends NetplayGameEvent>
    implements $NetplayGameEventCopyWith<$Res> {
  _$NetplayGameEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$NetplayGameEvent_LoadRomImplCopyWith<$Res> {
  factory _$$NetplayGameEvent_LoadRomImplCopyWith(
    _$NetplayGameEvent_LoadRomImpl value,
    $Res Function(_$NetplayGameEvent_LoadRomImpl) then,
  ) = __$$NetplayGameEvent_LoadRomImplCopyWithImpl<$Res>;
  @useResult
  $Res call({Uint8List data});
}

/// @nodoc
class __$$NetplayGameEvent_LoadRomImplCopyWithImpl<$Res>
    extends _$NetplayGameEventCopyWithImpl<$Res, _$NetplayGameEvent_LoadRomImpl>
    implements _$$NetplayGameEvent_LoadRomImplCopyWith<$Res> {
  __$$NetplayGameEvent_LoadRomImplCopyWithImpl(
    _$NetplayGameEvent_LoadRomImpl _value,
    $Res Function(_$NetplayGameEvent_LoadRomImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? data = null}) {
    return _then(
      _$NetplayGameEvent_LoadRomImpl(
        data: null == data
            ? _value.data
            : data // ignore: cast_nullable_to_non_nullable
                  as Uint8List,
      ),
    );
  }
}

/// @nodoc

class _$NetplayGameEvent_LoadRomImpl extends NetplayGameEvent_LoadRom {
  const _$NetplayGameEvent_LoadRomImpl({required this.data}) : super._();

  @override
  final Uint8List data;

  @override
  String toString() {
    return 'NetplayGameEvent.loadRom(data: $data)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$NetplayGameEvent_LoadRomImpl &&
            const DeepCollectionEquality().equals(other.data, data));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(data));

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$NetplayGameEvent_LoadRomImplCopyWith<_$NetplayGameEvent_LoadRomImpl>
  get copyWith =>
      __$$NetplayGameEvent_LoadRomImplCopyWithImpl<
        _$NetplayGameEvent_LoadRomImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(Uint8List data) loadRom,
    required TResult Function() startGame,
    required TResult Function(bool paused) pauseSync,
    required TResult Function(int kind) resetSync,
    required TResult Function(int frame, Uint8List data) syncState,
    required TResult Function(int playerIndex) playerLeft,
  }) {
    return loadRom(data);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(Uint8List data)? loadRom,
    TResult? Function()? startGame,
    TResult? Function(bool paused)? pauseSync,
    TResult? Function(int kind)? resetSync,
    TResult? Function(int frame, Uint8List data)? syncState,
    TResult? Function(int playerIndex)? playerLeft,
  }) {
    return loadRom?.call(data);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(Uint8List data)? loadRom,
    TResult Function()? startGame,
    TResult Function(bool paused)? pauseSync,
    TResult Function(int kind)? resetSync,
    TResult Function(int frame, Uint8List data)? syncState,
    TResult Function(int playerIndex)? playerLeft,
    required TResult orElse(),
  }) {
    if (loadRom != null) {
      return loadRom(data);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(NetplayGameEvent_LoadRom value) loadRom,
    required TResult Function(NetplayGameEvent_StartGame value) startGame,
    required TResult Function(NetplayGameEvent_PauseSync value) pauseSync,
    required TResult Function(NetplayGameEvent_ResetSync value) resetSync,
    required TResult Function(NetplayGameEvent_SyncState value) syncState,
    required TResult Function(NetplayGameEvent_PlayerLeft value) playerLeft,
  }) {
    return loadRom(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult? Function(NetplayGameEvent_StartGame value)? startGame,
    TResult? Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult? Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult? Function(NetplayGameEvent_SyncState value)? syncState,
    TResult? Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
  }) {
    return loadRom?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult Function(NetplayGameEvent_StartGame value)? startGame,
    TResult Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult Function(NetplayGameEvent_SyncState value)? syncState,
    TResult Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
    required TResult orElse(),
  }) {
    if (loadRom != null) {
      return loadRom(this);
    }
    return orElse();
  }
}

abstract class NetplayGameEvent_LoadRom extends NetplayGameEvent {
  const factory NetplayGameEvent_LoadRom({required final Uint8List data}) =
      _$NetplayGameEvent_LoadRomImpl;
  const NetplayGameEvent_LoadRom._() : super._();

  Uint8List get data;

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$NetplayGameEvent_LoadRomImplCopyWith<_$NetplayGameEvent_LoadRomImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$NetplayGameEvent_StartGameImplCopyWith<$Res> {
  factory _$$NetplayGameEvent_StartGameImplCopyWith(
    _$NetplayGameEvent_StartGameImpl value,
    $Res Function(_$NetplayGameEvent_StartGameImpl) then,
  ) = __$$NetplayGameEvent_StartGameImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$NetplayGameEvent_StartGameImplCopyWithImpl<$Res>
    extends
        _$NetplayGameEventCopyWithImpl<$Res, _$NetplayGameEvent_StartGameImpl>
    implements _$$NetplayGameEvent_StartGameImplCopyWith<$Res> {
  __$$NetplayGameEvent_StartGameImplCopyWithImpl(
    _$NetplayGameEvent_StartGameImpl _value,
    $Res Function(_$NetplayGameEvent_StartGameImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$NetplayGameEvent_StartGameImpl extends NetplayGameEvent_StartGame {
  const _$NetplayGameEvent_StartGameImpl() : super._();

  @override
  String toString() {
    return 'NetplayGameEvent.startGame()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$NetplayGameEvent_StartGameImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(Uint8List data) loadRom,
    required TResult Function() startGame,
    required TResult Function(bool paused) pauseSync,
    required TResult Function(int kind) resetSync,
    required TResult Function(int frame, Uint8List data) syncState,
    required TResult Function(int playerIndex) playerLeft,
  }) {
    return startGame();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(Uint8List data)? loadRom,
    TResult? Function()? startGame,
    TResult? Function(bool paused)? pauseSync,
    TResult? Function(int kind)? resetSync,
    TResult? Function(int frame, Uint8List data)? syncState,
    TResult? Function(int playerIndex)? playerLeft,
  }) {
    return startGame?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(Uint8List data)? loadRom,
    TResult Function()? startGame,
    TResult Function(bool paused)? pauseSync,
    TResult Function(int kind)? resetSync,
    TResult Function(int frame, Uint8List data)? syncState,
    TResult Function(int playerIndex)? playerLeft,
    required TResult orElse(),
  }) {
    if (startGame != null) {
      return startGame();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(NetplayGameEvent_LoadRom value) loadRom,
    required TResult Function(NetplayGameEvent_StartGame value) startGame,
    required TResult Function(NetplayGameEvent_PauseSync value) pauseSync,
    required TResult Function(NetplayGameEvent_ResetSync value) resetSync,
    required TResult Function(NetplayGameEvent_SyncState value) syncState,
    required TResult Function(NetplayGameEvent_PlayerLeft value) playerLeft,
  }) {
    return startGame(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult? Function(NetplayGameEvent_StartGame value)? startGame,
    TResult? Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult? Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult? Function(NetplayGameEvent_SyncState value)? syncState,
    TResult? Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
  }) {
    return startGame?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult Function(NetplayGameEvent_StartGame value)? startGame,
    TResult Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult Function(NetplayGameEvent_SyncState value)? syncState,
    TResult Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
    required TResult orElse(),
  }) {
    if (startGame != null) {
      return startGame(this);
    }
    return orElse();
  }
}

abstract class NetplayGameEvent_StartGame extends NetplayGameEvent {
  const factory NetplayGameEvent_StartGame() = _$NetplayGameEvent_StartGameImpl;
  const NetplayGameEvent_StartGame._() : super._();
}

/// @nodoc
abstract class _$$NetplayGameEvent_PauseSyncImplCopyWith<$Res> {
  factory _$$NetplayGameEvent_PauseSyncImplCopyWith(
    _$NetplayGameEvent_PauseSyncImpl value,
    $Res Function(_$NetplayGameEvent_PauseSyncImpl) then,
  ) = __$$NetplayGameEvent_PauseSyncImplCopyWithImpl<$Res>;
  @useResult
  $Res call({bool paused});
}

/// @nodoc
class __$$NetplayGameEvent_PauseSyncImplCopyWithImpl<$Res>
    extends
        _$NetplayGameEventCopyWithImpl<$Res, _$NetplayGameEvent_PauseSyncImpl>
    implements _$$NetplayGameEvent_PauseSyncImplCopyWith<$Res> {
  __$$NetplayGameEvent_PauseSyncImplCopyWithImpl(
    _$NetplayGameEvent_PauseSyncImpl _value,
    $Res Function(_$NetplayGameEvent_PauseSyncImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? paused = null}) {
    return _then(
      _$NetplayGameEvent_PauseSyncImpl(
        paused: null == paused
            ? _value.paused
            : paused // ignore: cast_nullable_to_non_nullable
                  as bool,
      ),
    );
  }
}

/// @nodoc

class _$NetplayGameEvent_PauseSyncImpl extends NetplayGameEvent_PauseSync {
  const _$NetplayGameEvent_PauseSyncImpl({required this.paused}) : super._();

  @override
  final bool paused;

  @override
  String toString() {
    return 'NetplayGameEvent.pauseSync(paused: $paused)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$NetplayGameEvent_PauseSyncImpl &&
            (identical(other.paused, paused) || other.paused == paused));
  }

  @override
  int get hashCode => Object.hash(runtimeType, paused);

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$NetplayGameEvent_PauseSyncImplCopyWith<_$NetplayGameEvent_PauseSyncImpl>
  get copyWith =>
      __$$NetplayGameEvent_PauseSyncImplCopyWithImpl<
        _$NetplayGameEvent_PauseSyncImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(Uint8List data) loadRom,
    required TResult Function() startGame,
    required TResult Function(bool paused) pauseSync,
    required TResult Function(int kind) resetSync,
    required TResult Function(int frame, Uint8List data) syncState,
    required TResult Function(int playerIndex) playerLeft,
  }) {
    return pauseSync(paused);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(Uint8List data)? loadRom,
    TResult? Function()? startGame,
    TResult? Function(bool paused)? pauseSync,
    TResult? Function(int kind)? resetSync,
    TResult? Function(int frame, Uint8List data)? syncState,
    TResult? Function(int playerIndex)? playerLeft,
  }) {
    return pauseSync?.call(paused);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(Uint8List data)? loadRom,
    TResult Function()? startGame,
    TResult Function(bool paused)? pauseSync,
    TResult Function(int kind)? resetSync,
    TResult Function(int frame, Uint8List data)? syncState,
    TResult Function(int playerIndex)? playerLeft,
    required TResult orElse(),
  }) {
    if (pauseSync != null) {
      return pauseSync(paused);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(NetplayGameEvent_LoadRom value) loadRom,
    required TResult Function(NetplayGameEvent_StartGame value) startGame,
    required TResult Function(NetplayGameEvent_PauseSync value) pauseSync,
    required TResult Function(NetplayGameEvent_ResetSync value) resetSync,
    required TResult Function(NetplayGameEvent_SyncState value) syncState,
    required TResult Function(NetplayGameEvent_PlayerLeft value) playerLeft,
  }) {
    return pauseSync(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult? Function(NetplayGameEvent_StartGame value)? startGame,
    TResult? Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult? Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult? Function(NetplayGameEvent_SyncState value)? syncState,
    TResult? Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
  }) {
    return pauseSync?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult Function(NetplayGameEvent_StartGame value)? startGame,
    TResult Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult Function(NetplayGameEvent_SyncState value)? syncState,
    TResult Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
    required TResult orElse(),
  }) {
    if (pauseSync != null) {
      return pauseSync(this);
    }
    return orElse();
  }
}

abstract class NetplayGameEvent_PauseSync extends NetplayGameEvent {
  const factory NetplayGameEvent_PauseSync({required final bool paused}) =
      _$NetplayGameEvent_PauseSyncImpl;
  const NetplayGameEvent_PauseSync._() : super._();

  bool get paused;

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$NetplayGameEvent_PauseSyncImplCopyWith<_$NetplayGameEvent_PauseSyncImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$NetplayGameEvent_ResetSyncImplCopyWith<$Res> {
  factory _$$NetplayGameEvent_ResetSyncImplCopyWith(
    _$NetplayGameEvent_ResetSyncImpl value,
    $Res Function(_$NetplayGameEvent_ResetSyncImpl) then,
  ) = __$$NetplayGameEvent_ResetSyncImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int kind});
}

/// @nodoc
class __$$NetplayGameEvent_ResetSyncImplCopyWithImpl<$Res>
    extends
        _$NetplayGameEventCopyWithImpl<$Res, _$NetplayGameEvent_ResetSyncImpl>
    implements _$$NetplayGameEvent_ResetSyncImplCopyWith<$Res> {
  __$$NetplayGameEvent_ResetSyncImplCopyWithImpl(
    _$NetplayGameEvent_ResetSyncImpl _value,
    $Res Function(_$NetplayGameEvent_ResetSyncImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? kind = null}) {
    return _then(
      _$NetplayGameEvent_ResetSyncImpl(
        kind: null == kind
            ? _value.kind
            : kind // ignore: cast_nullable_to_non_nullable
                  as int,
      ),
    );
  }
}

/// @nodoc

class _$NetplayGameEvent_ResetSyncImpl extends NetplayGameEvent_ResetSync {
  const _$NetplayGameEvent_ResetSyncImpl({required this.kind}) : super._();

  @override
  final int kind;

  @override
  String toString() {
    return 'NetplayGameEvent.resetSync(kind: $kind)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$NetplayGameEvent_ResetSyncImpl &&
            (identical(other.kind, kind) || other.kind == kind));
  }

  @override
  int get hashCode => Object.hash(runtimeType, kind);

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$NetplayGameEvent_ResetSyncImplCopyWith<_$NetplayGameEvent_ResetSyncImpl>
  get copyWith =>
      __$$NetplayGameEvent_ResetSyncImplCopyWithImpl<
        _$NetplayGameEvent_ResetSyncImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(Uint8List data) loadRom,
    required TResult Function() startGame,
    required TResult Function(bool paused) pauseSync,
    required TResult Function(int kind) resetSync,
    required TResult Function(int frame, Uint8List data) syncState,
    required TResult Function(int playerIndex) playerLeft,
  }) {
    return resetSync(kind);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(Uint8List data)? loadRom,
    TResult? Function()? startGame,
    TResult? Function(bool paused)? pauseSync,
    TResult? Function(int kind)? resetSync,
    TResult? Function(int frame, Uint8List data)? syncState,
    TResult? Function(int playerIndex)? playerLeft,
  }) {
    return resetSync?.call(kind);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(Uint8List data)? loadRom,
    TResult Function()? startGame,
    TResult Function(bool paused)? pauseSync,
    TResult Function(int kind)? resetSync,
    TResult Function(int frame, Uint8List data)? syncState,
    TResult Function(int playerIndex)? playerLeft,
    required TResult orElse(),
  }) {
    if (resetSync != null) {
      return resetSync(kind);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(NetplayGameEvent_LoadRom value) loadRom,
    required TResult Function(NetplayGameEvent_StartGame value) startGame,
    required TResult Function(NetplayGameEvent_PauseSync value) pauseSync,
    required TResult Function(NetplayGameEvent_ResetSync value) resetSync,
    required TResult Function(NetplayGameEvent_SyncState value) syncState,
    required TResult Function(NetplayGameEvent_PlayerLeft value) playerLeft,
  }) {
    return resetSync(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult? Function(NetplayGameEvent_StartGame value)? startGame,
    TResult? Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult? Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult? Function(NetplayGameEvent_SyncState value)? syncState,
    TResult? Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
  }) {
    return resetSync?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult Function(NetplayGameEvent_StartGame value)? startGame,
    TResult Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult Function(NetplayGameEvent_SyncState value)? syncState,
    TResult Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
    required TResult orElse(),
  }) {
    if (resetSync != null) {
      return resetSync(this);
    }
    return orElse();
  }
}

abstract class NetplayGameEvent_ResetSync extends NetplayGameEvent {
  const factory NetplayGameEvent_ResetSync({required final int kind}) =
      _$NetplayGameEvent_ResetSyncImpl;
  const NetplayGameEvent_ResetSync._() : super._();

  int get kind;

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$NetplayGameEvent_ResetSyncImplCopyWith<_$NetplayGameEvent_ResetSyncImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$NetplayGameEvent_SyncStateImplCopyWith<$Res> {
  factory _$$NetplayGameEvent_SyncStateImplCopyWith(
    _$NetplayGameEvent_SyncStateImpl value,
    $Res Function(_$NetplayGameEvent_SyncStateImpl) then,
  ) = __$$NetplayGameEvent_SyncStateImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int frame, Uint8List data});
}

/// @nodoc
class __$$NetplayGameEvent_SyncStateImplCopyWithImpl<$Res>
    extends
        _$NetplayGameEventCopyWithImpl<$Res, _$NetplayGameEvent_SyncStateImpl>
    implements _$$NetplayGameEvent_SyncStateImplCopyWith<$Res> {
  __$$NetplayGameEvent_SyncStateImplCopyWithImpl(
    _$NetplayGameEvent_SyncStateImpl _value,
    $Res Function(_$NetplayGameEvent_SyncStateImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? frame = null, Object? data = null}) {
    return _then(
      _$NetplayGameEvent_SyncStateImpl(
        frame: null == frame
            ? _value.frame
            : frame // ignore: cast_nullable_to_non_nullable
                  as int,
        data: null == data
            ? _value.data
            : data // ignore: cast_nullable_to_non_nullable
                  as Uint8List,
      ),
    );
  }
}

/// @nodoc

class _$NetplayGameEvent_SyncStateImpl extends NetplayGameEvent_SyncState {
  const _$NetplayGameEvent_SyncStateImpl({
    required this.frame,
    required this.data,
  }) : super._();

  @override
  final int frame;
  @override
  final Uint8List data;

  @override
  String toString() {
    return 'NetplayGameEvent.syncState(frame: $frame, data: $data)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$NetplayGameEvent_SyncStateImpl &&
            (identical(other.frame, frame) || other.frame == frame) &&
            const DeepCollectionEquality().equals(other.data, data));
  }

  @override
  int get hashCode => Object.hash(
    runtimeType,
    frame,
    const DeepCollectionEquality().hash(data),
  );

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$NetplayGameEvent_SyncStateImplCopyWith<_$NetplayGameEvent_SyncStateImpl>
  get copyWith =>
      __$$NetplayGameEvent_SyncStateImplCopyWithImpl<
        _$NetplayGameEvent_SyncStateImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(Uint8List data) loadRom,
    required TResult Function() startGame,
    required TResult Function(bool paused) pauseSync,
    required TResult Function(int kind) resetSync,
    required TResult Function(int frame, Uint8List data) syncState,
    required TResult Function(int playerIndex) playerLeft,
  }) {
    return syncState(frame, data);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(Uint8List data)? loadRom,
    TResult? Function()? startGame,
    TResult? Function(bool paused)? pauseSync,
    TResult? Function(int kind)? resetSync,
    TResult? Function(int frame, Uint8List data)? syncState,
    TResult? Function(int playerIndex)? playerLeft,
  }) {
    return syncState?.call(frame, data);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(Uint8List data)? loadRom,
    TResult Function()? startGame,
    TResult Function(bool paused)? pauseSync,
    TResult Function(int kind)? resetSync,
    TResult Function(int frame, Uint8List data)? syncState,
    TResult Function(int playerIndex)? playerLeft,
    required TResult orElse(),
  }) {
    if (syncState != null) {
      return syncState(frame, data);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(NetplayGameEvent_LoadRom value) loadRom,
    required TResult Function(NetplayGameEvent_StartGame value) startGame,
    required TResult Function(NetplayGameEvent_PauseSync value) pauseSync,
    required TResult Function(NetplayGameEvent_ResetSync value) resetSync,
    required TResult Function(NetplayGameEvent_SyncState value) syncState,
    required TResult Function(NetplayGameEvent_PlayerLeft value) playerLeft,
  }) {
    return syncState(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult? Function(NetplayGameEvent_StartGame value)? startGame,
    TResult? Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult? Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult? Function(NetplayGameEvent_SyncState value)? syncState,
    TResult? Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
  }) {
    return syncState?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult Function(NetplayGameEvent_StartGame value)? startGame,
    TResult Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult Function(NetplayGameEvent_SyncState value)? syncState,
    TResult Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
    required TResult orElse(),
  }) {
    if (syncState != null) {
      return syncState(this);
    }
    return orElse();
  }
}

abstract class NetplayGameEvent_SyncState extends NetplayGameEvent {
  const factory NetplayGameEvent_SyncState({
    required final int frame,
    required final Uint8List data,
  }) = _$NetplayGameEvent_SyncStateImpl;
  const NetplayGameEvent_SyncState._() : super._();

  int get frame;
  Uint8List get data;

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$NetplayGameEvent_SyncStateImplCopyWith<_$NetplayGameEvent_SyncStateImpl>
  get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$NetplayGameEvent_PlayerLeftImplCopyWith<$Res> {
  factory _$$NetplayGameEvent_PlayerLeftImplCopyWith(
    _$NetplayGameEvent_PlayerLeftImpl value,
    $Res Function(_$NetplayGameEvent_PlayerLeftImpl) then,
  ) = __$$NetplayGameEvent_PlayerLeftImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int playerIndex});
}

/// @nodoc
class __$$NetplayGameEvent_PlayerLeftImplCopyWithImpl<$Res>
    extends
        _$NetplayGameEventCopyWithImpl<$Res, _$NetplayGameEvent_PlayerLeftImpl>
    implements _$$NetplayGameEvent_PlayerLeftImplCopyWith<$Res> {
  __$$NetplayGameEvent_PlayerLeftImplCopyWithImpl(
    _$NetplayGameEvent_PlayerLeftImpl _value,
    $Res Function(_$NetplayGameEvent_PlayerLeftImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? playerIndex = null}) {
    return _then(
      _$NetplayGameEvent_PlayerLeftImpl(
        playerIndex: null == playerIndex
            ? _value.playerIndex
            : playerIndex // ignore: cast_nullable_to_non_nullable
                  as int,
      ),
    );
  }
}

/// @nodoc

class _$NetplayGameEvent_PlayerLeftImpl extends NetplayGameEvent_PlayerLeft {
  const _$NetplayGameEvent_PlayerLeftImpl({required this.playerIndex})
    : super._();

  @override
  final int playerIndex;

  @override
  String toString() {
    return 'NetplayGameEvent.playerLeft(playerIndex: $playerIndex)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$NetplayGameEvent_PlayerLeftImpl &&
            (identical(other.playerIndex, playerIndex) ||
                other.playerIndex == playerIndex));
  }

  @override
  int get hashCode => Object.hash(runtimeType, playerIndex);

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$NetplayGameEvent_PlayerLeftImplCopyWith<_$NetplayGameEvent_PlayerLeftImpl>
  get copyWith =>
      __$$NetplayGameEvent_PlayerLeftImplCopyWithImpl<
        _$NetplayGameEvent_PlayerLeftImpl
      >(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(Uint8List data) loadRom,
    required TResult Function() startGame,
    required TResult Function(bool paused) pauseSync,
    required TResult Function(int kind) resetSync,
    required TResult Function(int frame, Uint8List data) syncState,
    required TResult Function(int playerIndex) playerLeft,
  }) {
    return playerLeft(playerIndex);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(Uint8List data)? loadRom,
    TResult? Function()? startGame,
    TResult? Function(bool paused)? pauseSync,
    TResult? Function(int kind)? resetSync,
    TResult? Function(int frame, Uint8List data)? syncState,
    TResult? Function(int playerIndex)? playerLeft,
  }) {
    return playerLeft?.call(playerIndex);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(Uint8List data)? loadRom,
    TResult Function()? startGame,
    TResult Function(bool paused)? pauseSync,
    TResult Function(int kind)? resetSync,
    TResult Function(int frame, Uint8List data)? syncState,
    TResult Function(int playerIndex)? playerLeft,
    required TResult orElse(),
  }) {
    if (playerLeft != null) {
      return playerLeft(playerIndex);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(NetplayGameEvent_LoadRom value) loadRom,
    required TResult Function(NetplayGameEvent_StartGame value) startGame,
    required TResult Function(NetplayGameEvent_PauseSync value) pauseSync,
    required TResult Function(NetplayGameEvent_ResetSync value) resetSync,
    required TResult Function(NetplayGameEvent_SyncState value) syncState,
    required TResult Function(NetplayGameEvent_PlayerLeft value) playerLeft,
  }) {
    return playerLeft(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult? Function(NetplayGameEvent_StartGame value)? startGame,
    TResult? Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult? Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult? Function(NetplayGameEvent_SyncState value)? syncState,
    TResult? Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
  }) {
    return playerLeft?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(NetplayGameEvent_LoadRom value)? loadRom,
    TResult Function(NetplayGameEvent_StartGame value)? startGame,
    TResult Function(NetplayGameEvent_PauseSync value)? pauseSync,
    TResult Function(NetplayGameEvent_ResetSync value)? resetSync,
    TResult Function(NetplayGameEvent_SyncState value)? syncState,
    TResult Function(NetplayGameEvent_PlayerLeft value)? playerLeft,
    required TResult orElse(),
  }) {
    if (playerLeft != null) {
      return playerLeft(this);
    }
    return orElse();
  }
}

abstract class NetplayGameEvent_PlayerLeft extends NetplayGameEvent {
  const factory NetplayGameEvent_PlayerLeft({required final int playerIndex}) =
      _$NetplayGameEvent_PlayerLeftImpl;
  const NetplayGameEvent_PlayerLeft._() : super._();

  int get playerIndex;

  /// Create a copy of NetplayGameEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$NetplayGameEvent_PlayerLeftImplCopyWith<_$NetplayGameEvent_PlayerLeftImpl>
  get copyWith => throw _privateConstructorUsedError;
}
