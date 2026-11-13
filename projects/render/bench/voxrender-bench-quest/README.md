# voxrender-bench-quest

voxrender-bench's headset view and model gallery as a Meta Quest 3 OpenXR
app. The app opens an OpenXR session on Vulkan and builds wgpu from the
session's instance and device. Each frame, the box raster draws a bench
scene or a gallery's model for both eyes.

The app runs B0's device probe and logs each result on a line that starts
with `b0`. The voxrender-wgpu
[research](../../../../doc/plan/open/voxrender-wgpu/reference/research.md#7-next-phase-benchmarks-and-prototypes-in-order)
sets what B0 asks.

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

The property pins the GPU at level 4, the level the plan's thresholds
assume, until the headset reboots. `adb logcat -s VrApi` shows the level as
the second number after `GPU=` and the clock after the slash that follows.

The headset's library also lists the app under Unknown Sources. The session
starts only while the headset senses a face. The app logs B0's visibility
mask once the session has focus. The app logs the first window of frame
times about ten seconds after the session starts, then another every seven
seconds.

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

`debug.voxrenderbench.models false` has the app show the bench scene S1
around the wearer in place of its gallery. The box raster draws the scene
for both eyes each frame. `debug.voxrenderbench.view` holds the start of
another bench scene's name:

```sh
adb shell setprop debug.voxrenderbench.models false
adb shell setprop debug.voxrenderbench.view S2
adb shell am start -n com.tyleo.voxrenderbench/android.app.NativeActivity
```

The wearer starts at the scene's first view. Every 600 frames the app logs
the view's mean and p99 time on the CPU on a `view` line beside B0's GPU
frame times. `'setprop debug.voxrenderbench.view ""'` returns the app to S1.

`debug.voxrenderbench.walk` holds a speed in meters a second. The app then
walks the wearer 60 m ahead from the view and back at that speed. The head
bobs with each step and looks 60 degrees to each side. Two runs draw the
same views because the walk steps by frames drawn:

```sh
adb shell setprop debug.voxrenderbench.walk 1.4
```

`'setprop debug.voxrenderbench.walk ""'` stops the walk.

The view shades as the contract's opaque surface under the scene's first sun
and its hemispheres. The GPU fills the sun's shadows at start. A face's
mirror ray traces the scene's opaque faces through a ray query. The face it
meets shades flat with no shadow. The rougher the face, the nearer what it
mirrors has to lie to show. A model's emission blooms as vxl's review
renders bloom it. Glass and water tint what lies behind them and the
sunlight that crosses them.

## Models

With no property set, the app shows a gallery of Voxel Json models one at
a time. The gallery draws each model as the view draws a scene. The gallery
holds every `.voxj` and `.voxjz` document under `models` in the app's
files. The app stops at start while that folder holds none. The models show
in the order of their paths. The gallery opens with
`valley-5km/valley-5km.voxjz` when the folder holds it. The voxrender-wgpu
[benchmarks](../../../../doc/plan/open/voxrender-wgpu/reference/benchmarks.md#the-5-km-valley-2026-11-13)
give the commands that make that copy of the valley. `adb push` cannot
create folders there, so the models go through a temporary folder:

```sh
adb push <models-dir> /data/local/tmp/models
adb shell cp -r /data/local/tmp/models /sdcard/Android/data/com.tyleo.voxrenderbench/files/
adb shell rm -r /data/local/tmp/models
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

### Test models

Models that test the renderer stay out of the gallery's `models`. They go
under `tests` in the app's files. `debug.voxrenderbench.tests true` shows
that folder's models in the gallery's place. The bench's `test-models` folder
holds their sources:

| Source | Tests |
| --- | --- |
| `glass-chart.ts` | Panes that tint their shadows, a tank of water, a block of glass, a lamp in a glass case, and a wall's shadow across a glass slab |
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

## GPU counters

`ovrgpuprofiler` samples the GPU's counters each second while a view or a
gallery runs:

```sh
adb shell ovrgpuprofiler -m
adb shell ovrgpuprofiler -r="4,17,18,19,20,23,55"
```

`-m` lists the counters by id. `-r` prints the chosen ones each second. The
counts include the compositor's work.

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
