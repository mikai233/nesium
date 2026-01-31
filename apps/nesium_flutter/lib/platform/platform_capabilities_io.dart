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

bool get isLinux => !kIsWeb && defaultTargetPlatform == TargetPlatform.linux;
bool get isMacOS => !kIsWeb && defaultTargetPlatform == TargetPlatform.macOS;
bool get isWindows =>
    !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;

bool get supportsVirtualControls => isNativeMobile;

bool get preferVirtualControlsByDefault => isNativeMobile;

bool get supportsTcp => true;

bool get useAndroidNativeGameView =>
    !kIsWeb && defaultTargetPlatform == TargetPlatform.android;
