cmake_minimum_required(VERSION 3.14)
project(runner LANGUAGES CXX)

# Define the application target. To change its name, change BINARY_NAME in the
# top-level CMakeLists.txt, not the value here, or `flutter run` will no longer
# work.
#
# Any new source files that you add to the application should be added here.
add_executable(${BINARY_NAME} WIN32
  "flutter_window.cpp"
  "main.cpp"
  "utils.cpp"
  "win32_window.cpp"
  "${FLUTTER_MANAGED_DIR}/generated_plugin_registrant.cc"
  "Runner.rc"
  "runner.exe.manifest"
  "nesium/nesium_texture_plugin.cpp"
  "nesium/nesium_aux_texture_plugin.cpp"
)

# Apply the standard set of build settings. This can be removed for applications
# that need different build settings.
apply_standard_settings(${BINARY_NAME})

# Add preprocessor definitions for the build version.
target_compile_definitions(${BINARY_NAME} PRIVATE "FLUTTER_VERSION=\"${FLUTTER_VERSION}\"")
target_compile_definitions(${BINARY_NAME} PRIVATE "FLUTTER_VERSION_MAJOR=${FLUTTER_VERSION_MAJOR}")
target_compile_definitions(${BINARY_NAME} PRIVATE "FLUTTER_VERSION_MINOR=${FLUTTER_VERSION_MINOR}")
target_compile_definitions(${BINARY_NAME} PRIVATE "FLUTTER_VERSION_PATCH=${FLUTTER_VERSION_PATCH}")
target_compile_definitions(${BINARY_NAME} PRIVATE "FLUTTER_VERSION_BUILD=${FLUTTER_VERSION_BUILD}")

# Disable Windows macros that collide with C++ standard library functions.
target_compile_definitions(${BINARY_NAME} PRIVATE "NOMINMAX")

# Add dependency libraries and include directories. Add any application-specific
# dependencies here.
target_link_libraries(${BINARY_NAME} PRIVATE flutter flutter_wrapper_app flutter_wrapper_plugin)
target_link_libraries(${BINARY_NAME} PRIVATE "dwmapi.lib")
target_include_directories(${BINARY_NAME} PRIVATE "${CMAKE_SOURCE_DIR}")

# Run the Flutter tool portions of the build. This must not be removed.
add_dependencies(${BINARY_NAME} flutter_assemble)


# =========================================================
# Rust Integration: Build and Link Strategy
# =========================================================

