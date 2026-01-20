import 'dart:io';

Future<void> main(List<String> forwarded) async {
  final scriptFile = File.fromUri(Platform.script);
  final appDir = scriptFile.parent.parent.absolute;

  final buildExitCode = await _runStreaming(
    executable: Platform.resolvedExecutable,
    args: const ['run', 'tool/build_wasm.dart'],
    workingDirectory: appDir.path,
  );
  if (buildExitCode != 0) exit(buildExitCode);

  final flutterArgs = <String>['run', ...forwarded];
  final hasDeviceArg =
      forwarded.contains('-d') ||
      forwarded.contains('--device-id') ||
      forwarded.any((a) => a.startsWith('-d=') || a.startsWith('--device-id='));
  if (!hasDeviceArg) {
    flutterArgs.insertAll(1, const ['-d', 'chrome']);
  }

  final exitCode = await _runStreaming(
    executable: 'flutter',
    args: flutterArgs,
    workingDirectory: appDir.path,
  );
  exit(exitCode);
}

Future<int> _runStreaming({
  required String executable,
  required List<String> args,
  required String workingDirectory,
}) async {
  final proc = await Process.start(
    executable,
    args,
    workingDirectory: workingDirectory,
    mode: ProcessStartMode.inheritStdio,
    runInShell: true,
  );
  return proc.exitCode;
}
