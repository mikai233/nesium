plugins {
    id("com.android.application")
    id("kotlin-android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

// --- Rust/Android isolation logic ---
val targetPlatform = project.findProperty("target-platform") as String?
val flutterPlatforms =
    (project.findProperty("flutter.targetPlatforms") as String?)?.split(",") ?: emptyList()
val abiSlug = targetPlatform
    ?: (if (flutterPlatforms.isNotEmpty()) flutterPlatforms.joinToString("-") else "universal")

// Isolated directory in 'build/' ensures total separation between builds (Universal vs ARM64)
// and forces Gradle to execute the task when switching modes.
val jniLibsOutDir = file("$buildDir/rustJniLibs/$abiSlug")

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

    // Standard JNI libs mapping using a dynamic, isolated "source" directory.
    sourceSets["main"].jniLibs.setSrcDirs(listOf(jniLibsOutDir))

    // Our Rust cdylib links against libc++_shared when we compile vendor C++ code (HQX/SaI/xBRZ/etc).
    // Ensure APK packaging doesn't fail if other dependencies also ship the same runtime.
    packaging {
        jniLibs {
            pickFirsts.add("**/libc++_shared.so")
        }
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

    aaptOptions {
        ignoreAssetsPattern = "!*.git:!.git"
    }
}

flutter {
    source = "../.."
}

val repoRootDir = file("../../../../") // from apps/nesium_flutter/android/app -> repo root
val rustWorkspaceDir = repoRootDir     // adjust if your workspace root is elsewhere
val rustPackageName = "nesium-flutter" // Cargo package name (with hyphen)

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
    description = "Build Rust cdylib (.so) for Android ABIs and copy into isolated jniLibs"

    workingDir = rustWorkspaceDir
    outputs.dir(jniLibsOutDir)

    // Always run this task; let Cargo handle incremental compilation.
    // Gradle's caching doesn't understand Rust's complex dependency graph (crates, proc-macros, etc.).
    outputs.upToDateWhen { false }

    doFirst {
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

        // Default to all for local dev robustness
        val finalAbis = requestedAbis.ifEmpty {
            listOf(
                "armeabi-v7a",
                "arm64-v8a",
                "x86_64"
            )
        }
        val abiArgs = finalAbis.flatMap { listOf("-t", it) }

        logger.lifecycle("Rust build ($abiSlug) targeting ABIs: $finalAbis -> ${jniLibsOutDir.absolutePath}")

        // Clean isolated directory
        if (jniLibsOutDir.exists()) {
            jniLibsOutDir.deleteRecursively()
        }
        jniLibsOutDir.mkdirs()

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

    doLast {
        val ndkDir = android.ndkDirectory
            ?: throw GradleException("Android NDK directory not found (android.ndkDirectory is null)")

        val os = org.gradle.internal.os.OperatingSystem.current()
        val hostTag = when {
            os.isWindows -> "windows-x86_64"
            os.isMacOsX -> "darwin-x86_64"
            os.isLinux -> "linux-x86_64"
            else -> throw GradleException("Unsupported host OS for NDK: $os")
        }

        val sysrootUsrLib = ndkDir.resolve("toolchains/llvm/prebuilt/$hostTag/sysroot/usr/lib")
        val abiToTriple = mapOf(
            "armeabi-v7a" to "arm-linux-androideabi",
            "arm64-v8a" to "aarch64-linux-android",
            "x86_64" to "x86_64-linux-android",
        )

        // Copy libc++_shared.so into the same ABI folders where cargo-ndk emitted libnesium_flutter.so.
        // This fixes runtime crashes like:
        //   java.lang.UnsatisfiedLinkError: dlopen failed: library "libc++_shared.so" not found
        jniLibsOutDir.listFiles()
            ?.filter { it.isDirectory }
            ?.forEach { abiDir ->
                val abi = abiDir.name
                val triple = abiToTriple[abi] ?: return@forEach
                val src = sysrootUsrLib.resolve("$triple/libc++_shared.so")
                if (!src.exists()) {
                    throw GradleException("NDK libc++_shared.so not found at: ${src.absolutePath}")
                }
                src.copyTo(abiDir.resolve("libc++_shared.so"), overwrite = true)
            }
    }
}

tasks.register<Exec>("zipShaders") {
    group = "build"
    description = "Zip shaders into an asset for the app"

    workingDir = file("../..") // apps/nesium_flutter
    doFirst {
        val script = "tool/package_shaders.dart"
        logger.lifecycle("Running shader packager: dart $script")
        exec {
            commandLine("dart", script)
        }
    }
}

// Ensure shaders are zipped before assets are processed or the build starts
tasks.named("preBuild") {
    dependsOn("zipShaders")
}

// Ensure Rust is built before Android builds the APK/AAB.
tasks.named("preBuild") {
    dependsOn("buildRustAndroidSo")
}
