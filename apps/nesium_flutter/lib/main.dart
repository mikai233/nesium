import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

void main() {
  runApp(const NesiumApp());
}

/// Root widget for the Nesium Flutter frontend.
class NesiumApp extends StatelessWidget {
  const NesiumApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Nesium Flutter',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const NesiumHomePage(),
    );
  }
}

/// Home page that hosts the external texture provided by the macOS Runner.
class NesiumHomePage extends StatefulWidget {
  const NesiumHomePage({super.key});

  @override
  State<NesiumHomePage> createState() => _NesiumHomePageState();
}

class _NesiumHomePageState extends State<NesiumHomePage> {
  static const _channel = MethodChannel('nesium');

  int? _textureId;
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _initTexture();
  }

  Future<void> _initTexture() async {
    try {
      final id = await _channel.invokeMethod<int>('createNesTexture');
      if (!mounted) return;
      setState(() {
        _textureId = id;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Nesium Flutter')),
      body: Center(
        child: Padding(padding: const EdgeInsets.all(16), child: _buildBody()),
      ),
    );
  }

  Widget _buildBody() {
    if (_loading) {
      return const Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          CircularProgressIndicator(),
          SizedBox(height: 12),
          Text('Initializing NES texture...'),
        ],
      );
    }

    if (_error != null) {
      return Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.error, color: Colors.red),
          const SizedBox(height: 8),
          Text(
            'Failed to create texture:',
            style: const TextStyle(fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 4),
          Text(_error!, textAlign: TextAlign.center),
        ],
      );
    }

    if (_textureId == null) {
      return const Text('Texture ID is null (unexpected).');
    }

    // NES resolution is 256x240. We preserve this aspect ratio, but scale
    // the texture to fit inside the available space without overflowing.
    return LayoutBuilder(
      builder: (context, constraints) {
        const double nesWidth = 256;
        const double nesHeight = 240;
        const double aspect = nesWidth / nesHeight;

        // We have a column: [texture] + spacer + text. Reserve a bit of
        // vertical space for the text and its margin so the texture itself
        // never pushes outside the viewport.
        const double textAndMarginHeight = 60.0;

        final double maxTextureWidth = constraints.maxWidth;
        final double maxTextureHeight =
            (constraints.maxHeight - textAndMarginHeight).clamp(
              0,
              constraints.maxHeight,
            );

        // Start by trying to use the full width, then clamp by height.
        double textureWidth = maxTextureWidth;
        double textureHeight = textureWidth / aspect;

        if (textureHeight > maxTextureHeight && maxTextureHeight > 0) {
          textureHeight = maxTextureHeight;
          textureWidth = textureHeight * aspect;
        }

        return Column(
          mainAxisSize: MainAxisSize.min,
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            SizedBox(
              width: textureWidth,
              height: textureHeight,
              child: Texture(textureId: _textureId!),
            ),
            const SizedBox(height: 12),
            const Text(
              'This view is rendered from a macOS external texture.\n'
              'Once the NES core is wired in, this will show live NES frames.',
              textAlign: TextAlign.center,
            ),
          ],
        );
      },
    );
  }
}
