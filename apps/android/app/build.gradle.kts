// Mô-đun ứng dụng Android — Lịch và Văn hóa Việt (Clevix Lab)
//
// Thư viện Rust (liblvhv_ffi.so) do cargo-ndk dựng vào app/rustJniLibs/<abi>/ TRƯỚC khi chạy Gradle
// (xem apps/android/build-rust.sh). Mã Kotlin UniFFI được sinh vào app/build/generated/uniffi/
// bởi cùng script đó; Gradle chỉ biên dịch.
import java.time.LocalDate
import java.time.ZoneOffset
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

// Phiên bản: nguồn duy nhất là tệp VERSION ở gốc kho (X.Y.Z). versionCode = X*10000 + Y*100 + Z
// (Y, Z < 100), tăng dần theo phiên bản như Google Play yêu cầu. Không có biến môi trường ghi đè.
val versionFile = rootProject.file("../../VERSION")
val appVersionName: String = versionFile.readText().trim()
val versionParts: List<Int> = Regex("""^(\d+)\.(\d+)\.(\d+)$""").matchEntire(appVersionName)
    ?.groupValues?.drop(1)?.map { it.toInt() }
    ?: throw GradleException("Tệp VERSION (${versionFile.absolutePath}) phải có dạng X.Y.Z, đang là \"$appVersionName\"")
if (versionParts[1] > 99 || versionParts[2] > 99) throw GradleException("VERSION: số phụ và số vá phải < 100")
val appVersionCode = versionParts[0] * 10000 + versionParts[1] * 100 + versionParts[2]
// Số bản dựng (CI: github.run_number) và ngày dựng (YYYY-MM-DD, giờ UTC) — chỉ để hiển thị và nhắc cập nhật.
val appBuildNumber: String = System.getenv("LVHV_BUILD_NUMBER")?.takeIf { it.isNotBlank() } ?: "0"
val appBuildDate: String = System.getenv("LVHV_BUILD_DATE")?.takeIf { it.isNotBlank() }
    ?: LocalDate.now(ZoneOffset.UTC).toString()
if (!Regex("""^\d{4}-\d{2}-\d{2}$""").matches(appBuildDate)) throw GradleException("LVHV_BUILD_DATE phải có dạng YYYY-MM-DD")
if (!Regex("""^\d+$""").matches(appBuildNumber)) throw GradleException("LVHV_BUILD_NUMBER phải là số")

// libjnidispatch bản máy tính lấy từ jar JNA cho kiểm thử JVM (bản AAR chỉ có .so Android).
val jnaHost: Configuration by configurations.creating
val jnaHostDir: String = run {
    val os = System.getProperty("os.name").lowercase()
    val arch = System.getProperty("os.arch").lowercase()
    val a = if (arch == "aarch64" || arch == "arm64") "aarch64" else "x86-64"
    if (os.contains("mac")) "darwin-$a" else "linux-$a"
}
val extractJnaHost by tasks.registering(Copy::class) {
    from(provider { jnaHost.files.map { zipTree(it) } }) {
        include("com/sun/jna/$jnaHostDir/libjnidispatch.*")
        eachFile { path = name }
        includeEmptyDirs = false
    }
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
    into(layout.buildDirectory.dir("jna-host"))
}

android {
    namespace = "vn.clevix.lichviet"
    compileSdk = 35

    defaultConfig {
        applicationId = "vn.clevix.lichviet"
        minSdk = 26
        targetSdk = 35
        versionCode = appVersionCode
        versionName = appVersionName
        buildConfigField("String", "BUILD_NUMBER", "\"$appBuildNumber\"")
        buildConfigField("String", "BUILD_DATE", "\"$appBuildDate\"")
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

    testOptions {
        unitTests {
            // Robolectric (Compose UI, ảnh chụp màn hình) cần tài nguyên Android.
            isIncludeAndroidResources = true
            all { t ->
                // Kiểm thử JVM gọi lõi Rust thật: thư viện của máy dựng (target/release/liblvhv_ffi.so, do
                // build-rust.sh hoặc bước "host" của CI tạo) + libjnidispatch của JNA bản máy tính.
                t.systemProperty("jna.library.path", rootProject.file("../../target/release").absolutePath)
                t.systemProperty("jna.boot.library.path", layout.buildDirectory.dir("jna-host").get().asFile.absolutePath)
                t.systemProperty("jna.nosys", "true")
                // Ảnh chụp màn hình (Roborazzi) ghi thẳng vào apps/android/screenshots/.
                t.systemProperty("roborazzi.test.record", "true")
                t.systemProperty("lvhv.screenshots.dir", rootProject.file("screenshots").absolutePath)
                t.maxHeapSize = "3g"
                // Biến thể release không có ui-test-manifest: chạy mọi kiểm thử trừ kiểm thử giao diện Compose.
                if (t.name.contains("Release")) t.exclude("**/ui/people/**")
                // Mã UniFFI (android = true) dùng android.system.SystemCleaner khi API ≥ 33; dưới Robolectric lớp này
                // gọi jdk.internal.ref.CleanerFactory của JDK — cần mở gói cho kiểm thử.
                t.jvmArgs("--add-exports=java.base/jdk.internal.ref=ALL-UNNAMED", "--add-opens=java.base/jdk.internal.ref=ALL-UNNAMED")
                t.testLogging {
                    events("failed", "skipped")
                    exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
                    showStandardStreams = false
                }
                t.dependsOn("extractJnaHost")
            }
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
// -Plvhv.hostOnly=true (CI, job "ui"): chỉ có thư viện Rust bản máy tính cho kiểm thử JVM, không có .so Android.
if (!project.hasProperty("lvhv.hostOnly")) {
    tasks.named("preBuild") { dependsOn(checkRustArtifacts) }
}

// S11 / BM2-09: khóa phụ thuộc. CI chạy `:app:dependencies --write-locks` để sinh app/gradle.lockfile (đồ thị đã phân giải,
// gồm phụ thuộc bắc cầu) rồi quét lỗ hổng bằng osv-scanner. Khi tệp khóa được commit, mọi lần dựng phải khớp nó.
dependencyLocking {
    lockAllConfigurations()
}

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
    // Kiểm thử giao diện trên JVM (Robolectric) + ảnh chụp màn hình (Roborazzi 1.40.1, Robolectric 4.14.1).
    testImplementation("org.robolectric:robolectric:4.14.1")
    testImplementation(composeBom)
    testImplementation("androidx.compose.ui:ui-test-junit4")
    debugImplementation("androidx.compose.ui:ui-test-manifest")
    testImplementation("io.github.takahirom.roborazzi:roborazzi:1.40.1")
    testImplementation("io.github.takahirom.roborazzi:roborazzi-compose:1.40.1")
    jnaHost("net.java.dev.jna:jna:5.17.0")
}
