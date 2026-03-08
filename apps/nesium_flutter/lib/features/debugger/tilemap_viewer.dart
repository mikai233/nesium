import 'dart:async';
import 'dart:typed_data';

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

part 'tilemap_viewer/tilemap_viewer_types.dart';
part 'tilemap_viewer/tilemap_viewer_models.dart';
part 'tilemap_viewer/tilemap_viewer_painters.dart';
part 'tilemap_viewer/tilemap_viewer_state_base.dart';
part 'tilemap_viewer/tilemap_viewer_widgets.dart';
part 'tilemap_viewer/tilemap_viewer_ui.dart';

/// Tilemap Viewer that displays NES nametables via a Flutter Texture.
class TilemapViewer extends ConsumerStatefulWidget {
  const TilemapViewer({super.key});

  @override
  ConsumerState<TilemapViewer> createState() => _TilemapViewerState();
}

class _TilemapViewerState extends _TilemapViewerStateBase
    with _TilemapViewerUiMixin {
  @override
  Widget build(BuildContext context) {
    if (_error != null) {
      return _buildErrorState();
    }
    final hasRom = ref.watch(nesControllerProvider).romHash != null;
    final loading = !hasRom || _isCreating || _flutterTextureId == null;
    return ViewerSkeletonizer(
      enabled: loading,
      child: _buildMainLayout(context),
    );
  }
}