# 1. Determine the Workspace Root
#    Assumes this CMakeLists.txt is located at: <workspace>/apps/nesium_flutter/windows/runner
#    We need to navigate up 4 levels to reach the Rust workspace root.
get_filename_component(NESIUM_WORKSPACE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../.." ABSOLUTE)

# 2. Configuration Definitions
set(RUST_PACKAGE_NAME "nesium-flutter")
set(RUST_LIB_NAME     "nesium_flutter") # Name of the generated DLL (without extension)

# 3. Detect Target Architecture
if(CMAKE_VS_PLATFORM_NAME STREQUAL "ARM64")
    set(RUST_TARGET "aarch64-pc-windows-msvc")
else()
    set(RUST_TARGET "x86_64-pc-windows-msvc")
endif()

# 4. Define Output Paths
#    The DLL will be temporarily staged in the CMake binary directory before being copied to the Runner.
set(RUST_DLL_STAGING "${CMAKE_CURRENT_BINARY_DIR}/${RUST_LIB_NAME}.dll")
set(BUILD_SCRIPT     "${CMAKE_CURRENT_BINARY_DIR}/cargo_build_runner.bat")

# 5. Generate the Batch Build Script
#    This script handles the environment setup, Cargo execution, and artifact retrieval.
file(WRITE "${BUILD_SCRIPT}" "
@echo off
setlocal enabledelayedexpansion

:: Arguments passed from CMake
set PACKAGE_NAME=%1
set TARGET_ARCH=%2
set BUILD_MODE=%3
set OUTPUT_DEST=%4

:: --- [Phase 1] Environment Setup ---
:: Clear potential conflicts from VS environment variables to ensure a clean Rust build.
set LIB=
set LIBPATH=
set INCLUDE=
set VSINSTALLDIR=
set VCINSTALLDIR=
set PATH=C:\\Windows\\system32;C:\\Windows;C:\\Windows\\System32\\Wbem;C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\;%USERPROFILE%\\.cargo\\bin

:: --- [Phase 2] Determine Cargo Profile ---
:: Logic: Use 'release-dist' for Release builds, otherwise default to standard 'release'.
if \"%BUILD_MODE%\"==\"release-dist\" (
    echo [Rust] Build Profile: RELEASE-DIST
    set CARGO_ARGS=--profile release-dist
    set SUBDIR=release-dist
) else (
    echo [Rust] Build Profile: RELEASE (Standard)
    set CARGO_ARGS=--release
    set SUBDIR=release
)

:: --- [Phase 3] Execute Cargo Build ---
echo [Rust] Running: cargo build -p %PACKAGE_NAME% --target %TARGET_ARCH% %CARGO_ARGS%
cargo build -p %PACKAGE_NAME% --target %TARGET_ARCH% %CARGO_ARGS%

if %errorlevel% neq 0 (
    echo [Rust] Fatal Error: Cargo build failed with code %errorlevel%
    exit /b %errorlevel%
)

:: --- [Phase 4] Retrieve Artifact ---
:: Locate the generated DLL within the Rust target directory and copy it to the staging area.
:: Note: The working directory is set to the Workspace Root, so paths are relative to that.
set SRC_DLL=target\\%TARGET_ARCH%\\!SUBDIR!\\${RUST_LIB_NAME}.dll

if not exist \"!SRC_DLL!\" (
    echo [Rust] Fatal Error: DLL artifact not found at: !SRC_DLL!
    echo [Rust] Please verify the cargo output directory structure.
    exit /b 1
)

echo [Rust] Copying artifact to staging area...
copy /Y \"!SRC_DLL!\" \"%OUTPUT_DEST%\" >nul
if %errorlevel% neq 0 (
    echo [Rust] Fatal Error: Failed to copy DLL.
    exit /b 1
)

echo [Rust] Build sequence completed successfully.
")

# 6. Determine Build Mode String for the Script
#    Logic: If CMake is in Release or Profile mode, pass 'release-dist'.
#           For Debug or any other mode, pass 'standard'.
set(MODE_GENEX "$<IF:$<OR:$<CONFIG:Release>,$<CONFIG:Profile>>,release-dist,standard>")

# 7. Register Custom Build Command
#    This command executes the generated batch script at build time.
add_custom_command(
    OUTPUT            "${RUST_DLL_STAGING}"
    COMMAND           "${BUILD_SCRIPT}" 
                      "${RUST_PACKAGE_NAME}" 
                      "${RUST_TARGET}" 
                      "${MODE_GENEX}" 
                      "${RUST_DLL_STAGING}"
    WORKING_DIRECTORY "${NESIUM_WORKSPACE_DIR}"
    COMMENT           "Compiling Rust backend (Mode: ${MODE_GENEX})..."
    VERBATIM
)

# 8. Define Custom Target & Dependencies
#    Ensure the Rust build runs before the main Flutter Runner links.
add_custom_target(nesium_rust_build ALL DEPENDS "${RUST_DLL_STAGING}")
add_dependencies(${BINARY_NAME} nesium_rust_build)

# 9. Post-Build Action: Deploy to Runner
#    Copy the staged DLL to the final executable directory alongside the Flutter Runner.
add_custom_command(TARGET ${BINARY_NAME} POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
    "${RUST_DLL_STAGING}"
    "$<TARGET_FILE_DIR:${BINARY_NAME}>/${RUST_LIB_NAME}.dll"
    VERBATIM
)