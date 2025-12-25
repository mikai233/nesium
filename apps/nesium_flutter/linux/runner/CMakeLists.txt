cmake_minimum_required(VERSION 3.13)

project(runner LANGUAGES CXX)

# ---- Rust shared library (cdylib) build + bundling ----
#
# We build the Rust shared library when building the Linux runner and then copy
# it into the Flutter bundle directory so the executable can load it.
set(NESIUM_REPO_ROOT "${CMAKE_CURRENT_LIST_DIR}/../../../..")
set(NESIUM_RUST_CRATE_DIR "${NESIUM_REPO_ROOT}/crates/nesium-flutter")

# Use release-dist for Flutter Release builds; otherwise use the regular release
# profile to keep iteration times reasonable.
if(CMAKE_BUILD_TYPE STREQUAL "Release")
  set(NESIUM_RUST_PROFILE "release-dist")
  set(NESIUM_RUST_PROFILE_DIR "release-dist")
else()
  set(NESIUM_RUST_PROFILE "release")
  set(NESIUM_RUST_PROFILE_DIR "release")
endif()

set(NESIUM_RUST_LIB_SRC "${NESIUM_REPO_ROOT}/target/${NESIUM_RUST_PROFILE_DIR}/libnesium_flutter.so")

# Flutter's Linux template sets CMAKE_RUNTIME_OUTPUT_DIRECTORY to the bundle dir.
# Fall back to ${CMAKE_BINARY_DIR}/bundle if it's not set for some reason.
if(DEFINED CMAKE_RUNTIME_OUTPUT_DIRECTORY)
  set(NESIUM_BUNDLE_DIR "${CMAKE_RUNTIME_OUTPUT_DIRECTORY}")
else()
  set(NESIUM_BUNDLE_DIR "${CMAKE_BINARY_DIR}/bundle")
endif()

set(NESIUM_RUST_LIB_DST_DIR "${NESIUM_BUNDLE_DIR}/lib")
set(NESIUM_RUST_LIB_DST "${NESIUM_RUST_LIB_DST_DIR}/libnesium_flutter.so")

# Define the application target. To change its name, change BINARY_NAME in the
# top-level CMakeLists.txt, not the value here, or `flutter run` will no longer
# work.
#
# Any new source files that you add to the application should be added here.
add_executable(${BINARY_NAME}
  "main.cc"
  "my_application.cc"
  "${FLUTTER_MANAGED_DIR}/generated_plugin_registrant.cc"
  "nesium/nesium_channels.cc"
  "nesium/nesium_texture.cc"
)

# Build the Rust library and copy it into the bundle.
#
# NOTE:
# The runner links against libnesium_flutter.so, so the .so must exist before
# the final link step. We therefore use a dedicated custom target and make the
# runner depend on it.
add_custom_target(nesium_rust_lib ALL
  COMMAND ${CMAKE_COMMAND} -E echo "[nesium] building Rust library (${NESIUM_RUST_PROFILE})"
  COMMAND cargo build --manifest-path "${NESIUM_RUST_CRATE_DIR}/Cargo.toml" --profile ${NESIUM_RUST_PROFILE}
  COMMAND ${CMAKE_COMMAND} -E make_directory "${NESIUM_RUST_LIB_DST_DIR}"
  COMMAND ${CMAKE_COMMAND} -E copy_if_different "${NESIUM_RUST_LIB_SRC}" "${NESIUM_RUST_LIB_DST}"
  WORKING_DIRECTORY "${NESIUM_REPO_ROOT}"
  VERBATIM
)

add_dependencies(${BINARY_NAME} nesium_rust_lib)

# Import the Rust shared library and link it.
add_library(nesium_flutter SHARED IMPORTED GLOBAL)
set_target_properties(nesium_flutter PROPERTIES
  IMPORTED_LOCATION "${NESIUM_RUST_LIB_SRC}"
)

target_link_libraries(${BINARY_NAME} PRIVATE nesium_flutter)

# Ensure the loader can find bundled shared libraries at runtime.
set_target_properties(${BINARY_NAME} PROPERTIES
  BUILD_RPATH "$ORIGIN/lib"
  INSTALL_RPATH "$ORIGIN/lib"
)

# Apply the standard set of build settings. This can be removed for applications
# that need different build settings.
apply_standard_settings(${BINARY_NAME})

# Add preprocessor definitions for the application ID.
add_definitions(-DAPPLICATION_ID="${APPLICATION_ID}")

# Add dependency libraries. Add any application-specific dependencies here.
target_link_libraries(${BINARY_NAME} PRIVATE flutter)
target_link_libraries(${BINARY_NAME} PRIVATE PkgConfig::GTK)

target_include_directories(${BINARY_NAME} PRIVATE "${CMAKE_SOURCE_DIR}")
