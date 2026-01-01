import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:nesium_flutter/app.dart';
import 'package:nesium_flutter/windows/window_types.dart';
import 'package:nesium_flutter/bridge/frb_generated.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());
  testWidgets('Can call rust function', (WidgetTester tester) async {
    await tester.pumpWidget(
      const ProviderScope(child: NesiumApp(windowKind: WindowKind.main)),
    );
    expect(find.textContaining('Result: `Hello, Tom!`'), findsOneWidget);
  });
}
