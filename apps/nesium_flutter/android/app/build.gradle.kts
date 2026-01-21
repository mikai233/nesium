plugins {
    id("com.android.application")
    id("kotlin-android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

android {
    namespace = "io.github.mikai233.nesium"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_17.toString()
    }

    defaultConfig {
        // TODO: Specify your own unique Application ID (https://developer.android.com/studio/build/application-id.html).
        applicationId = "io.github.mikai233.nesium"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        minSdk = 26
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    buildTypes {
        release {
            // TODO: Add your own signing config for the release build.
            // Signing with the debug keys for now, so `flutter run --release` works.
            signingConfig = signingConfigs.getByName("debug")
        }
    }
}

flutter {
    source = "../.."
}


// --- Rust/Android integration (always rebuild, workspace-friendly) ---
//
// This task builds your Rust dynamic library (.so) for all Android ABIs
// and places the outputs into Android's jniLibs folder so Flutter can load
// them via dart:ffi.
//
// Why workspace root?
// - In a Cargo workspace with complex dependencies, building from the crate
//   directory may still be OK, but pointing Cargo at the workspace root is
//   often more reliable (path deps, workspace patches, shared features, etc).

val repoRootDir = file("../../../../") // from apps/nesium_flutter/android/app -> repo root
val rustWorkspaceDir = repoRootDir     // adjust if your workspace root is elsewhere
val rustPackageName = "nesium-flutter" // Cargo package name (with hyphen)
val jniLibsOutDir = file("$projectDir/src/main/jniLibs")

fun cargoCmd(): List<String> {
    // Windows requires "cmd /c" to run cargo reliably from Gradle Exec.
    val isWindows = org.gradle.internal.os.OperatingSystem.current().isWindows
    return if (isWindows) {
        listOf("cmd", "/c", "cargo")
    } else {
        listOf("cargo")
    }
}

tasks.register<Exec>("buildRustAndroidSo") {
    group = "build"
    description = "Build Rust cdylib (.so) for Android ABIs and copy into jniLibs (always rebuild)"

    // Always rebuild: no inputs/outputs / no incremental logic on purpose.
    // If you want to avoid any stale artifacts, you can also add a clean step.

    workingDir = rustWorkspaceDir
    outputs.dir(jniLibsOutDir)

    doFirst {
        // Detect target architectures from Flutter properties
        val targetPlatform = project.findProperty("target-platform") as String?
        val flutterPlatforms =
            (project.findProperty("flutter.targetPlatforms") as String?)?.split(",") ?: emptyList()

        val requestedAbis = mutableListOf<String>()
        if (targetPlatform != null) {
            when (targetPlatform) {
                "android-arm" -> requestedAbis.add("armeabi-v7a")
                "android-arm64" -> requestedAbis.add("arm64-v8a")
                "android-x64" -> requestedAbis.add("x86_64")
            }
        }

        if (requestedAbis.isEmpty()) {
            for (p in flutterPlatforms) {
                when (p.trim()) {
                    "android-arm" -> requestedAbis.add("armeabi-v7a")
                    "android-arm64" -> requestedAbis.add("arm64-v8a")
                    "android-x64" -> requestedAbis.add("x86_64")
                }
            }
        }

        val finalAbis = if (requestedAbis.isNotEmpty()) requestedAbis else listOf(
            "armeabi-v7a",
            "arm64-v8a",
            "x86_64"
        )
        val abiArgs = finalAbis.flatMap { listOf("-t", it) }

        logger.lifecycle("Rust build targeting ABIs: $finalAbis")

        // Clean up jniLibs to avoid stale artifacts from previous builds
        if (jniLibsOutDir.exists()) {
            jniLibsOutDir.deleteRecursively()
        }
        jniLibsOutDir.mkdirs()

        // Choose Rust build profile based on the invoked Gradle tasks.
        // - For Flutter/Android release builds (assembleRelease / bundleRelease / etc.): use `release-dist`.
        // - For all other builds: use `release`.
        val buildModeProp = (project.findProperty("flutter.buildMode") as String?)?.lowercase()
        val isReleaseTask =
            gradle.startParameter.taskNames.any { it.contains("release", ignoreCase = true) }
        val isDebugTask =
            gradle.startParameter.taskNames.any { it.contains("debug", ignoreCase = true) }
        val isProfileTask =
            gradle.startParameter.taskNames.any { it.contains("profile", ignoreCase = true) }

        val isFlutterRelease =
            buildModeProp == "release" || (isReleaseTask && !isDebugTask && !isProfileTask)
        val rustProfile = if (isFlutterRelease) "release-dist" else "release"

        // Build the specific package inside the workspace.
        // `--profile <name>` lets us select custom profiles like `release-dist`.
        commandLine(
            cargoCmd() + listOf(
                "ndk",
                "--platform", "26",
            ) + abiArgs + listOf(
                "-o", jniLibsOutDir.absolutePath,
                "build",
                "--profile", rustProfile,
                "-p", rustPackageName,
            )
        )
    }

    // Build the specific package inside the workspace.
    // -P <package> ensures the correct crate is built even in a large workspace.
    //
    // Output layout will be:
    //   android/app/src/main/jniLibs/arm64-v8a/lib<crate_name>.so
    //   android/app/src/main/jniLibs/armeabi-v7a/lib<crate_name>.so
    //   android/app/src/main/jniLibs/x86_64/lib<crate_name>.so
    //   android/app/src/main/jniLibs/x86/lib<crate_name>.so
    //
    // Note: The produced library name usually converts '-' to '_' (e.g. libnesium_flutter.so).

    // If your NDK isn't auto-detected, uncomment and set ANDROID_NDK_HOME.
    // environment("ANDROID_NDK_HOME", System.getenv("ANDROID_NDK_HOME") ?: "/absolute/path/to/ndk")
}

// Ensure Rust is built before Android builds the APK/AAB.
tasks.named("preBuild") {
    dependsOn("buildRustAndroidSo")
}
