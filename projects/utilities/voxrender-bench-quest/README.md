# voxrender-bench-quest

voxrender's GPU benchmarks as a Meta Quest 3 OpenXR app. The app opens an
OpenXR session on Vulkan and builds wgpu from the session's instance and
device. Each frame, one multiview pass clears both eyes, and B2's timer
records its next timed pass.

The app runs B0's device probe and logs each result on a line that starts
with `b0`. Before the session opens, the app runs B1 and logs its report on
lines that start with `b1`. Once the session runs, the app logs each window
of B2's times on `b2` lines. The voxrender-wgpu
[research](../../../doc/plan/open/voxrender-wgpu/reference/research.md#7-next-phase-benchmarks-and-prototypes-in-order)
sets what each benchmark asks.

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
adb shell setprop debug.oculus.gpuLevel 4
adb shell am start -n com.tyleo.voxrenderbench/android.app.NativeActivity
adb logcat -s voxrender-bench
```

The property pins the GPU at level 4, the level the benchmarks' thresholds
assume, until the headset reboots. `adb logcat -s VrApi` shows the level as
the second number after `GPU=`.

The headset's library also lists the app under Unknown Sources. B1 takes
about 22 seconds, and the session opens after it. The session starts only
while someone wears the headset. The app logs B0's visibility mask once the
session has focus. The app logs the first window of frame times about ten
seconds after the session starts, then another every seven seconds. A window
of B2's times takes 120 timed passes per kernel after a warm-up of 300, and
the kernels slow the frame rate while they run.

## Lint

A desktop build sees an empty crate, so lint for Android:

```sh
cargo clippy -p voxrender-bench-quest --target aarch64-linux-android -- -D warnings
```
