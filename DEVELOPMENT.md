# Developer and AI reference

[Quick start](README.md) · [User guide](USER-GUIDE.md) · [Full mapping](MAPPING.md) · [Controller porting](CONTROLLER-PORTING.md)

This document defines the bridge's current contract. It is not an instruction to change AP settings or run diagnostics automatically.

## Build and package

Install stable [Rust](https://rustup.rs/) and the native build tools for your platform:

| Platform | Prerequisites |
|---|---|
| Windows | MSVC Rust toolchain; Visual Studio Build Tools with **Desktop development with C++**, including the Windows SDK |
| macOS | Xcode command-line tools |
| Linux | C compiler, `pkg-config` and ALSA development headers; Debian/Ubuntu packages: `build-essential pkg-config libasound2-dev` |

From the repository root:

```sh
cargo run --release --locked
```

This compiles and starts the bridge. To compile only, use `cargo build --release --locked`; the executable is in `target/release/` (`flx4-pilot.exe` on Windows). Neither command requires the packaging script. Running connects to the controller and AP, so complete the [AP setup](README.md#configure-autopilot) first.

To make a distributable Windows archive:

```powershell
cargo build --release --locked --target x86_64-pc-windows-msvc
powershell -NoProfile -File scripts/package.ps1 -Target x86_64-pc-windows-msvc
```

Use the appropriate target and `pwsh` on other platforms. The manual **Build FLX4 Pilot** workflow covers Windows x64, macOS ARM64/Intel and Linux x64. It builds and uploads workflow artifacts; it does not publish a GitHub Release or run tests.

Packaging uses a unique temporary directory, cleans it on exit, and replaces the output only after creating the archive and checksum. It respects Cargo's target directory and includes the user/developer docs, control guide, MIT license and dependency notice files. Windows uses the static MSVC runtime; Linux needs compatible system ALSA/libc. Binaries are unsigned/unnotarized. [Release status](STATUS.md) records build and validation coverage.

To check a downloaded Windows ZIP's integrity, compare this command's hash with its accompanying `.zip.sha256` file:

```powershell
Get-FileHash .\flx4-pilot-0.3.3-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

A matching checksum detects corruption; it is not a publisher signature.

## Runtime and wire contract

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

## Files and change boundaries

| File | Responsibility |
|---|---|
| `src/main.rs` | CLI, MIDI/UDP setup and event loop |
| `src/bridge.rs` | Pickup, state, held actions, input and feedback handling |
| `src/mapping.rs` | MIDI addresses and supported player-route translations |
| `src/profile.rs` | Default pad banks, content overrides, JSON validation |
| `README.md` / `USER-GUIDE.md` | Quick start and operating instructions |
| `MAPPING.md` | Complete default mapping and behaviour |
| `CONTROL-GUIDE.html` / `.pdf` | Offline source and printable guide |
| `scripts/package.ps1` | Binary, docs, licenses, archive and checksum |

Keep both tempo directions and all user-facing documentation consistent when changing the wire contract. Do not infer fixed OSC inputs from similarly named feedback addresses, or equate controls with different semantics across player types. Preserve pad-release routing, pickup and per-deck state. Do not edit AP mapping/settings files, install firmware, publish releases, or run tests/hardware probes without explicit authorization.

## CLI and profiles

```text
flx4-pilot.exe --export-config my-pads.json
flx4-pilot.exe --config my-pads.json
flx4-pilot.exe --list
flx4-pilot.exe --monitor
flx4-pilot.exe --input "#0" --output "#1" --verbose
```

On macOS/Linux, use `./flx4-pilot` in the executable's directory. With Cargo, pass bridge options after `--`, for example `cargo run --release --locked -- --config my-pads.json`.

- `--export-config` requires a new output file and exits without MIDI access.
- `--list` lists ports without opening MIDI connections or loading a pad profile. `--monitor` opens input only: no MIDI output, keepalive or OSC. Input/output indices are independent.
- `--osc IP:PORT` and `--feedback IP:PORT` configure networking. Remote operation needs matching address families and reachable addresses. OSC is unauthenticated, unencrypted UDP.
- `--direct-tempo` bypasses pickup but **still sends bipolar values**; it does not enable legacy compatibility. It can cause tempo jumps.
- `--mixer` is a no-op compatibility option.

Profile schema: `version: 1`, eight banks, eight `pads` per bank. Each pad is an action or `null`. Actions have `address`, integer `press`, optional `release`; `{deck}` expands to the selected deck 1–4. Optional `shifted_pads` contains eight actions/nulls; omission reuses normal pads.

Optional `for_player` entries (`track`, `arrangement`, `clip`, `sampler`) supply replacement `name`, `pads` and optional `shifted_pads`. An override without shifted pads reuses its own normal pads.

SlipLoop actions require press 1/release 0. `slip_timing` is `straight` (default), `dotted`, `triplet` or `selected`. Fixed timing temporarily overrides the selection. `toggle: true` is valid only for deck SlipDotted/SlipTriplet with press 1 and no release/timing field. Do not mix raw D/T writes with bridge-owned timing. Custom profiles replace defaults; no general macro engine or MIDI passthrough is provided.

## References

- [AlphaTheta MIDI specification](https://downloads.support.alphatheta.com/software_info/dj-controllers/DDJ-FLX4/DDJ-FLX4_MIDI_message_List_E1.pdf), pp.1–3 and p.5 footnote 3: controller messages and SYNC timing.
- [AP guide §13.4](https://autopilot.audio/user-guide/index.html?c=c13-4): general OSC routes/feedback; §4.6: Browser Control Mode. Consult the explicit bipolar tempo contract above for this bridge.
- [Mixxx FLX4 implementation](https://github.com/mixxxdj/mixxx/blob/main/res/controllers/Pioneer-DDJ-FLX4-script.js), `sendKeepAlive`, `padModeKeyPressed`, `vuMeterUpdate`: protocol reference, not a runtime dependency.
