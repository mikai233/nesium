typedef WebCmdSender = void Function(String cmd, Map<String, Object?>? extra);
typedef WebRequestSender =
    Future<T> Function<T>(String cmd, [Map<String, Object?>? extra]);

WebCmdSender? _sender;
WebRequestSender? _requestSender;
bool _nesReady = false;

void setWebCmdSender(WebCmdSender? sender) {
  _sender = sender;
}

void setWebRequestSender(WebRequestSender? sender) {
  _requestSender = sender;
}

void setWebNesReady(bool ready) {
  _nesReady = ready;
}

bool get isWebNesReady => _nesReady;

void webPostCmd(String cmd, [Map<String, Object?>? extra]) {
  _sender?.call(cmd, extra);
}

Future<T> webRequest<T>(String cmd, [Map<String, Object?>? extra]) async {
  if (_requestSender == null) {
    throw StateError('Web request sender not initialized');
  }
  return _requestSender!<T>(cmd, extra);
}
