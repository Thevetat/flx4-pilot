# FLX4 Pilot

Control up to four decks in **[AUTO/PILOT](https://autopilot.audio/)** with a **Pioneer DJ DDJ-FLX4**. A native Rust MIDI → OSC bridge with deck switching, content-aware pads, mixer pickup and LED feedback. No firmware changes or virtual MIDI cable.

**0.3.3 · Windows x64 preview.** Hardware behaviour is unverified. Jog/scratch, repeat scrubbing, crossfader and FX are not mapped. See [release status](STATUS.md) for platform coverage and limitations.

## Get started

**Downloads:** no prebuilt binaries are published on [GitHub Releases](https://github.com/Thevetat/flx4-pilot/releases) yet. Build from source with [Rust](https://rustup.rs/) and your platform's [build prerequisites](DEVELOPMENT.md#build-and-package). Configure AP below before starting:

```sh
git clone https://github.com/Thevetat/flx4-pilot.git
cd flx4-pilot
cargo run --release --locked
```

`cargo run` builds and starts the bridge; `cargo build --release --locked` builds without starting it.

Already have a Windows build? Extract the ZIP and double-click **Start FLX4 Pilot.cmd**. Keep the extracted files together; do not run from inside the ZIP. Rust is not required for a prebuilt executable.

Keep the console open while playing. **Ctrl+C** stops the bridge and attempts to release held controls. Restart it after unplugging the FLX4 or restarting AP.

## Configure AUTO/PILOT

Connect the FLX4. Run AP and the bridge on the same computer, close other DJ applications using its MIDI ports, and run only one bridge instance.

In AP's **Settings**:

| Setting | Value |
|---|---|
| MIDI → INPUTS → DDJ-FLX4 | **Disabled** — the bridge handles the controller |
| OSC → Input / Listen port | **Enabled / 9090** |
| OSC FEEDBACK → Destination IP / port | **127.0.0.1 / 9096** |
| OSC FEEDBACK → Update rate | **30 Hz** |
| OSC FEEDBACK → Subscriptions | **Toggle All → all checked**, including TempoSlider |
| GENERAL → BROWSER MODE | **CONTROL MODE** |

Leaving AP's direct FLX4 MIDI input enabled causes duplicate control. Leave audio settings and saved mappings alone. Keep the Browser visible; use **Perform mode and four-deck view** for decks 3/4.

**Tempo compatibility:** faders require AP's **−1..+1 TempoSlider input and feedback** (0 = native tempo). Legacy 0–1 APIs are unsupported. Missing feedback leaves normal tempo control blocked; other supported controls can still work.

## Choose your deck

| Side | Starts on | Switch decks |
|---|---|---|
| Left | Deck 1 | Hold **left Shift**, tap **left LOAD**, release Shift: **1 ↔ 3** |
| Right | Deck 2 | Hold **right Shift**, tap **right LOAD**, release Shift: **2 ↔ 4** |

The sides switch independently; other decks keep playing. Any deck can hold a track, arrangement, clip or sampler. Supported transport, pads, mixer controls and lights follow the selection: choosing Deck 3 makes the left fader control main mixer channel 3.

- **Read the console/title:** mode lights show the pad bank, not the deck number.
- **Shift+LOAD only selects.** Normal LOAD loads the Browser selection into that deck; switching does not stop, sync or make it master.
- Each deck remembers its bank and repeat timing. Held actions still release on their original deck. Restart selects **1/2**.
- **Pickup:** after startup or switching decks, move a knob/fader to AP's displayed position before it takes control. Missing feedback can also block it.

A **mode button** selects its normal bank; **Shift+mode** selects its alternate bank, then release Shift. **Shift+pad** uses the selected bank's shifted action—not a different deck.

## Controls and help

- [User guide](USER-GUIDE.md) — pickup, pads, sampler patterns and troubleshooting
- [Printable control guide](CONTROL-GUIDE.pdf) · [Offline HTML guide](CONTROL-GUIDE.html)
- [Full mapping](MAPPING.md) — exact assignments, supported player types and OSC routes
- [Developer / AI reference](DEVELOPMENT.md) — CLI, profiles, protocol contract and packaging
- [Adapting other controllers](CONTROLLER-PORTING.md) — protocol research and porting steps for developers and AI agents

[Source repository](https://github.com/Thevetat/flx4-pilot) · [MIT license](LICENSE)

Independent project, not affiliated with AlphaTheta/Pioneer DJ or AUTO/PILOT.
