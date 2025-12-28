import '../bridge/api/input.dart' as frb_input;

Future<void> setPadMask({required int pad, required int mask}) =>
    frb_input.setPadMask(pad: pad, mask: mask);

Future<void> setTurboMask({required int pad, required int mask}) =>
    frb_input.setTurboMask(pad: pad, mask: mask);

Future<void> setTurboTiming({required int onFrames, required int offFrames}) =>
    frb_input.setTurboTiming(onFrames: onFrames, offFrames: offFrames);
