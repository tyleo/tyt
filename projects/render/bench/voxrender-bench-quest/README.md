# voxrender-bench-quest

voxrender's GPU benchmarks as a Meta Quest 3 OpenXR app. The app opens an
OpenXR session on Vulkan and builds wgpu from the session's instance and
device. Each frame, one multiview pass clears both eyes, and B2's timer
records its next timed pass.

The app runs B0's device probe and logs each result on a line that starts
with `b0`. Before the session opens, the app runs B1 and logs its report on
lines that start with `b1`. Once the session runs, the app logs each window
of B2's times on `b2` lines. The voxrender-wgpu
[research](../../../../doc/plan/open/voxrender-wgpu/reference/research.md#7-next-phase-benchmarks-and-prototypes-in-order)
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
the second number after `GPU=` and the clock after the slash that follows.
The clock moves within a level every few seconds, so each window of B2's
times starts with a `b2` line holding each clock's share of the window's
time.

The headset's library also lists the app under Unknown Sources. B1 takes
about 22 seconds, and the session opens after it. The session starts only
while the headset senses a face. The app logs B0's visibility mask once the
session has focus. The app logs the first window of frame times about ten
seconds after the session starts, then another every seven seconds. A window
of B2's times takes 120 timed passes per kernel after a warm-up of 300, and
the kernels slow the frame rate while they run. The kernels take turns of 20
passes in a row, and the first 4 of a turn go untimed. A window's table
gives each kernel's least, p10, p50, and p99 times. The compositor
interrupts some passes for about 0.75 ms, so the p10 compares two kernels
best.

Two more properties shorten a run. `debug.voxrenderbench.b1 false` skips
B1. `debug.voxrenderbench.kernels` has B2 time only the kernels whose labels
hold the property's text and log no closing lines. The device's shell takes
the whole command in quotes because the text can hold spaces:

```sh
adb shell setprop debug.voxrenderbench.b1 false
adb shell 'setprop debug.voxrenderbench.kernels "box raster"'
```

`setprop debug.voxrenderbench.b1 true` and
`'setprop debug.voxrenderbench.kernels ""'` return the app to the full run.

### Scale

`debug.voxrenderbench.scale` holds a part over 0 and at most 1. The app
draws that part of the eyes' recommended size along each side. Unset, the app
draws the recommended size of 1680 by 1760 an eye on Quest 3. At `0.8` it
draws 1344 by 1408, and the runtime stretches the image to the display.

### Off the face

Two commands let the session run with the headset on a desk. The first
pauses Guardian, which otherwise blocks the launch. The second fakes the
proximity sensor:

```sh
adb shell setprop debug.oculus.guardian_pause 1
adb shell am broadcast -a com.oculus.vrpowermanager.prox_close
```

Undo both before wearing the headset again:

```sh
adb shell setprop debug.oculus.guardian_pause 0
adb shell am broadcast -a com.oculus.vrpowermanager.automation_disable
```

A reboot also clears the property.

## View

`debug.voxrenderbench.view` holds the start of a bench scene's name. The app
then skips its benchmarks and shows the scene around the wearer. The box
raster draws the scene for both eyes each frame:

```sh
adb shell setprop debug.voxrenderbench.view S1
adb shell am start -n com.tyleo.voxrenderbench/android.app.NativeActivity
```

The wearer starts at the scene's first view. The app logs the view's time on
the CPU every 600 frames on a `view` line beside B0's GPU frame times.
`'setprop debug.voxrenderbench.view ""'` returns the app to its benchmarks.

The view shades as the contract's opaque surface under the scene's first sun
and its hemispheres. The GPU fills the sun's shadows at start. A face's
mirror ray traces the scene's opaque faces through a ray query. The face it
meets shades flat with no shadow. The rougher the face, the nearer what it
mirrors has to lie to show. A model's emission blooms as vxl's review
renders bloom it. Glass and water tint what lies behind them.

## Models

`debug.voxrenderbench.models true` has the app skip its benchmarks and show
a gallery of Voxel Json models one at a time. The gallery draws each model as
the view draws a scene. The gallery holds every `.voxj` and `.voxjz` document
under `models` in the app's files. The models show in the order of their
paths. `adb push` cannot create folders there, so the models go through a
temporary folder:

```sh
adb push <models-dir> /data/local/tmp/models
adb shell cp -r /data/local/tmp/models /sdcard/Android/data/com.tyleo.voxrenderbench/files/
adb shell rm -r /data/local/tmp/models
adb shell setprop debug.voxrenderbench.models true
adb shell am start -n com.tyleo.voxrenderbench/android.app.NativeActivity
```

A model lands on a table before the wearer and spans a meter. The model
sits under vxl's studio rig as vxl's hero view places the rig. The Touch
controllers steer:

| Control | Effect |
| --- | --- |
| A or X | Shows the next model |
| B or Y | Shows the model before |
| Left stick | Walks the way the head faces |
| Right stick to a side | Turns the wearer 30 degrees |
| Right stick ahead or back | Rises or sinks |
| Right trigger | Grows the model around the point a meter ahead |
| Left trigger | Shrinks the model |
| A press of either stick | Switches between the table and life size |

At life size the wearer stands on the model's floor 1.5 m before its
front. While a model loads on a thread of its own, the model before it
stays in view under a dimmed sky. The app logs each model on a `models`
line with the time its load took.

Two more properties steer a run:

1. `debug.voxrenderbench.model` starts the gallery at the first model whose
   path holds the property's text
2. `debug.voxrenderbench.tour` steps the gallery to the next model once each
   has shown for the property's count of seconds

`setprop debug.voxrenderbench.models false` returns the app to its
benchmarks.

### Test models

Models that test the renderer stay out of the gallery's `models`. They go
under `tests` in the app's files. `debug.voxrenderbench.tests true` shows
that folder's models in the gallery's place. The bench's `test-models` folder
holds their sources:

| Source | Tests |
| --- | --- |
| `pbr-chart.ts` | Five steps of roughness across metal slabs and gold, steel, and plastic spheres |

vxl builds a source into the document to push:

```sh
vxl sdf-doc build pbr-chart.ts
vxl sdf-doc voxelize pbr-chart.sdfj --voxel-size 0.006 --format zip
adb shell mkdir -p /sdcard/Android/data/com.tyleo.voxrenderbench/files/tests
adb push pbr-chart.voxjz /data/local/tmp/
adb shell mv /data/local/tmp/pbr-chart.voxjz /sdcard/Android/data/com.tyleo.voxrenderbench/files/tests/
adb shell setprop debug.voxrenderbench.tests true
```

## Profile

The profile runs six of B2's kernels in turn, each alone for 10 seconds.
`ovrgpuprofiler` samples its counters each second, so each sample sees one
kernel. The app skips B1 and logs each kernel as it starts on `profile`
lines. Each line also gives the batches the kernel before it recorded and
their seconds. With `debug.voxrenderbench.kernels` set, the profile runs the
kernels whose labels hold that text in place of the six. Each of their
batches records one pass:

```sh
adb shell setprop debug.voxrenderbench.profile true
adb shell am start -n com.tyleo.voxrenderbench/android.app.NativeActivity
adb shell ovrgpuprofiler -m
adb shell ovrgpuprofiler -r="4,17,18,19,20,23,55"
```

`-m` lists the counters by id. `-r` prints the chosen ones each second.
The profile repeats until the app stops. `setprop
debug.voxrenderbench.profile false` returns the app to B1 and B2.

## Pipeline statistics

`debug.voxrenderbench.statistics true` has the app log what the driver
reports for each of the view's pipelines on `pipeline statistics` lines
before its session opens:

```sh
adb shell setprop debug.voxrenderbench.statistics true
adb shell am start -n com.tyleo.voxrenderbench/android.app.NativeActivity
adb logcat -s voxrender-bench | grep "pipeline statistics"
```

Each line gives one stage of one pipeline. On Quest the driver counts the
stage's instructions and its reads, its registers, and the share of its
fibers that run at once. The line of a fragment stage reads `Mesh Shader`.
`setprop debug.voxrenderbench.statistics false` turns the lines off.

## Lint

A desktop build sees an empty crate, so lint for Android:

```sh
cargo clippy -p voxrender-bench-quest --target aarch64-linux-android -- -D warnings
```
