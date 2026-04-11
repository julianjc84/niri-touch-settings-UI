# niri-touch-settings-UI

A GTK4 / libadwaita preferences app for configuring the touchscreen and
touchpad gesture subsystems in the niri Wayland compositor. Provides a
live-editing UI for:

- Touchscreen / touchpad device settings (accel, click method, natural
  scroll, disable-while-typing, etc.)
- Gesture recognition tuning (swipe / pinch / rotate trigger distances,
  dominance ratios, progress scaling)
- Dynamic gesture bind table — add, remove, enable/disable, and retarget
  individual gesture binds without editing KDL by hand

Each change is written back immediately and niri is signalled to reload
its config (`niri msg action load-config-file`).

## How it writes config

The app doesn't edit your main `config.kdl`. Instead, it writes two
include files that niri merges into the main config:

- **`~/.config/niri/touchscreen-gestures.kdl`** — replaces
  `input { touchscreen { … } }` plus a top-level `binds {}` block
  containing every `TouchSwipe`, `TouchPinch`, `TouchRotate`, and
  `TouchEdge` bind the tool knows about.
- **`~/.config/niri/touchpad-gestures.kdl`** — replaces
  `input { touchpad { … } }` plus a `binds {}` block with `TouchpadSwipe`
  entries.

On first run the tool appends include lines to your main `config.kdl`
if they aren't already present:

```kdl
include "touchscreen-gestures.kdl" optional=true
include "touchpad-gestures.kdl"    optional=true
```

Hand-edited binds in your main config still work — niri merges the
two sources, and include files override matching keys.

## Requirements: niri with property-form triggers

niri's gesture trigger syntax changed from hardcoded enum-form names
(`TouchSwipe3Up`, `TouchEdgeTop:Left`) to property-form KDL nodes
(`TouchSwipe fingers=3 direction="up"`,
`TouchEdge edge="top" zone="left"`) in niri commit `558adc5f`
(*refactor: parameterize Touch/Touchpad triggers with KDL properties*).
The old enum-form no longer parses.

**This tool writes property-form only.** It requires a niri build with
that refactor or later — i.e. the `feat/configurable-touch-gestures`
branch of [julianjc84/niri](https://github.com/julianjc84/niri), or any
upstream niri that has landed equivalent work.

If you're running an older niri, writes from this tool will be rejected
at parse time.

## Build / install

```bash
./install_niri_touch_settings_UI.sh
```

Builds in debug, stops any running instance, and installs the binary to
`/usr/local/bin/niri-touch-settings`. Launch it as
`niri-touch-settings`.

## What you can tune

### Touchscreen — Detection tab

Writes to `input { touchscreen { gestures { … } } }`:

| knob | what it does |
|---|---|
| `swipe-trigger-distance` | Pixels of centroid movement before a swipe commits |
| `edge-start-distance` | Width of the screen-edge start zone (px) |
| `swipe-multi-finger-scale` | Extra swipe threshold per finger above 3 — biases ambiguous high-finger gestures toward pinch/rotate |
| `pinch-trigger-distance` | Minimum spread change before a pinch commits |
| `pinch-dominance-ratio` | Spread must exceed swipe distance by this factor (higher = stricter pinch) |
| `pinch-sensitivity` | Multiplier for pinch deltas sent over IPC |
| `rotation-trigger-angle` | Minimum rotation in degrees before a rotate commits |
| `rotation-dominance-ratio` | Rotation arc must dominate swipe+spread by this factor (higher = stricter rotate) |
| `rotation-progress-angle` | Degrees of rotation mapped to IPC progress = 1.0 |
| `swipe-progress-distance` | Screen pixels mapped to IPC progress = 1.0 |

### Touchscreen — Gesture Binds tab

An add-form with a **family picker** (Swipe / Pinch / Rotate / Edge)
that morphs the rest of the form to show the right properties:

- **Swipe / Pinch / Rotate**: Fingers (3–10) + Direction (Up/Down/Left/Right,
  In/Out, or CW/CCW)
- **Edge**: Edge (Left/Right/Top/Bottom) + Zone (Full, or one of three
  axis-appropriate thirds — Top/Bottom edges take Left/Center/Right,
  Left/Right edges take Top/Center/Bottom)

Each active bind gets an expandable row with Action, Sensitivity,
Natural Scroll, Tag (IPC event identifier for external tools like
`niri-tag-sidebar`), and an enable/delete pair in the suffix.

### Touchpad — Device tab

Full libinput device settings (tap, dwt, dwtp, drag, drag-lock, click
method, tap button map, accel speed + profile, scroll method + factor,
left-handed, middle-emulation, scroll-button, disabled-on-external-mouse)
plus the two gesture knobs niri exposes for touchpad (`swipe-trigger-distance`,
`swipe-progress-distance`).

### Touchpad — Gesture Binds tab

Same bind-row UI as touchscreen, but the only family is `TouchpadSwipe`
— the picker shows Fingers + Direction directly. 3+ finger only;
libinput handles 2-finger scroll and pinch itself.

## Status

Working tool, in regular use. The project tracks niri's gesture subsystem
closely, so expect churn when niri changes its config schema.
