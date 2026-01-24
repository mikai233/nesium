// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'server_settings.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$ServerSettings {

 int get port; String get playerName; String get p2pServerAddr; bool get p2pEnabled; int? get p2pHostRoomCode; NetplayTransportOption get transport; String get sni; String get fingerprint; String get directAddr;
/// Create a copy of ServerSettings
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ServerSettingsCopyWith<ServerSettings> get copyWith => _$ServerSettingsCopyWithImpl<ServerSettings>(this as ServerSettings, _$identity);

  /// Serializes this ServerSettings to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ServerSettings&&(identical(other.port, port) || other.port == port)&&(identical(other.playerName, playerName) || other.playerName == playerName)&&(identical(other.p2pServerAddr, p2pServerAddr) || other.p2pServerAddr == p2pServerAddr)&&(identical(other.p2pEnabled, p2pEnabled) || other.p2pEnabled == p2pEnabled)&&(identical(other.p2pHostRoomCode, p2pHostRoomCode) || other.p2pHostRoomCode == p2pHostRoomCode)&&(identical(other.transport, transport) || other.transport == transport)&&(identical(other.sni, sni) || other.sni == sni)&&(identical(other.fingerprint, fingerprint) || other.fingerprint == fingerprint)&&(identical(other.directAddr, directAddr) || other.directAddr == directAddr));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,port,playerName,p2pServerAddr,p2pEnabled,p2pHostRoomCode,transport,sni,fingerprint,directAddr);

@override
String toString() {
  return 'ServerSettings(port: $port, playerName: $playerName, p2pServerAddr: $p2pServerAddr, p2pEnabled: $p2pEnabled, p2pHostRoomCode: $p2pHostRoomCode, transport: $transport, sni: $sni, fingerprint: $fingerprint, directAddr: $directAddr)';
}


}

/// @nodoc
abstract mixin class $ServerSettingsCopyWith<$Res>  {
  factory $ServerSettingsCopyWith(ServerSettings value, $Res Function(ServerSettings) _then) = _$ServerSettingsCopyWithImpl;
@useResult
$Res call({
 int port, String playerName, String p2pServerAddr, bool p2pEnabled, int? p2pHostRoomCode, NetplayTransportOption transport, String sni, String fingerprint, String directAddr
});




}
/// @nodoc
class _$ServerSettingsCopyWithImpl<$Res>
    implements $ServerSettingsCopyWith<$Res> {
  _$ServerSettingsCopyWithImpl(this._self, this._then);

  final ServerSettings _self;
  final $Res Function(ServerSettings) _then;

/// Create a copy of ServerSettings
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? port = null,Object? playerName = null,Object? p2pServerAddr = null,Object? p2pEnabled = null,Object? p2pHostRoomCode = freezed,Object? transport = null,Object? sni = null,Object? fingerprint = null,Object? directAddr = null,}) {
  return _then(_self.copyWith(
port: null == port ? _self.port : port // ignore: cast_nullable_to_non_nullable
as int,playerName: null == playerName ? _self.playerName : playerName // ignore: cast_nullable_to_non_nullable
as String,p2pServerAddr: null == p2pServerAddr ? _self.p2pServerAddr : p2pServerAddr // ignore: cast_nullable_to_non_nullable
as String,p2pEnabled: null == p2pEnabled ? _self.p2pEnabled : p2pEnabled // ignore: cast_nullable_to_non_nullable
as bool,p2pHostRoomCode: freezed == p2pHostRoomCode ? _self.p2pHostRoomCode : p2pHostRoomCode // ignore: cast_nullable_to_non_nullable
as int?,transport: null == transport ? _self.transport : transport // ignore: cast_nullable_to_non_nullable
as NetplayTransportOption,sni: null == sni ? _self.sni : sni // ignore: cast_nullable_to_non_nullable
as String,fingerprint: null == fingerprint ? _self.fingerprint : fingerprint // ignore: cast_nullable_to_non_nullable
as String,directAddr: null == directAddr ? _self.directAddr : directAddr // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [ServerSettings].
extension ServerSettingsPatterns on ServerSettings {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ServerSettings value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ServerSettings() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ServerSettings value)  $default,){
final _that = this;
switch (_that) {
case _ServerSettings():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ServerSettings value)?  $default,){
final _that = this;
switch (_that) {
case _ServerSettings() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int port,  String playerName,  String p2pServerAddr,  bool p2pEnabled,  int? p2pHostRoomCode,  NetplayTransportOption transport,  String sni,  String fingerprint,  String directAddr)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ServerSettings() when $default != null:
return $default(_that.port,_that.playerName,_that.p2pServerAddr,_that.p2pEnabled,_that.p2pHostRoomCode,_that.transport,_that.sni,_that.fingerprint,_that.directAddr);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int port,  String playerName,  String p2pServerAddr,  bool p2pEnabled,  int? p2pHostRoomCode,  NetplayTransportOption transport,  String sni,  String fingerprint,  String directAddr)  $default,) {final _that = this;
switch (_that) {
case _ServerSettings():
return $default(_that.port,_that.playerName,_that.p2pServerAddr,_that.p2pEnabled,_that.p2pHostRoomCode,_that.transport,_that.sni,_that.fingerprint,_that.directAddr);case _:
  throw StateError('Unexpected subclass');

}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int port,  String playerName,  String p2pServerAddr,  bool p2pEnabled,  int? p2pHostRoomCode,  NetplayTransportOption transport,  String sni,  String fingerprint,  String directAddr)?  $default,) {final _that = this;
switch (_that) {
case _ServerSettings() when $default != null:
return $default(_that.port,_that.playerName,_that.p2pServerAddr,_that.p2pEnabled,_that.p2pHostRoomCode,_that.transport,_that.sni,_that.fingerprint,_that.directAddr);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _ServerSettings implements ServerSettings {
  const _ServerSettings({this.port = 5233, this.playerName = 'Player', this.p2pServerAddr = 'nesium.mikai.link:5233', this.p2pEnabled = false, this.p2pHostRoomCode, this.transport = NetplayTransportOption.auto, this.sni = 'localhost', this.fingerprint = '', this.directAddr = 'localhost'});
  factory _ServerSettings.fromJson(Map<String, dynamic> json) => _$ServerSettingsFromJson(json);

@override@JsonKey() final  int port;
@override@JsonKey() final  String playerName;
@override@JsonKey() final  String p2pServerAddr;
@override@JsonKey() final  bool p2pEnabled;
@override final  int? p2pHostRoomCode;
@override@JsonKey() final  NetplayTransportOption transport;
@override@JsonKey() final  String sni;
@override@JsonKey() final  String fingerprint;
@override@JsonKey() final  String directAddr;

/// Create a copy of ServerSettings
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ServerSettingsCopyWith<_ServerSettings> get copyWith => __$ServerSettingsCopyWithImpl<_ServerSettings>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ServerSettingsToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ServerSettings&&(identical(other.port, port) || other.port == port)&&(identical(other.playerName, playerName) || other.playerName == playerName)&&(identical(other.p2pServerAddr, p2pServerAddr) || other.p2pServerAddr == p2pServerAddr)&&(identical(other.p2pEnabled, p2pEnabled) || other.p2pEnabled == p2pEnabled)&&(identical(other.p2pHostRoomCode, p2pHostRoomCode) || other.p2pHostRoomCode == p2pHostRoomCode)&&(identical(other.transport, transport) || other.transport == transport)&&(identical(other.sni, sni) || other.sni == sni)&&(identical(other.fingerprint, fingerprint) || other.fingerprint == fingerprint)&&(identical(other.directAddr, directAddr) || other.directAddr == directAddr));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,port,playerName,p2pServerAddr,p2pEnabled,p2pHostRoomCode,transport,sni,fingerprint,directAddr);

@override
String toString() {
  return 'ServerSettings(port: $port, playerName: $playerName, p2pServerAddr: $p2pServerAddr, p2pEnabled: $p2pEnabled, p2pHostRoomCode: $p2pHostRoomCode, transport: $transport, sni: $sni, fingerprint: $fingerprint, directAddr: $directAddr)';
}


}

