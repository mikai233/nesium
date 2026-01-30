export '../bridge/api/video.dart'
    show
        LcdGridOptions,
        NtscOptions,
        NtscBisqwitOptions,
        ScanlineOptions,
        VideoFilter,
        VideoOutputInfo,
        ShaderParameters;

import '../bridge/api/video.dart' as frb_video;

Future<frb_video.VideoOutputInfo> setVideoFilter({
  required frb_video.VideoFilter filter,
}) => frb_video.setVideoFilter(filter: filter);

Future<void> setNtscOptions({required frb_video.NtscOptions options}) =>
    frb_video.setNtscOptions(options: options);

Future<void> setLcdGridOptions({required frb_video.LcdGridOptions options}) =>
    frb_video.setLcdGridOptions(options: options);

Future<void> setScanlineOptions({required frb_video.ScanlineOptions options}) =>
    frb_video.setScanlineOptions(options: options);

Future<void> setNtscBisqwitOptions({
  required frb_video.NtscBisqwitOptions options,
}) => frb_video.setNtscBisqwitOptions(options: options);

Future<void> setShaderEnabled({required bool enabled}) =>
    frb_video.setShaderEnabled(enabled: enabled);

Future<frb_video.ShaderParameters> setShaderPresetPath({String? path}) =>
    frb_video.setShaderPresetPath(path: path);

Future<frb_video.ShaderParameters> setShaderConfig({
  required bool enabled,
  String? path,
}) => frb_video.setShaderConfig(enabled: enabled, path: path);
