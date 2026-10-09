# FLX4 Pilot

[Source repository](https://github.com/Thevetat/flx4-pilot)

Use a **Pioneer DJ DDJ-FLX4** to control up to four decks in **[AUTO/PILOT](https://autopilot.audio/)**. FLX4 Pilot reads the controller and sends commands to AP. It also receives AP's control positions and sends lights/meters back to the controller.

**0.3.3 · Windows x64 preview.** No firmware changes, virtual MIDI cable or programming knowledge needed. Hardware behaviour is not validated. macOS/Linux build instructions are available for developers.

[Setup](#setup-windows) · [Deck selection](#choose-which-deck-you-control) · [Controls](#use-the-pads-and-controls) · [Troubleshooting](#if-something-does-not-work) · [AI/developer reference](#technical-reference-for-developers-and-ai-agents)

## Before you start

You need a connected FLX4, AUTO/PILOT and the Windows ZIP containing this bridge. Keep your usual FLX4 audio setup—the bridge changes **control**, not sound routing.

**Tempo faders require an AP build with bipolar TempoSlider control and feedback:** −1 minimum, 0 native tempo, +1 maximum. Legacy 0–1 tempo APIs are incompatible. If AP does not provide TempoSlider feedback, the bridge leaves tempo inactive; other supported controls can still work. There is no automatic API-version detection.

Jog wheels, scratching, repeat scrubbing, crossfader and FX are **not mapped**. This is not a complete replacement for every FLX4 function.

## Setup (Windows)

### 1. Extract the bridge

Right-click `flx4-pilot-0.3.3-x86_64-pc-windows-msvc.zip` → **Extract All**. Keep the extracted files together. Do not launch from inside the ZIP.

Connect the FLX4. Close other DJ applications that use its MIDI ports. If another FLX4 Pilot window is running, stop it with **Ctrl+C** before starting a replacement.

### 2. Let the bridge handle the controller

In AUTO/PILOT:

- Open **Settings → MIDI → INPUTS** and **disable DDJ-FLX4** there. The bridge—not AP's direct MIDI mapping—will read the controller. Leaving both enabled causes duplicate control.
- Leave audio settings and saved mapping files alone.
- Use **Perform** mode and **four-deck view** for decks 3/4. The bridge does not change AP's view for you.

### 3. Enable OSC input

OSC is how the bridge sends commands to AP. In **Settings → OSC**:

| Setting | Value |
|---|---|
| Input | **Enabled** |
| Listen port | **9090** |

### 4. Enable feedback

Feedback tells the bridge which deck type is loaded and where AP's controls are positioned. In **Settings → OSC FEEDBACK**:

| Setting | Value |
|---|---|
| Destination IP | **127.0.0.1** |
| Destination port | **9096** |
| Update rate | **30 Hz** |
| Subscriptions | **Toggle All → all checked**, including TempoSlider |

Make sure the boxes end up **checked**; pressing Toggle All again may turn them off. Extra feedback values are ignored by the bridge.

`127.0.0.1` means **this computer**. AP and the bridge run on the same machine; you do not need another device or router port forwarding.

### 5. Enable controller browsing

Set **Settings → GENERAL → BROWSER MODE → CONTROL MODE** and keep the Browser visible. This lets the FLX4's browse knob navigate AP's library.

### 6. Start FLX4 Pilot

Double-click **Start FLX4 Pilot.cmd** in the extracted folder. Keep its console window open while using the controller. It shows the selected decks, loaded content types, pad banks and repeat timing.

Startup selection is **LEFT: Deck 1 / RIGHT: Deck 2**. **Ctrl+C** in that window stops the bridge and attempts to release held controls. After unplugging the FLX4 or restarting AP, stop and restart the bridge too.

To return to AP's direct MIDI mapping: stop the bridge, then re-enable DDJ-FLX4 under AP's MIDI inputs.

## Choose which deck you control

| Physical side | Starts on | How to switch | Available decks |
|---|---|---|---|
| Left | Deck 1 | Hold **left Shift**, tap **left LOAD**, release Shift | **1 ↔ 3** |
| Right | Deck 2 | Hold **right Shift**, tap **right LOAD**, release Shift | **2 ↔ 4** |

The sides switch independently. Left controls 1 or 3; right controls 2 or 4. All four decks can keep playing. A deck can contain a track, clip, sampler or arrangement; its number does not determine its role.

**Example:** after switching left to Deck 3, its supported transport controls and pads operate Deck 3. Its fader, trim, EQ, filter and headphone CUE operate **main mixer channel 3**. Its lights/meter follow that selection. Deck 1 keeps playing and retains its settings.

- **Read the console/title before using controls.** Mode-button lights show the pad bank, not the deck number.
- **Selecting a deck does not load, stop, sync or make it master.** Release Shift and press LOAD normally to load the Browser selection into that deck.
- **Each deck remembers its pad bank and repeat timing.** Switching back recalls them. A button already held still releases its action on the original deck.
- **Knobs/faders may wait for pickup** after switching. See below.
- **Restart selects 1/2.** AP must be in four-deck view to use 3/4.

**Deck vs bank:** Shift+LOAD chooses a **deck**. Shift+HOT CUE/PAD FX/BEAT JUMP/SAMPLER chooses a **pad bank on that deck**. Browser navigation and headphone MIX are shared controls.

## Why a knob or fader may not move AP immediately

This is **pickup**, which prevents jumps when two physical channel strips control four software decks.

If AP's channel volume is low but the hardware fader is high, the bridge waits. Move the fader to AP's displayed position; once it reaches or crosses that position, it takes control. The same principle applies to trim, EQ, cutoff, resonance and supported tempo faders.

Pickup re-arms on startup/deck selection. Cutoff/resonance also re-arm when that side's Shift changes. If the required feedback is missing, the control stays blocked and the console explains why.

Pickup does not re-arm after every external mouse/controller edit. Restart the bridge after AP restarts or if state is uncertain. The shared headphone MIX knob has no pickup. Leave the advanced `--direct-tempo` option **off**: it bypasses tempo pickup and can cause jumps.

## Use the pads and controls

**Mode button:** select its normal bank. **Shift+mode:** select its alternate bank, then release Shift. **Shift+pad:** use the selected bank's shifted action.

| Control | Action |
|---|---|
| LOAD | Load the Browser selection into the selected deck |
| Browse turn / press / Shift+press | Up/down / enter / back |
| SYNC tap / hold | Sync / Tempo Master, where supported |
| CUE / Shift+CUE | Hold cue / jump to Grid Start |
| Filter / same-side Shift+filter | Cutoff / resonance |
| Headphone CUE | Toggle headphone monitoring |
| HOT CUE | Track/clip cues 1–8; arrangement marker layout below |
| PAD FX | Repeat lengths and dotted/triplet timing |
| BEAT JUMP | ±1/2/4/8 beats; hold Shift for ±16/32/64/128 |
| SAMPLER | Select sampler lanes 1–8, not individual sample hits |
| Shift+HOT CUE | Pitch steps/reset and formant controls |
| Shift+PAD FX | Loop movement and IN/OUT (halve/double while looping) |
| Shift+BEAT JUMP | Deck tools; sampler steps 1–8 on sampler decks |
| Shift+SAMPLER | Navigation/locks; sampler steps 9–16 on sampler decks |

### Beat repeats: PAD FX

Pads 1–4 are the top row; 5–8 are the bottom. Lengths are in **bars**, not beats.

| Pad 1 | Pad 2 | Pad 3 | Pad 4 |
|---|---|---|---|
| Hold 1/32 | Hold 1/16 | Hold 1/8 | Hold 1/4 |
| **Pad 5: hold 1/2** | **Pad 6: unassigned** | **Pad 7: toggle dotted** | **Pad 8: toggle triplet** |

Dotted and triplet are mutually exclusive. Tap the active one again for straight timing. The selection stays latched per deck. The latest held length takes priority; changing timing while holding a length retriggers that repeat.

Timing starts straight. Mouse D/T edits are not tracked. Hot Cue Shift+pad 1 is a separate, always-straight 1/32 shortcut. Graceful exit clears bridge-owned timing switches.

### Arrangement markers

On an arrangement deck, HOT CUE pads **1–5** jump to the five visible markers. Pad **6** jumps to Grid Start; **7/8** select previous/next marker page. Create/edit markers in AP's arrangement editor.

### Sampler patterns

1. Press **SAMPLER** and choose a lane with pads 1–8.
2. Select **Shift+BEAT JUMP** for steps 1–8, or **Shift+SAMPLER** for steps 9–16.
3. **Release Shift.** Tap a pad to enable its step; hold Shift+pad to disable it.
4. Use PLAY to run/pause the pattern. **Save in AP** to retain edits.

These pads edit the selected lane's sequence; they do not play individual hits. For a four-on-the-floor pattern, enable steps 1, 5, 9 and 13 on the kick lane.

[Printable control guide](CONTROL-GUIDE.pdf) · [Offline HTML guide](CONTROL-GUIDE.html) · [Full mapping and OSC routes](MAPPING.md)

## If something does not work

| Symptom | What to do |
|---|---|
| One control moves two decks | Disable AP's physical FLX4 MIDI input. Only the bridge should read it. |
| Knob/fader waits after a deck switch | Bring it to AP's displayed position for pickup. If the console reports missing feedback, make sure Toggle All leaves all feedback boxes checked. |
| Tempo fader stays blocked | Enable TempoSlider feedback and use AP's **−1…+1** tempo API. A build without this feedback cannot provide tempo pickup. Do not bypass pickup to hide the problem. |
| Deck type is `Unknown`; buttons do nothing | Check feedback IP **127.0.0.1**, port **9096**, and PlayerType subscription. Use four-deck view for 3/4. |
| Console says a control is unavailable | The loaded player type has no supported route for that action. Not every bank works on every type. |
| Browse knob does nothing | Select AP's Browser **CONTROL MODE** and show the Browser. |
| No MIDI port / MIDI port busy | Connect the FLX4 and close other programs using its MIDI ports. Check the controller/driver installation if it is absent from Windows. |
| Feedback port busy | Stop the other bridge instance before launching another. |
| Sampler pads do not play hits | They select lanes or set/clear steps; PLAY runs the sequence. |

## Current limits

- Clip support includes play, cue, Grid Start, cues, pitch and phase. **Clip/sampler sync, master and tempo are not mapped.**
- Hot cues are taps, not held previews. Cue deletion, direct sampler hits/mute, velocity editing and arrangement-internal mixing are not mapped.
- No jog/scratch, repeat scrubbing, crossfader, Shift+browse rotation, loop-call arrows or FX assignments.
- No hardware deck-number display or pad cue/step-state LEDs. Read the console/window title; title visibility depends on terminal settings.
- AP may ignore controls that are not visible. The bridge does not switch EQ/filter panels or install learned OSC mappings.
- Speed/reverse holds clear their switch on release, not a saved prior state. Avoid conflicting speed holds or external edits during a hold.
- No automatic reconnection, AP settings changes, background installation or firmware writes. Cleanup cannot guarantee releases after UDP loss, content changes, crashes or forced termination.

---

## Technical reference for developers and AI agents

This section defines the bridge's current contract. It is not an instruction to change AP settings or run diagnostics automatically.

### Runtime and wire contract

| Item | Contract |
|---|---|
| MIDI | One FLX4 input/output; default substring `DDJ-FLX4` must identify exactly one of each |
| OSC input to AP | UDP `127.0.0.1:9090` |
| Feedback | UDP `127.0.0.1:9096`; accept only configured AP source IP; ignore unused addresses |
| Player types | `track`, `arrangement`, `clip`, `sampler`, `empty`; unknown blocks deck actions |
| Tempo route | `/AutoPilot/Deck/{1-4}/TempoSlider`, **float −1..+1 both directions**; 0 is native tempo |
| Tempo output | `2 * physical_position - 1`; physical position is 14-bit MIDI divided by 16383. Central codes 8191/8192 send exact 0 |
| Tempo feedback | Store `(feedback + 1) / 2` for normalized pickup; reject non-finite/out-of-range values and re-arm tempo pickup |
| Mixer input/feedback | **0..1**, unchanged by tempo scaling; never apply the bipolar conversion globally |
| Missing state | No inferred pickup targets or legacy tempo auto-detection; normal tempo output waits for compatible feedback |

With selective feedback instead of Toggle All, enable for **every deck/channel 1–4**:

- Deck: `PlayerType`, `Playing`, `Sync`, `Loop`, **`TempoSlider`**.
- Mixer: `Volume`, `Trim`, `EQ/High`, `EQ/Mid`, `EQ/Low`, `Filter`, `Filter/Resonance`, `Cue`, `Meter`.

[MAPPING.md](MAPPING.md) defines pickup, deck routing and held-action behaviour. A successful bridge build does not establish API compatibility or hardware correctness.

### Files and change boundaries

| File | Responsibility |
|---|---|
| `src/main.rs` | CLI, MIDI/UDP setup and event loop |
| `src/bridge.rs` | Pickup, state, held actions, input and feedback handling |
| `src/mapping.rs` | MIDI addresses and supported player-route translations |
| `src/profile.rs` | Default pad banks, content overrides, JSON validation |
| `MAPPING.md` | Complete default mapping and behaviour |
| `CONTROL-GUIDE.html` / `.pdf` | Offline source and printable guide |
| `scripts/package.ps1` | Binary, docs, licenses, archive and checksum |

Keep both tempo directions and all user-facing documentation consistent when changing the wire contract. Do not infer fixed OSC inputs from similarly named feedback addresses, or equate controls with different semantics across player types. Preserve pad-release routing, pickup and per-deck state. Do not edit AP mapping/settings files, install firmware, publish releases, or run tests/hardware probes without explicit authorization.

### CLI and profiles

```text
flx4-pilot.exe --export-config my-pads.json
flx4-pilot.exe --config my-pads.json
flx4-pilot.exe --list
flx4-pilot.exe --monitor
flx4-pilot.exe --input "#0" --output "#1" --verbose
```

- `--export-config` requires a new output file and exits without MIDI access.
- `--list` lists ports without opening MIDI connections or loading a pad profile. `--monitor` opens input only: no MIDI output, keepalive or OSC. Input/output indices are independent.
- `--osc IP:PORT` and `--feedback IP:PORT` configure networking. Remote operation needs matching address families and reachable addresses. OSC is unauthenticated, unencrypted UDP.
- `--direct-tempo` bypasses pickup but **still sends bipolar values**; it does not enable legacy compatibility. It can cause tempo jumps.
- `--mixer` is a no-op compatibility option.

Profile schema: `version: 1`, eight banks, eight `pads` per bank. Each pad is an action or `null`. Actions have `address`, integer `press`, optional `release`; `{deck}` expands to the selected deck 1–4. Optional `shifted_pads` contains eight actions/nulls; omission reuses normal pads.

Optional `for_player` entries (`track`, `arrangement`, `clip`, `sampler`) supply replacement `name`, `pads` and optional `shifted_pads`. An override without shifted pads reuses its own normal pads.

SlipLoop actions require press 1/release 0. `slip_timing` is `straight` (default), `dotted`, `triplet` or `selected`. Fixed timing temporarily overrides the selection. `toggle: true` is valid only for deck SlipDotted/SlipTriplet with press 1 and no release/timing field. Do not mix raw D/T writes with bridge-owned timing. Custom profiles replace defaults; no general macro engine or MIDI passthrough is provided.

### Build and package

Requires stable Rust plus MSVC/Windows SDK, Xcode command-line tools, or a Linux C compiler with `pkg-config` and ALSA headers (`libasound2-dev` on Debian/Ubuntu).

```text
cargo build --release --locked --target x86_64-pc-windows-msvc
powershell -NoProfile -File scripts/package.ps1 -Target x86_64-pc-windows-msvc
```

Use the appropriate target and `pwsh` on other platforms. The manual **Build FLX4 Pilot** workflow covers Windows x64, macOS ARM64/Intel and Linux x64. Packaging uses a unique temporary directory, cleans it on exit, and replaces the output only after creating the archive and checksum. It respects Cargo's target directory and includes docs, control guide, MIT license and dependency notice files. Windows uses the static MSVC runtime; Linux needs compatible system ALSA/libc. Binaries are unsigned/unnotarized. [Release status](STATUS.md) records build and validation coverage.

To check a downloaded Windows ZIP's integrity, compare this command's hash with its accompanying `.zip.sha256` file:

```powershell
Get-FileHash .\flx4-pilot-0.3.3-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

A matching checksum detects corruption; it is not a publisher signature.

### References

- [AlphaTheta MIDI specification](https://downloads.support.alphatheta.com/software_info/dj-controllers/DDJ-FLX4/DDJ-FLX4_MIDI_message_List_E1.pdf), pp.1–3 and p.5 footnote 3: controller messages and SYNC timing.
- [AP guide §13.4](https://autopilot.audio/user-guide/index.html?c=c13-4): general OSC routes/feedback; §4.6: Browser Control Mode. Consult the explicit bipolar tempo contract above for this bridge.
- [Mixxx FLX4 implementation](https://github.com/mixxxdj/mixxx/blob/main/res/controllers/Pioneer-DDJ-FLX4-script.js), `sendKeepAlive`, `padModeKeyPressed`, `vuMeterUpdate`: protocol reference, not a runtime dependency.

Independent project, not affiliated with AlphaTheta/Pioneer DJ or AUTO/PILOT.
