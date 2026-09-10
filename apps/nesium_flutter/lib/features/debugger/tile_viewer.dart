import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/domain/aux_texture_ids.dart';
import 'package:nesium_flutter/domain/nes_controller.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/logging/app_logger.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';
import 'package:nesium_flutter/widgets/single_position_scrollbar.dart';

part 'tile_viewer/tile_viewer_types.dart';
part 'tile_viewer/tile_viewer_models.dart';
part 'tile_viewer/tile_viewer_painters.dart';
part 'tile_viewer/tile_viewer_state_base.dart';
part 'tile_viewer/tile_viewer_widgets.dart';
part 'tile_viewer/tile_viewer_ui.dart';

/// Tile Viewer that displays NES CHR pattern tables via a Flutter Texture.
class TileViewer extends ConsumerStatefulWidget {
  const TileViewer({super.key});

  @override
  ConsumerState<TileViewer> createState() => _TileViewerState();
}

class _TileViewerState extends _TileViewerStateBase with _TileViewerUiMixin {
  /// Called when ROM is ejected - reset to PPU preset
  void _onRomEjected() {
    _applyPreset(_Preset.ppu);
  }

  @override
  Widget build(BuildContext context) {
    // Listen for ROM ejection to reset source to PPU
    ref.listen(nesControllerProvider, (prev, next) {
      final prevHasRom = prev?.romHash != null;
      final nextHasRom = next.romHash != null;
      if (prevHasRom && !nextHasRom) {
        _onRomEjected();
      }
    });

    if (_error != null) {
      return _buildErrorState();
    }
    final hasRom = ref.watch(nesControllerProvider).romHash != null;
    final loading = !hasRom || _isCreating || _flutterTextureId == null;
    final base = ViewerSkeletonizer(
      enabled: loading,
      child: _buildMainLayout(context),
    );
    return base;
  }
}
