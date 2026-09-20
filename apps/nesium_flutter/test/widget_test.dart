import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/app.dart';
import 'package:nesium_flutter/bridge/api/video.dart';
import 'package:nesium_flutter/bridge/frb_generated.dart';
import 'package:nesium_flutter/windows/window_types.dart';

// Only the native operations used by the empty main window are stubbed.
// Unexpected calls still fail, so startup API changes remain visible.
class _StartupApi implements RustLibApi {
  bool started = false;

  @override
  Future<void> crateApiLoadRomStartNesRuntime() async => started = true;

  @override
  Future<Uint8List?> crateApiLoadRomGetRomHash() async => null;

  @override
  Future<VideoOutputInfo> crateApiVideoSetVideoFilter({
    required VideoFilter filter,
  }) async => const VideoOutputInfo(outputWidth: 256, outputHeight: 240);

  @override
  dynamic noSuchMethod(Invocation invocation) {
    switch (invocation.memberName) {
      case #crateApiEventsRuntimeNotifications:
      case #crateApiEventsEmulationStatusStream:
      case #crateApiEventsReplayEventStream:
      case #crateApiNetplayNetplayGameEventStream:
      case #crateApiNetplayNetplayStatusStream:
      case #crateApiServerNetserverStatusStream:
        return const Stream<Never>.empty();
      case #crateApiEmulationSetIntegerFpsMode:
      case #crateApiEmulationSetHighPriorityEnabled:
      case #crateApiEmulationSetRewindConfig:
      case #crateApiEmulationSetRewindSpeed:
      case #crateApiEmulationSetFastForwardSpeed:
      case #crateApiInputSetTurboTiming:
      case #crateApiInputSetPadMask:
      case #crateApiInputSetTurboMask:
      case #crateApiPaletteSetPalettePreset:
        return Future<void>.value();
    }
    return super.noSuchMethod(invocation);
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final api = _StartupApi();
  setUpAll(() => RustLib.initMock(api: api));
  tearDownAll(RustLib.dispose);

  testWidgets('Nesium main window starts and disposes', (tester) async {
    const channel = MethodChannel('nesium');
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    messenger.setMockMethodCallHandler(channel, (call) async {
      return call.method == 'createNesTexture' ? 1 : null;
    });
    addTearDown(() => messenger.setMockMethodCallHandler(channel, null));

    await tester.pumpWidget(
      const ProviderScope(child: NesiumApp(windowKind: WindowKind.main)),
    );
    await tester.pump();
    expect(find.byType(NesiumApp), findsOneWidget);
    expect(api.started, isTrue);
    expect(tester.takeException(), isNull);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump();
    expect(tester.takeException(), isNull);
  });
}
