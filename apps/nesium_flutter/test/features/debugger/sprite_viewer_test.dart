import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/bridge/frb_generated.dart';
import 'package:nesium_flutter/domain/aux_texture_ids.dart';
import 'package:nesium_flutter/domain/nes_controller.dart';
import 'package:nesium_flutter/domain/nes_state.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_controller.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_state.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/widgets/sprite_screen_preview.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/widgets/sprite_settings_menu.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';

class _LoadedNesController extends NesController {
  @override
  NesState build() => const NesState(romHash: 'test-rom');
}

class _SpriteApi implements RustLibApi {
  late StreamController<bridge.SpriteSnapshot> snapshots;
  Completer<void>? captureReady;
  int subscriptions = 0;

  @override
  Future<bridge.AuxTextureIds> crateApiEventsAuxTextureIds() async =>
      const bridge.AuxTextureIds(
        tilemap: 1,
        tile: 2,
        sprite: 3,
        spriteScreen: 4,
      );

  @override
  Future<void> crateApiEventsSetSpriteCaptureVblankStart() async {
    await captureReady?.future;
  }

  @override
  Stream<bridge.SpriteSnapshot> crateApiEventsSpriteStateStream() {
    subscriptions++;
    return snapshots.stream;
  }

  @override
  Future<void> crateApiEventsUnsubscribeSpriteState() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

bridge.SpriteSnapshot _snapshot({int height = 8}) => bridge.SpriteSnapshot(
  sprites: List.generate(
    64,
    (index) => bridge.SpriteInfo(
      index: index,
      x: index * 3,
      y: 10,
      tileIndex: index,
      palette: 0,
      flipH: false,
      flipV: false,
      behindBg: false,
      visible: true,
    ),
  ),
  thumbnailWidth: 8,
  thumbnailHeight: height,
  largeSprites: height == 16,
  patternBase: 0,
  rgbaPalette: Uint8List(128),
);

Widget _app(Widget child) => MaterialApp(
  localizationsDelegates: AppLocalizations.localizationsDelegates,
  supportedLocales: AppLocalizations.supportedLocales,
  locale: const Locale('en'),
  home: Scaffold(body: child),
);

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel('nesium_aux');
  late _SpriteApi api;
  late List<MethodCall> textureCalls;
  Future<Object?> Function(MethodCall)? nativeHandler;

  setUpAll(() {
    api = _SpriteApi();
    RustLib.initMock(api: api);
  });

  tearDownAll(RustLib.dispose);

