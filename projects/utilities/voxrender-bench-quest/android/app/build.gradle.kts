plugins {
    id("com.android.application")
}

android {
    namespace = "com.tyleo.voxrenderbench"
    compileSdk = 34
    ndkVersion = "30.0.16248370"

    defaultConfig {
        applicationId = "com.tyleo.voxrenderbench"
        minSdk = 32
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"

        ndk {
            abiFilters += "arm64-v8a"
        }
    }
}

dependencies {
    implementation("org.khronos.openxr:openxr_loader_for_android:1.1.63")
}

/** Builds the crate's release library into `outputDir`'s ABI folders. */
abstract class CargoNdk : Exec() {
    @get:OutputDirectory
    abstract val outputDir: DirectoryProperty
}

val cargoNdk = tasks.register<CargoNdk>("cargoNdk") {
    val crateDir = layout.projectDirectory.dir("../..")
    val ndkDirectory = androidComponents.sdkComponents.ndkDirectory

    workingDir = crateDir.asFile
    executable = "cargo"
    inputs.dir(crateDir.dir("src"))
    inputs.file(crateDir.file("Cargo.toml"))

    doFirst {
        environment("ANDROID_NDK_HOME", ndkDirectory.get().asFile.absolutePath)
        args(
            "ndk",
            "-t", "arm64-v8a",
            "-P", "32",
            "-o", outputDir.get().asFile.absolutePath,
            "build", "--release",
        )
    }
}

androidComponents {
    onVariants { variant ->
        variant.sources.jniLibs?.addGeneratedSourceDirectory(cargoNdk, CargoNdk::outputDir)
    }
}
