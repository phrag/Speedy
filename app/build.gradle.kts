plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "dev.speedy"
    compileSdk = 35

    defaultConfig {
        applicationId = "dev.speedy"
        minSdk = 34       // covers Pixel 10 / Android 16; specialUse FGS available
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"

        // Pixel 10 is arm64-only, so we ship a single ABI.
        ndk {
            abiFilters += "arm64-v8a"
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    // The Kotlin sources live under src/main/kotlin (AGP also scans this by default).
    sourceSets["main"].kotlin.srcDir("src/main/kotlin")

    // Store libspeedy_core.so uncompressed/page-aligned so it isn't extracted at
    // install time — smaller install footprint and faster first load.
    packaging {
        jniLibs.useLegacyPackaging = false
    }

    // libspeedy_core.so is produced by scripts/build-rust.sh into src/main/jniLibs.
    lint {
        // Keep lint informative without failing the build on style warnings.
        abortOnError = false
    }
}

dependencies {
    testImplementation("junit:junit:4.13.2")
}
