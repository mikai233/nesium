import 'app_storage.dart';
import 'storage_codec.dart';

final class StorageKey<T> {
  const StorageKey(this.name, this.codec);

  final String name;
  final StorageCodec<T> codec;
}

extension AppStorageTypedX on AppStorage {
  T? read<T>(StorageKey<T> key) => key.codec.decode(get<Object?>(key.name));

  Future<void> write<T>(StorageKey<T> key, T value) =>
      put(key.name, key.codec.encode(value));
}
