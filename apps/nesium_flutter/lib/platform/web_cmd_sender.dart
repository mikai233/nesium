typedef WebCmdSender = void Function(String cmd, Map<String, Object?>? extra);

WebCmdSender? _sender;
bool _nesReady = false;

void setWebCmdSender(WebCmdSender? sender) {
  _sender = sender;
}

void setWebNesReady(bool ready) {
  _nesReady = ready;
}

bool get isWebNesReady => _nesReady;

void webPostCmd(String cmd, [Map<String, Object?>? extra]) {
  _sender?.call(cmd, extra);
}
