import 'dart:io';
import 'package:archive/archive.dart';
import 'package:flutter/services.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import '../../logging/app_logger.dart';
import 'shader_node.dart';

class ShaderAssetService {
  static const String _zipPath = 'assets/bundled/shaders.zip';
  static const String _hashPath = 'assets/bundled/shaders.zip.md5';
  static const String _shadersFolder = 'shaders';
  static const String _markerFile = '.md5';

  Future<Directory> get _targetDir async {
    final docDir = await getApplicationSupportDirectory();
    return Directory(p.join(docDir.path, _shadersFolder));
  }

  Future<void> syncShaders() async {
    final target = await _targetDir;
    final marker = File(p.join(target.path, _markerFile));

    String? bundledHash;
    try {
      bundledHash = (await rootBundle.loadString(_hashPath)).trim();
    } catch (_) {
      logWarning(
        'Could not load bundled shader hash from $_hashPath',
        logger: 'shader_asset_service',
      );
    }

    if (marker.existsSync() && bundledHash != null) {
      final storedHash = await marker.readAsString();
      if (storedHash.trim() == bundledHash) {
        logInfo(
          'Shaders up to date ($bundledHash)',
          logger: 'shader_asset_service',
        );
        return;
      }
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

      try {
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
            Directory(
              p.join(target.path, filename),
            ).createSync(recursive: true);
          }
        }

        if (bundledHash != null) {
          marker.writeAsStringSync(bundledHash);
        } else {
          // If no bundled hash is available, create an empty marker file.
          // This serves as a flag that the initial extraction has successfully completed,
          // preventing redundant extraction attempts on subsequent app launches.
          marker.createSync();
        }
        logInfo('Shader extraction complete.', logger: 'shader_asset_service');
      } catch (e) {
        logWarning(
          'Failed to load shaders from $_zipPath. Shaders will not be available.',
          logger: 'shader_asset_service',
        );
      }
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

  Future<List<ShaderNode>> listShaders(String? relativePath) async {
    final root = await getShadersRoot();
    if (root == null) return [];

    final absolutePath = relativePath == null
        ? root
        : p.join(root, relativePath);
    final dir = Directory(absolutePath);

    if (!dir.existsSync()) {
      throw FileSystemException('Directory does not exist', absolutePath);
    }

    final entities = dir.listSync().where((e) {
      final name = p.basename(e.path);
      if (name.startsWith('.')) return false;
      if (e is File) return name.endsWith('.slangp');
      return e is Directory;
    }).toList();

    entities.sort((a, b) {
      if (a is Directory && b is File) return -1;
      if (a is File && b is Directory) return 1;
      return p.basename(a.path).compareTo(p.basename(b.path));
    });

    return entities.map((e) {
      return ShaderNode(
        name: p.basename(e.path),
        path: p.relative(e.path, from: root),
        isDirectory: e is Directory,
      );
    }).toList();
  }

  Future<List<ShaderNode>> searchShaders(String query) async {
    final root = await getShadersRoot();
    if (root == null) return [];

    final dir = Directory(root);
    if (!dir.existsSync()) return [];

    final lowerQuery = query.toLowerCase();

    // Recursive listing
    final entities = dir.listSync(recursive: true).whereType<File>().where((f) {
      final name = p.basename(f.path);
      return name.endsWith('.slangp') &&
          name.toLowerCase().contains(lowerQuery);
    }).toList();

    entities.sort((a, b) => p.basename(a.path).compareTo(p.basename(b.path)));

    return entities.map((e) {
      return ShaderNode(
        name: p.basename(e.path),
        path: p.relative(e.path, from: root),
        isDirectory: false,
      );
    }).toList();
  }
}