  setUp(() {
    api.snapshots = StreamController<bridge.SpriteSnapshot>.broadcast();
    api.captureReady = null;
    api.subscriptions = 0;
    AuxTextureIdsCache.reset();
    textureCalls = [];
    nativeHandler = null;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, (call) async {
          textureCalls.add(call);
          if (nativeHandler != null) return nativeHandler!(call);
          return call.method == 'createAuxTexture'
              ? call.arguments['id']
              : null;
        });
  });

  tearDown(() async {
    await api.snapshots.close();
    AuxTextureIdsCache.reset();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, null);
  });

  Future<void> showViewer(WidgetTester tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          nesControllerProvider.overrideWith(_LoadedNesController.new),
        ],
        child: _app(const SpriteViewer()),
      ),
    );
    await tester.pump();
  }

  List<MethodCall> creations() =>
      textureCalls.where((call) => call.method == 'createAuxTexture').toList();

  testWidgets('one snapshot initializes both textures and reuses them', (
    tester,
  ) async {
    await showViewer(tester);
    api.snapshots.add(_snapshot());
    await tester.pump();
    await tester.pump();

    expect(creations().map((call) => call.arguments['id']), [3, 4]);
    expect(
      tester
          .widget<ViewerSkeletonizer>(find.byType(ViewerSkeletonizer))
          .enabled,
      isFalse,
    );
    final preview = tester.widget<SpriteScreenPreview>(
      find.byType(SpriteScreenPreview),
    );
    expect(preview.screenTextureId, 4);
    expect(preview.thumbTextureId, 3);

    api.snapshots.add(_snapshot());
    await tester.pump();
    expect(creations(), hasLength(2));

    api.snapshots.add(_snapshot(height: 16));
    await tester.pump();
    await tester.pump();
    expect(creations(), hasLength(3));
    expect(creations().last.arguments, {'id': 3, 'width': 64, 'height': 128});
    await tester.pumpWidget(const SizedBox.shrink());
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
    await tester.pump();
  });

  testWidgets(
    'a size change during creation is applied without another snapshot',
    (tester) async {
      final firstTexture = Completer<int>();
      nativeHandler = (call) async {
        if (call.method != 'createAuxTexture') return null;
        if (call.arguments['id'] == 3 && call.arguments['height'] == 64) {
          return firstTexture.future;
        }
        return call.arguments['id'];
      };
      await showViewer(tester);
      api.snapshots.add(_snapshot());
      await tester.pump();
      api.snapshots.add(_snapshot(height: 16));
      await tester.pump();
      firstTexture.complete(3);
      await tester.pump();
      await tester.pump();
      expect(creations().map((call) => call.arguments['id']), [3, 4, 3]);
      expect(creations().last.arguments['height'], 128);
      expect(
        tester
            .widget<ViewerSkeletonizer>(find.byType(ViewerSkeletonizer))
            .enabled,
        isFalse,
      );
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      await tester.pump();
    },
  );

  testWidgets(
    'closing during texture creation disposes the completed texture',
    (tester) async {
      final pendingTexture = Completer<int>();
      nativeHandler = (call) async {
        if (call.method == 'createAuxTexture') {
          return await pendingTexture.future;
        }
        return null;
      };
      await showViewer(tester);
      api.snapshots.add(_snapshot());
      await tester.pump();
      expect(creations(), hasLength(1));
      await tester.pumpWidget(const SizedBox.shrink());
      pendingTexture.complete(3);
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      await tester.pump();
      await tester.pump();
      expect(textureCalls.last.method, 'disposeAuxTexture');
      expect(textureCalls.last.arguments['id'], 3);
      expect(creations(), hasLength(1));
      expect(tester.takeException(), isNull);
    },
  );

  test('closing during capture setup does not subscribe afterwards', () async {
    api.captureReady = Completer<void>();
    final controller = SpriteViewerController();
    final started = controller.startStreaming(
      captureState: const SpriteViewerCaptureState(),
      onSnapshot: (_) {},
    );
    await controller.dispose();
    api.captureReady!.complete();
    await started;
    expect(api.subscriptions, 0);
  });

  testWidgets('preview zoom limits follow the current viewport', (
    tester,
  ) async {
    final transform = TransformationController();
    final overlay = ValueNotifier(
      const SpriteOverlayData(sprites: [], largeSprites: false),
    );
    addTearDown(transform.dispose);
    addTearDown(overlay.dispose);
    Future<void> resize(double width) async {
      await tester.pumpWidget(
        _app(
          Center(
            child: SizedBox(
              width: width,
              height: width,
              child: SpriteScreenPreview(
                snapshot: _snapshot(),
                screenTextureId: null,
                thumbTextureId: null,
                overlayDataListenable: overlay,
                backgroundBuilder: const SizedBox.expand(),
                showOutline: false,
                showOffscreenRegions: false,
                hoveredIndex: null,
                selectedIndex: null,
                selectedPosition: null,
                transformationController: transform,
                maxScale: 12,
                onViewportChanged: (_, _, _) {},
                onHover: (_, _, _) {},
                onHoverExit: () {},
                onTap: (_, _) {},
              ),
            ),
          ),
        ),
      );
    }

    await resize(128);
    expect(
      tester.widget<InteractiveViewer>(find.byType(InteractiveViewer)).minScale,
      closeTo(126 / 256, 0.001),
    );
    await resize(512);
    expect(
      tester.widget<InteractiveViewer>(find.byType(InteractiveViewer)).minScale,
      1,
    );
    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('menu retains consecutive display and capture edits', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(1000, 1100));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    var display = const SpriteViewerDisplayOptions();
    var capture = const SpriteViewerCaptureState();
    final scanline = TextEditingController(text: '0');
    final dot = TextEditingController(text: '0');
    addTearDown(scanline.dispose);
    addTearDown(dot.dispose);
    final applied = <SpriteViewerCaptureState>[];
    await tester.pumpWidget(
      _app(
        Builder(
          builder: (context) => Align(
            alignment: Alignment.topLeft,
            child: TextButton(
              onPressed: () => showSpriteViewerSettingsMenu(
                context: context,
                displayOptions: display,
                captureState: capture,
                scanlineController: scanline,
                dotController: dot,
                onDisplayOptionsChanged: (value) => display = value,
                onCaptureStateChanged: (value) => capture = value,
                onApplyCaptureMode: () => applied.add(capture),
              ),
              child: const Text('Settings'),
            ),
          ),
        ),
      ),
    );
    final l10n = AppLocalizations.of(tester.element(find.text('Settings')))!;
    await tester.tap(find.text('Settings'));
    await tester.pumpAndSettle();
    await tester.tap(find.text(l10n.spriteViewerShowGrid));
    await tester.pumpAndSettle();
    expect(
      tester
          .widget<CheckboxListTile>(
            find.widgetWithText(CheckboxListTile, l10n.spriteViewerShowGrid),
          )
          .value,
      isFalse,
    );
    await tester.tap(find.text(l10n.spriteViewerShowOutline));
    await tester.pumpAndSettle();
    expect(display.showGrid, isFalse);
    expect(display.showOutline, isTrue);

    await tester.ensureVisible(find.text(l10n.tilemapCaptureManual));
    await tester.tap(find.text(l10n.tilemapCaptureManual));
    await tester.pumpAndSettle();
    final fields = find.byType(TextField);
    expect(tester.widget<TextField>(fields.first).enabled, isTrue);
    await tester.enterText(fields.first, '123');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    await tester.enterText(fields.last, '45');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    expect(capture.mode, SpriteCaptureMode.scanline);
    expect(capture.scanline, 123);
    expect(capture.dot, 45);
    expect(applied.last.scanline, 123);
    expect(applied.last.dot, 45);
    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pumpAndSettle();
  });
}
