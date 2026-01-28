import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'shader_asset_service_io.dart'
    if (dart.library.js_interop) 'shader_asset_service_web.dart';

export 'shader_asset_service_io.dart'
    if (dart.library.js_interop) 'shader_asset_service_web.dart';

export 'shader_node.dart';

final shaderAssetServiceProvider = Provider((ref) => ShaderAssetService());
