import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'shader_asset_service.dart';

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
      final nodes = await assetService.listShaders(relativePath);
      state = AsyncValue.data(nodes);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  Future<void> search(String query) async {
    if (query.trim().isEmpty) {
      // Logic for clearing search: reload current path
      _loadDirectory(currentPath);
      return;
    }

    state = const AsyncValue.loading();
    try {
      final assetService = ref.read(shaderAssetServiceProvider);
      final nodes = await assetService.searchShaders(query);
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
