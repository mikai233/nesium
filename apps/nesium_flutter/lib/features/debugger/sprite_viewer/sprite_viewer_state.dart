import 'package:flutter/foundation.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';

@immutable
class SpriteViewerDisplayOptions {
  const SpriteViewerDisplayOptions({
    this.showGrid = true,
    this.dimOffscreenGrid = true,
    this.showOutline = false,
    this.showOffscreenRegions = false,
    this.showSidePanel = true,
    this.showListView = false,
    this.background = SpriteViewerBackground.gray,
    this.dataSource = SpriteDataSource.spriteRam,
  });

  final bool showGrid;
  final bool dimOffscreenGrid;
  final bool showOutline;
  final bool showOffscreenRegions;
  final bool showSidePanel;
  final bool showListView;
  final SpriteViewerBackground background;
  final SpriteDataSource dataSource;

  SpriteViewerDisplayOptions copyWith({
    bool? showGrid,
    bool? dimOffscreenGrid,
    bool? showOutline,
    bool? showOffscreenRegions,
    bool? showSidePanel,
    bool? showListView,
    SpriteViewerBackground? background,
    SpriteDataSource? dataSource,
  }) {
    return SpriteViewerDisplayOptions(
      showGrid: showGrid ?? this.showGrid,
      dimOffscreenGrid: dimOffscreenGrid ?? this.dimOffscreenGrid,
      showOutline: showOutline ?? this.showOutline,
      showOffscreenRegions: showOffscreenRegions ?? this.showOffscreenRegions,
      showSidePanel: showSidePanel ?? this.showSidePanel,
      showListView: showListView ?? this.showListView,
      background: background ?? this.background,
      dataSource: dataSource ?? this.dataSource,
    );
  }
}

@immutable
class SpriteViewerCaptureState {
  const SpriteViewerCaptureState({
    this.mode = SpriteCaptureMode.vblankStart,
    this.scanline = 0,
    this.dot = 0,
  });

  final SpriteCaptureMode mode;
  final int scanline;
  final int dot;

  SpriteViewerCaptureState copyWith({
    SpriteCaptureMode? mode,
    int? scanline,
    int? dot,
  }) {
    return SpriteViewerCaptureState(
      mode: mode ?? this.mode,
      scanline: scanline ?? this.scanline,
      dot: dot ?? this.dot,
    );
  }
}
