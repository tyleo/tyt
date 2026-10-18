# voxrender-bench-quest

voxrender's GPU benchmarks as a Meta Quest 3 OpenXR app. The app opens an
OpenXR session on Vulkan and builds wgpu from the session's instance and
device. Each frame, one multiview pass clears both eyes.

The crate builds only for Android. The repo README's Quest builds section
installs the tools.

## Build

The Gradle project lives in `android/`. Its `cargoNdk` task builds the crate's
release library, and AGP packages it with the Khronos OpenXR loader:

```sh
cd android
./gradlew assembleDebug
```

The first build downloads the JDK that `gradle/gradle-daemon-jvm.properties`
pins for Gradle's daemon.

## Run

With the headset connected over USB:

```sh
./gradlew installDebug
adb shell am start -n com.tyleo.voxrenderbench/android.app.NativeActivity
adb logcat -s voxrender-bench
```

The headset's library also lists the app under Unknown Sources. The session
starts only while someone wears the headset.

## Lint

A desktop build sees an empty crate, so lint for Android:

```sh
cargo clippy -p voxrender-bench-quest --target aarch64-linux-android -- -D warnings
```
