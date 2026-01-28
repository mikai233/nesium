import 'shader_node.dart';

class ShaderAssetService {
  Future<void> syncShaders() async {
    // No-op on web
  }

  Future<String?> getShadersRoot() async {
    return null;
  }

  Future<List<ShaderNode>> listShaders(String? relativePath) async {
    return [];
  }

  Future<List<ShaderNode>> searchShaders(String query) async {
    return [];
  }
}
