import 'web_cmd_sender.dart';

Future<void> setPadMask({required int pad, required int mask}) async {
  _padMasks[pad] = mask & 0xFF;
  _flush(pad);
}

Future<void> setTurboMask({required int pad, required int mask}) async {
  _turboMasks[pad] = mask & 0xFF;
  _flush(pad);
}

Future<void> setTurboTiming({
  required int onFrames,
  required int offFrames,
}) async {
  _turboOnFrames = onFrames.clamp(1, 255);
  _turboOffFrames = offFrames.clamp(1, 255);
  _flushTurboTiming();
}

final Map<int, int> _padMasks = <int, int>{};
final Map<int, int> _turboMasks = <int, int>{};
int _turboOnFrames = 2;
int _turboOffFrames = 2;

void _flush(int pad) {
  if (!isWebNesReady) return;
  final padMask = _padMasks[pad] ?? 0;
  final turboMask = _turboMasks[pad] ?? 0;
  webPostCmd('setPad', {'port': pad, 'bits': padMask & 0xFF});
  webPostCmd('setTurboMask', {'port': pad, 'bits': turboMask & 0xFF});
}

void _flushTurboTiming() {
  if (!isWebNesReady) return;
  webPostCmd('setTurboTiming', {
    'onFrames': _turboOnFrames,
    'offFrames': _turboOffFrames,
  });
}
