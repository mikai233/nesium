import 'dart:io';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import 'shader_asset_service.dart';

class ShaderNode {
  final String name;
  final String path;
  final bool isDirectory;

  ShaderNode({
    required this.name,
    required this.path,
    required this.isDirectory,
  });
}

class ShaderBrowserController extends Notifier<AsyncValue<List<ShaderNode>>> {
  final List<String> _pathStack = [];

  String? get currentPath => _pathStack.isEmpty ? null : _pathStack.last;
  bool get canGoBack => _pathStack.isNotEmpty;

  @override
  AsyncValue<List<ShaderNode>> build() {
    // Initial load
    Future.microtask(() => _loadDirectory(null));
    return const AsyncValue.loading();
  }

  Future<void> _loadDirectory(String? relativePath) async {
    state = const AsyncValue.loading();
    try {
      final assetService = ref.read(shaderAssetServiceProvider);
      final root = await assetService.getShadersRoot();
      if (root == null) {
        state = AsyncValue.error('Shaders root not found', StackTrace.current);
        return;
      }

      final absolutePath = relativePath == null
          ? root
          : p.join(root, relativePath);
      final dir = Directory(absolutePath);

      if (!dir.existsSync()) {
        state = AsyncValue.error(
          'Directory does not exist: $absolutePath',
          StackTrace.current,
        );
        return;
      }

      final entities = dir.listSync().where((e) {
        final name = p.basename(e.path);
        if (name.startsWith('.')) return false;
        if (e is File) return name.endsWith('.slangp');
        return e is Directory;
      }).toList();

      // Sort: Directories first, then files alphabetically
      entities.sort((a, b) {
        if (a is Directory && b is File) return -1;
        if (a is File && b is Directory) return 1;
        return p.basename(a.path).compareTo(p.basename(b.path));
      });

      final nodes = entities.map((e) {
        return ShaderNode(
          name: p.basename(e.path),
          path: p.relative(e.path, from: root),
          isDirectory: e is Directory,
        );
      }).toList();

      state = AsyncValue.data(nodes);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  void enterDirectory(String relativePath) {
    _pathStack.add(relativePath);
    _loadDirectory(relativePath);
  }

  void goBack() {
    if (_pathStack.isNotEmpty) {
      _pathStack.removeLast();
      _loadDirectory(_pathStack.isEmpty ? null : _pathStack.last);
    }
  }
}

final shaderBrowserProvider =
    NotifierProvider<ShaderBrowserController, AsyncValue<List<ShaderNode>>>(
      ShaderBrowserController.new,
    );
