import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/domain/nes_texture_service.dart';

/// Tilemap Viewer that displays NES nametables via a Flutter Texture.
class TilemapViewer extends ConsumerStatefulWidget {
  const TilemapViewer({super.key});

  @override
  ConsumerState<TilemapViewer> createState() => _TilemapViewerState();
}

class _TilemapViewerState extends ConsumerState<TilemapViewer> {
  static const int _tilemapTextureId = 1;
  static const int _width = 512;
  static const int _height = 480;

  final NesTextureService _textureService = NesTextureService();
  int? _flutterTextureId;
  bool _isCreating = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _createTexture();
  }

  Future<void> _createTexture() async {
    if (_isCreating) return;
    setState(() {
      _isCreating = true;
      _error = null;
    });

    try {
      final textureId = await _textureService.createAuxTexture(
        id: _tilemapTextureId,
        width: _width,
        height: _height,
      );

      // Subscribe to tilemap texture updates (activates Rust-side rendering).
      await bridge.subscribeTilemapTexture();

      if (mounted) {
        setState(() {
          _flutterTextureId = textureId;
          _isCreating = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = e.toString();
          _isCreating = false;
        });
      }
    }
  }

  @override
  void dispose() {
    // 1. Pause updates immediately (stops platform layer from accessing texture)
    _textureService.pauseAuxTexture(_tilemapTextureId);

    // 2. Unsubscribe from Rust tilemap events (async, but updates are paused)
    bridge.unsubscribeTilemapTexture();

    // 3. Dispose texture (safe now that updates are paused)
    _textureService.disposeAuxTexture(_tilemapTextureId);

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return _buildBody();
  }

  Widget _buildBody() {
    if (_error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.red),
            const SizedBox(height: 16),
            Text('Error: $_error'),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: _createTexture,
              child: const Text('Retry'),
            ),
          ],
        ),
      );
    }

    if (_isCreating || _flutterTextureId == null) {
      return const Center(child: CircularProgressIndicator());
    }

    return Center(
      child: AspectRatio(
        aspectRatio: _width / _height,
        child: Texture(textureId: _flutterTextureId!),
      ),
    );
  }
}
