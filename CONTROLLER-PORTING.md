# Adapting the bridge for another controller

For developers and AI agents whose user has a controller other than the DDJ-FLX4. Start with that controller's documentation, not a search-and-replace of FLX4 note numbers.

**This is a source-porting guide, not a list of supported controllers.** JSON profiles change pad actions; they do not describe MIDI hardware. `--input` and `--output` only select ports. The normal bridge sends FLX4-specific mode/LED messages and keepalive SysEx: **do not point it at another device unchanged**.

Read the [AP contract](DEVELOPMENT.md#runtime-and-wire-contract) and [current mapping](MAPPING.md). Keep the AP-facing behaviour where appropriate; replace the device-specific decoding and output. No firmware changes are required by this approach, but another device's usable protocol must first be established.

## 1. Establish the target and scope

Use information already supplied; ask only for missing details that affect the implementation:

- Exact manufacturer/model, hardware revision, relevant firmware version and operating system.
- Operating mode and connection: USB MIDI, DIN MIDI through an interface, HID or a proprietary protocol. A similar appearance or model name does not establish compatibility.
- Desired controls and deck layout: two strips switching between four decks, four dedicated strips, a pad-only device, etc. Do not assign fixed track/clip/sampler roles to deck numbers.
- AP version/API availability and any existing mapping the user wants preserved. Read user mapping files only with permission; do not publish their settings or assume they prove an undocumented API.
- Whether this is a separate fork or an addition that must retain FLX4 support. Do not silently replace the existing device's defaults.

Research and code inspection come first. Hardware capture, live operation, tests and publication require explicit authorization for that work. Do not change AP settings, install drivers, switch persistent device modes or write firmware as a side effect of adapting code.

## 2. Collect evidence before assigning controls

Find the exact model's **official user manual and MIDI implementation/programmer's reference**. Read sections on modes, channels, message formats, LEDs, initialization and shutdown. If needed, inspect a maintained implementation such as a Mixxx mapping for that exact device; read both its routing configuration and callbacks.

For each fact, record a source URL plus page/section or repository revision and function. Fetch and read the source: search snippets, adjacent models and AI recollections are not protocol evidence. Check the original PDF table when extraction loses column alignment. Keep **documented**, **observed with permission**, **inferred** and **unknown** separate.

Build a small inventory before editing:

| Physical control / mode | Device → host message | Meaning / decoding | Host → device feedback | Evidence / status |
|---|---|---|---|---|
| Button, shifted button, pad, knob, encoder, fader… | Port; status/channel; note/CC or other format; values | Press/release, absolute position or relative delta | Address, value range, colour/blink/mode effect, or none | URL + page/function; observations and gaps |

Include these commonly missed details:

- **Numbering:** decimal versus hex; manuals often number channels 1–16 while MIDI status nibbles/code use 0–15. Define direction explicitly as device→host or host→device rather than trusting an ambiguous “MIDI IN” heading.
- **Buttons:** Note On/Off, zero-velocity Note On, release velocity, separate shifted notes/channels, and firmware-generated short/long-press events. Do not invent a hold timer if the firmware already classifies the gesture.
- **Continuous controls:** 7-bit CC, paired 14-bit CC, pitch bend, NRPN or another encoding; message order, endpoints, direction and centre detent. The current decoder does not implement all these formats.
- **Encoders/jogs:** relative encoding, acceleration, touch messages and mode dependence. Different encoders on one device can use different formats.
- **Output:** LED/meter addresses need not match input addresses. Record whether output changes the hardware mode or subsequent pad messages, and whether messages are echoed back.
- **Initialization:** required handshake/keepalive, interval, any position reports it produces, and safe shutdown. An unknown output protocol is a reason to omit hardware LEDs/meters, not to try the FLX4's SysEx. Keep AP state feedback for pickup and routing.

Reference implementations can supply protocol evidence, but their application actions are not AP routes. Respect their licenses before copying code; this project's MIT license does not relicense third-party code.

### How this was done for FLX4

The bridge combines a device protocol reference with AP's own control semantics, rather than translating another application's mapping wholesale:

| Evidence / design step | Result in this bridge |
|---|---|
| [Official FLX4 MIDI list](https://downloads.support.alphatheta.com/software_info/dj-controllers/DDJ-FLX4/DDJ-FLX4_MIDI_message_List_E1.pdf), pp.1–2, “DECK”, “MIXER”, “BROWSE” | Button messages, 14-bit absolute controls and a separately decoded relative browse encoder. |
| Same MIDI list, p.2 “BEAT SYNC” and p.5 footnote 3 | Preserve native short/long SYNC messages, which are emitted on finger release; do not delay LOAD to implement a competing gesture. |
| Same MIDI list, pp.3–5 “PERFORMANCE PAD” | Decode pad index, side and shifted channel; normalize physical identity so changing a bank/Shift does not lose the release. |
| [Mixxx FLX4 source](https://raw.githubusercontent.com/mixxxdj/mixxx/main/res/controllers/Pioneer-DDJ-FLX4-script.js), `sendKeepAlive`, `init`, `padModeKeyPressed`, `vuMeterUpdate` | Protocol reference for keepalive bytes/200 ms scheduling, mode feedback and meters—not a Mixxx runtime dependency or authority for AP actions. |
| [AP-facing implementation](src/mapping.rs), `Player::route`; [wire contract](DEVELOPMENT.md#runtime-and-wire-contract) | Route by loaded player type, with tempo scaling isolated from normalized mixer/pickup state. |
| [Deck selection and repeat design](MAPPING.md#deck-selection) | Shift+LOAD selects a logical deck; bank and repeat state belong to that deck. These are bridge design choices, not universal controller gestures. |

This establishes where the implementation came from, **not hardware validation**. See [release status](STATUS.md) for what has actually been built or exercised.

## 3. Map gestures to supported AP actions

Write down `physical gesture → decoded event → logical deck/channel → AP action → feedback` for each proposed control. Start with transport, browser/load and mixer; add performance banks only where the controller and AP support them.

- For each AP input, establish its exact route, argument type/range, units, and trigger/toggle/momentary semantics. Establish feedback separately: a feedback address is not proof of a writable input.
- Preserve `PlayerType` routing. Track, arrangement, clip and sampler controls are not interchangeable just because their names resemble one another. Unknown/empty players must not acquire guessed deck actions.
- If strips serve multiple decks, choose an unambiguous selection gesture and visible indication. Keep deck selection, bank selection and shifted pad actions distinct. Selection must not accidentally load, stop or change tempo master.
- Prefer explicit on/off operations when no confirmed state is available. Do not invent cue occupancy, sampler step state or LED indicators from stale assumptions.
- Leave unsupported actions unassigned and explain why. A jog can send perfectly documented MIDI while AP lacks the necessary input operation. Waveform seeking is not a substitute for scratching or moving a held slip-loop window; “signed” alone does not establish fractional precision.

For AP source references and this bridge's explicit bipolar tempo requirement, use [DEVELOPMENT.md](DEVELOPMENT.md#references). If the installed AP contract is unclear, record that blocker instead of guessing from a similar control or bypassing pickup.

## 4. Change the hardware boundary, not the safety model

There is no universal controller-adapter interface today. Hardware assumptions are spread across these files:

| Location | What to inspect or adapt |
|---|---|
| [`src/main.rs`](src/main.rs): `Options`, `main`, `run` | Port defaults and selection, connection count, startup text, 200 ms keepalive schedule. The normal path currently requires one MIDI input and one output. |
| [`src/mapping.rs`](src/mapping.rs): `MODES`, `KEEPALIVE`, `deck_button`, `key`, `analog_address` | Hardware constants, button/analog assignments and physical-key aliases. Keep the AP-specific `Player::route` logic separate from replacements. |
| [`src/bridge.rs`](src/bridge.rs): `input`, `cc` | Three-byte Note/CC dispatch; hardcoded channels, Shift/load/pad handling, relative browse decoding and paired-CC assembly. Changing constants alone is insufficient. |
| Same file: `start`, `keepalive`, `mode_feedback`, `led`, `refresh_leds`, `cleanup` | Every MIDI output path, including startup and exit—not just the recurring SysEx. |
| Same file: `selected`, `banks`, `players`, `switch_deck`, `show_selection` | Two physical sides, four logical decks, 1↔3 / 2↔4 switching and side labels. A four-strip or single-strip controller needs deliberate routing changes. |
| [`src/profile.rs`](src/profile.rs): defaults and `Profile::load` | Eight banks with eight action/null slots each. Fewer physical pads can leave slots unused or need paging; a different schema requires an explicit compatibility decision. |

Use a small, isolated hardware implementation. If both controllers must coexist, introduce only the device selection/adapter boundary needed to keep their decoders and output protocols separate. Do not build a speculative plugin framework. HID-only devices require a different transport; input-only MIDI devices require optional output throughout startup, feedback and cleanup. Neither is solved by changing `--input`.

Preserve these invariants while adapting:

1. **Physical identity survives modifiers.** Press under Shift, release without it—or change banks/decks while holding—and release the original resolved action. Do not merge distinct physical buttons into one key. Multiple input ports also need port identity; the current callback carries bytes from only one input.
2. **Decode before scaling.** A 7-bit absolute control uses its documented range (commonly 0–127), not the FLX4's 0–16383 divisor. Adapt pairing, inversion, centre handling and pickup tolerance to the actual resolution. Do not apply absolute-control decoding to relative encoders.
3. **Normalize once.** Keep internal absolute position/pickup normalized to 0–1. AP tempo is bipolar −1..+1 in both directions; mixer values remain 0–1. The FLX4-specific centre codes 8191/8192 are not universal.
4. **Pickup needs real feedback and movement.** Startup/position-query reports establish a baseline, not permission to jump a control. Re-arm when changing destination or control layer; do not combine old MSBs or crossing history with a newly selected layer. Missing targets stay blocked, not defaulted to zero. Pickup is not continuous arbitration of external edits.
5. **State belongs to its destination.** Retain per-deck banks/timing, confirmed-feedback headphone CUE toggles, held release destinations and last-held-repeat priority. A fixed straight repeat shortcut must not erase latched dotted/triplet selection.
6. **Failure must remain bounded.** Keep queue-overflow handling, bounded OSC processing, feedback source filtering and graceful release attempts. Claim only best-effort cleanup over UDP, not guaranteed recovery from crashes or device loss.

## 5. Resolve remaining unknowns only with permission

Do not use the normal bridge as a discovery tool on an unsupported device. Do not send guessed SysEx, sweep LED addresses or change firmware to see what happens. A missing handshake specification is a blocker, not permission to experiment.

If the user explicitly authorizes MIDI capture, these existing commands separate enumeration from input-only monitoring:

```sh
cargo run --release --locked -- --list
cargo run --release --locked -- --monitor --input "#0"
```

`#0` is only an example: choose the input index from the list. `--monitor` opens MIDI input but sends no MIDI output, keepalive or OSC; omitting that flag starts the live bridge. Normal input/output indices are independent. Some devices require initialization before reporting controls, so a silent passive capture is inconclusive.

Within the approved capture scope, record the selected device/mode, raw bytes and the exact gesture—press/release, Shift transitions, both encoder directions or a fader endpoint. Capture one unknown at a time. Do not turn permission for passive capture into permission for output experiments or a full performance session.

Without testing permission, use source inspection and compilation only. `cargo build --release --locked` does not start the bridge; `cargo run` does. Do not add/run tests, hardware probes or manual QA by default. If an essential fact cannot be established, stop with the sources consulted, unresolved question and smallest additional evidence needed.

## 6. Deliver a clearly scoped port

- Provide the controller-specific mapping and source notes, plus supported/unsupported features and unresolved protocol questions.
- Update startup/port names, setup instructions and printable controls to match the actual device. Keep AP settings changes as user instructions unless separately authorized.
- If retaining FLX4 support, leave its selection path and behaviour intact. If making a separately named fork, align Cargo metadata, executable/launcher names, workflow, packaging and license notices; do not ship it under a misleading device name.
- Record **source-reviewed**, **compiled**, **hardware-observed** and **end-to-end validated** separately, including device/mode/AP version where applicable. Do not present proposed checks as completed work or inherit FLX4 validation claims for another device.
- Include the new documentation in packages and update links. No release publication, firmware changes or machine-specific deployment paths without authorization.

The deliverable is a small, explained mapping with explicit gaps—not a claim that every control works because the binary compiled.
