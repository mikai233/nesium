import 'package:flutter/foundation.dart';

bool get isNativeDesktop =>
    !kIsWeb &&
    (defaultTargetPlatform == TargetPlatform.macOS ||
        defaultTargetPlatform == TargetPlatform.linux ||
        defaultTargetPlatform == TargetPlatform.windows);

bool get isNativeMobile =>
    !kIsWeb &&
    (defaultTargetPlatform == TargetPlatform.android ||
        defaultTargetPlatform == TargetPlatform.iOS);

bool get supportsVirtualControls => isNativeMobile;

bool get preferVirtualControlsByDefault => isNativeMobile;

bool get supportsTcp => true;

bool get useAndroidNativeGameView =>
    !kIsWeb && defaultTargetPlatform == TargetPlatform.android;
