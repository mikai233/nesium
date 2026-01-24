export '../bridge/api/video.dart'
    show NtscOptions, VideoFilter, VideoOutputInfo;

import '../bridge/api/video.dart' as frb_video;

Future<frb_video.VideoOutputInfo> setVideoFilter({
  required frb_video.VideoFilter filter,
}) => frb_video.setVideoFilter(filter: filter);

Future<void> setNtscOptions({required frb_video.NtscOptions options}) =>
    frb_video.setNtscOptions(options: options);
