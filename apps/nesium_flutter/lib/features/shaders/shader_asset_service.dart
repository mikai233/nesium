import 'dart:io';
import 'package:archive/archive.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import '../../logging/app_logger.dart';

class ShaderAssetService {
  static const String _zipPath = 'assets/shaders.zip';
  static const String _shadersFolder = 'shaders';
  static const String _markerFile = '.extracted';

  Future<Directory> get _targetDir async {
    final docDir = await getApplicationSupportDirectory();
    return Directory(p.join(docDir.path, _shadersFolder));
  }

  Future<void> syncShaders() async {
    final target = await _targetDir;
    final marker = File(p.join(target.path, _markerFile));

    if (marker.existsSync()) {
      logInfo(
        'Shaders already extracted at ${target.path}',
        logger: 'shader_asset_service',
      );
      return;
    }

    try {
      logInfo(
        'Extracting shaders to ${target.path}...',
        logger: 'shader_asset_service',
      );
      if (target.existsSync()) {
        target.deleteSync(recursive: true);
      }
      target.createSync(recursive: true);

      final data = await rootBundle.load(_zipPath);
      final bytes = data.buffer.asUint8List();
      final archive = ZipDecoder().decodeBytes(bytes);

      for (final file in archive) {
        final filename = file.name;
        if (file.isFile) {
          final data = file.content as List<int>;
          File(p.join(target.path, filename))
            ..createSync(recursive: true)
            ..writeAsBytesSync(data);
        } else {
          Directory(p.join(target.path, filename)).createSync(recursive: true);
        }
      }

      marker.createSync();
      logInfo('Shader extraction complete.', logger: 'shader_asset_service');
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to extract shaders',
        logger: 'shader_asset_service',
      );
    }
  }

  Future<String?> getShadersRoot() async {
    final target = await _targetDir;
    return target.path;
  }
}

final shaderAssetServiceProvider = Provider((ref) => ShaderAssetService());