/// @nodoc
abstract mixin class _$ServerSettingsCopyWith<$Res> implements $ServerSettingsCopyWith<$Res> {
  factory _$ServerSettingsCopyWith(_ServerSettings value, $Res Function(_ServerSettings) _then) = __$ServerSettingsCopyWithImpl;
@override @useResult
$Res call({
 int port, String playerName, String p2pServerAddr, bool p2pEnabled, int? p2pHostRoomCode, NetplayTransportOption transport, String sni, String fingerprint, String directAddr
});




}
/// @nodoc
class __$ServerSettingsCopyWithImpl<$Res>
    implements _$ServerSettingsCopyWith<$Res> {
  __$ServerSettingsCopyWithImpl(this._self, this._then);

  final _ServerSettings _self;
  final $Res Function(_ServerSettings) _then;

/// Create a copy of ServerSettings
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? port = null,Object? playerName = null,Object? p2pServerAddr = null,Object? p2pEnabled = null,Object? p2pHostRoomCode = freezed,Object? transport = null,Object? sni = null,Object? fingerprint = null,Object? directAddr = null,}) {
  return _then(_ServerSettings(
port: null == port ? _self.port : port // ignore: cast_nullable_to_non_nullable
as int,playerName: null == playerName ? _self.playerName : playerName // ignore: cast_nullable_to_non_nullable
as String,p2pServerAddr: null == p2pServerAddr ? _self.p2pServerAddr : p2pServerAddr // ignore: cast_nullable_to_non_nullable
as String,p2pEnabled: null == p2pEnabled ? _self.p2pEnabled : p2pEnabled // ignore: cast_nullable_to_non_nullable
as bool,p2pHostRoomCode: freezed == p2pHostRoomCode ? _self.p2pHostRoomCode : p2pHostRoomCode // ignore: cast_nullable_to_non_nullable
as int?,transport: null == transport ? _self.transport : transport // ignore: cast_nullable_to_non_nullable
as NetplayTransportOption,sni: null == sni ? _self.sni : sni // ignore: cast_nullable_to_non_nullable
as String,fingerprint: null == fingerprint ? _self.fingerprint : fingerprint // ignore: cast_nullable_to_non_nullable
as String,directAddr: null == directAddr ? _self.directAddr : directAddr // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
