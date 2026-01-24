export '../bridge/api/video.dart'
    show
        LcdGridOptions,
        NtscOptions,
        NtscBisqwitOptions,
        ScanlineOptions,
        VideoFilter,
        VideoOutputInfo;

import '../bridge/api/video.dart' as frb_video;
import 'web_cmd_sender.dart';

Future<frb_video.VideoOutputInfo> setVideoFilter({
  required frb_video.VideoFilter filter,
}) {
  frb_video.VideoOutputInfo outputInfoFor(frb_video.VideoFilter filter) {
    const baseW = 256;
    const baseH = 240;
    switch (filter) {
      case frb_video.VideoFilter.none:
        return const frb_video.VideoOutputInfo(
          outputWidth: baseW,
          outputHeight: baseH,
        );
      case frb_video.VideoFilter.prescale2X:
        return const frb_video.VideoOutputInfo(
          outputWidth: baseW * 2,
          outputHeight: baseH * 2,
        );
      case frb_video.VideoFilter.prescale3X:
        return const frb_video.VideoOutputInfo(
          outputWidth: baseW * 3,
          outputHeight: baseH * 3,
        );
      case frb_video.VideoFilter.prescale4X:
        return const frb_video.VideoOutputInfo(
          outputWidth: baseW * 4,
          outputHeight: baseH * 4,
        );
      case frb_video.VideoFilter.sai2X:
      case frb_video.VideoFilter.super2XSai:
      case frb_video.VideoFilter.superEagle:
        return const frb_video.VideoOutputInfo(
          outputWidth: baseW * 2,
          outputHeight: baseH * 2,
        );
      case frb_video.VideoFilter.lcdGrid:
      case frb_video.VideoFilter.scanlines:
      case frb_video.VideoFilter.xbrz2X:
      case frb_video.VideoFilter.xbrz3X:
      case frb_video.VideoFilter.xbrz4X:
      case frb_video.VideoFilter.xbrz5X:
      case frb_video.VideoFilter.xbrz6X:
        throw UnsupportedError(
          'This video filter is not supported on web yet (WASM supports PrescaleNx + SaI only).',
        );
      case frb_video.VideoFilter.hq2X:
      case frb_video.VideoFilter.hq3X:
      case frb_video.VideoFilter.hq4X:
      case frb_video.VideoFilter.ntscComposite:
      case frb_video.VideoFilter.ntscSVideo:
      case frb_video.VideoFilter.ntscRgb:
      case frb_video.VideoFilter.ntscMonochrome:
      case frb_video.VideoFilter.ntscBisqwit2X:
      case frb_video.VideoFilter.ntscBisqwit4X:
      case frb_video.VideoFilter.ntscBisqwit8X:
        throw UnsupportedError(
          'This video filter is not supported on web yet (WASM supports PrescaleNx + SaI only).',
        );
    }
  }

  final info = outputInfoFor(filter);
  if (!isWebNesReady) return Future.value(info);

  webPostCmd('setVideoFilter', {'filter': filter.index});
  return Future.value(info);
}

Future<void> setNtscOptions({required frb_video.NtscOptions options}) {
  // NTSC filters are not supported on web yet.
  return Future.value();
}

Future<void> setLcdGridOptions({required frb_video.LcdGridOptions options}) {
  // LCD Grid is not supported on web yet.
  return Future.value();
}

Future<void> setScanlineOptions({required frb_video.ScanlineOptions options}) {
  // Scanlines are not supported on web yet.
  return Future.value();
}

Future<void> setNtscBisqwitOptions({
  required frb_video.NtscBisqwitOptions options,
}) {
  // NTSC (Bisqwit) is not supported on web yet.
  return Future.value();
}
