import '../bridge/api/video.dart' as frb_video;

Future<frb_video.VideoOutputInfo> setVideoFilter({
  required frb_video.VideoFilter filter,
}) {
  throw UnsupportedError('Video filters are not supported on web');
}

Future<void> setNtscOptions({required frb_video.NtscOptions options}) {
  throw UnsupportedError('Video filters are not supported on web');
}
