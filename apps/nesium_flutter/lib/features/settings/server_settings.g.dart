// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'server_settings.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_ServerSettings _$ServerSettingsFromJson(
  Map<String, dynamic> json,
) => _ServerSettings(
  port: (json['port'] as num?)?.toInt() ?? 5233,
  playerName: json['playerName'] as String? ?? 'Player',
  p2pServerAddr: json['p2pServerAddr'] as String? ?? 'nesium.mikai.link:5233',
  p2pEnabled: json['p2pEnabled'] as bool? ?? false,
  p2pHostRoomCode: (json['p2pHostRoomCode'] as num?)?.toInt(),
  transport:
      $enumDecodeNullable(_$NetplayTransportOptionEnumMap, json['transport']) ??
      NetplayTransportOption.auto,
  sni: json['sni'] as String? ?? 'localhost',
  fingerprint: json['fingerprint'] as String? ?? '',
  directAddr: json['directAddr'] as String? ?? 'localhost',
);

Map<String, dynamic> _$ServerSettingsToJson(_ServerSettings instance) =>
    <String, dynamic>{
      'port': instance.port,
      'playerName': instance.playerName,
      'p2pServerAddr': instance.p2pServerAddr,
      'p2pEnabled': instance.p2pEnabled,
      'p2pHostRoomCode': instance.p2pHostRoomCode,
      'transport': _$NetplayTransportOptionEnumMap[instance.transport]!,
      'sni': instance.sni,
      'fingerprint': instance.fingerprint,
      'directAddr': instance.directAddr,
    };

const _$NetplayTransportOptionEnumMap = {
  NetplayTransportOption.auto: 'auto',
  NetplayTransportOption.tcp: 'tcp',
  NetplayTransportOption.quic: 'quic',
};
