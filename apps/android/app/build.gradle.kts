// Mô-đun ứng dụng Android — Lịch và Văn hóa Việt (Clevix Lab)
//
// Thư viện Rust (liblvhv_ffi.so) do cargo-ndk dựng vào app/rustJniLibs/<abi>/ TRƯỚC khi chạy Gradle
// (xem apps/android/build-rust.sh). Mã Kotlin UniFFI được sinh vào app/build/generated/uniffi/
// bởi cùng script đó; Gradle chỉ biên dịch.
import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

// Ký bản phát hành: đọc từ biến môi trường (CI) hoặc keystore.properties (máy riêng, KHÔNG commit).
val keystoreProps = Properties().apply {
    val f = rootProject.file("keystore.properties")
    if (f.exists()) f.inputStream().use { load(it) }
}
fun signingValue(env: String, prop: String): String? =
    System.getenv(env)?.takeIf { it.isNotBlank() } ?: keystoreProps.getProperty(prop)?.takeIf { it.isNotBlank() }

val releaseStoreFile = signingValue("LVHV_KEYSTORE_FILE", "storeFile")
val releaseStorePassword = signingValue("LVHV_KEYSTORE_PASSWORD", "storePassword")
val releaseKeyAlias = signingValue("LVHV_KEY_ALIAS", "keyAlias")
val releaseKeyPassword = signingValue("LVHV_KEY_PASSWORD", "keyPassword")
val hasReleaseSigning = listOf(releaseStoreFile, releaseStorePassword, releaseKeyAlias, releaseKeyPassword).all { it != null } &&
    file(releaseStoreFile!!).exists()

val appVersionCode = (System.getenv("LVHV_VERSION_CODE") ?: "1").toInt()
val appVersionName = System.getenv("LVHV_VERSION_NAME") ?: "0.1.0"

android {
    namespace = "vn.clevix.lichviet"
    compileSdk = 35

    defaultConfig {
        applicationId = "vn.clevix.lichviet"
        minSdk = 26
        targetSdk = 35
        versionCode = appVersionCode
        versionName = appVersionName
        // Chỉ đóng gói tài nguyên tiếng Việt (và mặc định).
        resourceConfigurations += listOf("vi")
        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    signingConfigs {
        if (hasReleaseSigning) {
            create("release") {
                storeFile = file(releaseStoreFile!!)
                storePassword = releaseStorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            // Không có khóa → bản release KHÔNG ký (CI kiểm tra dựng); có khóa → ký.
            signingConfig = if (hasReleaseSigning) signingConfigs.getByName("release") else null
        }
        debug {
            applicationIdSuffix = ".debug"
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("rustJniLibs")
            java.srcDirs("build/generated/uniffi/kotlin")
        }
    }

    packaging {
        jniLibs {
            // .so không nén, căn trang 16 KB (Android 15+); AGP 8.5.1+ xử lý khi useLegacyPackaging = false.
            useLegacyPackaging = false
        }
        resources {
            excludes += listOf("/META-INF/{AL2.0,LGPL2.1}", "/META-INF/*.kotlin_module")
        }
    }

    lint {
        abortOnError = true
        checkReleaseBuilds = true
        warningsAsErrors = false
    }

    dependenciesInfo {
        // Không nhúng khối thông tin phụ thuộc (đã mã hóa bằng khóa Google) vào APK/AAB.
        includeInApk = false
        includeInBundle = false
    }
}

// Báo lỗi sớm, dễ hiểu, nếu quên chạy build-rust.sh.
val checkRustArtifacts by tasks.registering {
    doLast {
        val so = file("rustJniLibs/arm64-v8a/liblvhv_ffi.so")
        val kt = file("build/generated/uniffi/kotlin/vn/clevix/lichviet/core/lvhv_ffi.kt")
        val missing = listOf(so, kt).filterNot { it.exists() }
        if (missing.isNotEmpty()) {
            throw GradleException(
                "Thiếu thư viện Rust hoặc mã UniFFI (${missing.joinToString { it.absolutePath }}). " +
                    "Chạy apps/android/build-rust.sh trước (xem docs/04-BUILD-NATIVE.md)."
            )
        }
    }
}
tasks.named("preBuild") { dependsOn(checkRustArtifacts) }

dependencies {
    // Compose BOM 2024.12.01 (Compose 1.7.6, Material3 1.3.1).
    val composeBom = platform("androidx.compose:compose-bom:2024.12.01")
    implementation(composeBom)
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui-tooling-preview")
    debugImplementation("androidx.compose.ui:ui-tooling")

    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.7")

    // JNA cho UniFFI. Bản AAR có sẵn libjnidispatch.so cho Android. Bản 5.16.0+ căn trang 16 KB.
    implementation("net.java.dev.jna:jna:5.17.0@aar")

    testImplementation("junit:junit:4.13.2")
}
