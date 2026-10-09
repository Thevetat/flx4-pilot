# Release status — 0.3.3

FLX4 Pilot is a native DDJ-FLX4 MIDI → OSC bridge for AUTO/PILOT. It controls up to four decks using stock controller firmware. This is a preview build, not a fully validated controller integration.

## Build coverage

| Target | Status |
|---|---|
| Windows x64 | Compiled and packaged; static MSVC runtime |
| macOS ARM64 / Intel | Build workflow provided; packages unverified |
| Linux x64 | Build workflow provided; package unverified; requires system ALSA/libc |
| Hardware validation | Not performed |
| Automated tests | Not run |
| Signing / notarization | Unsigned / unnotarized |

## Included

- Four-deck selection, with per-deck pad banks and content-aware routing.
- Track cues, arrangement marker paging and sampler lane/step editing.
- Slip loops with per-deck straight/dotted/triplet selection and last-held-pad priority.
- Fixed small/large beat-jump layers, pitch/formant, loop, speed/phase and navigation controls.
- Mixer and bipolar tempo pickup, headphone cue toggles, transport LEDs and channel meters.
- Console/title display of selected decks, types, banks and repeat timing.
- Custom JSON pad profiles, MIDI port listing and read-only MIDI monitoring.
- Setup documentation, full mapping reference, printable/offline control guide, MIT license and dependency notices.

## Compatibility boundaries

- **Tempo:** input and position feedback use **−1..+1**, with 0 at native tempo. Requires AP's matching bipolar API; legacy 0–1 is unsupported and cannot be auto-detected. Feedback is normalized internally for pickup; missing/invalid feedback blocks uncaptured normal tempo output. `--direct-tempo` bypasses pickup but keeps bipolar output and can cause jumps. End-to-end behaviour against AP is unverified.
- **Deck types:** capability coverage varies. Track/arrangement controls are the most extensive; clip/sampler sync, master and tempo are not mapped.
- **Unassigned controls:** jog/scratch, repeat scrubbing, crossfader, FX, loop-call arrows and Shift+browse rotation.
- **Additional gaps:** held hot-cue previews, cue deletion, direct sampler hits/mute, sampler velocity editing and arrangement-internal mixing are not included in the default mapping.
- **State:** repeat timing belongs to the bridge and does not follow external D/T edits. AP may ignore commands for hidden controls. There is no hardware deck-number display or pad-state feedback.
- **Recovery:** no automatic reconnection. Graceful shutdown attempts releases; network loss, content changes and forced termination cannot guarantee delivery.

## Source layout

| File | Responsibility |
|---|---|
| `src/main.rs` | CLI, MIDI/UDP setup and event loop |
| `src/bridge.rs` | Runtime state, input handling, pickup, held actions and feedback |
| `src/mapping.rs` | FLX4 messages and player-specific OSC route translation |
| `src/profile.rs` | Default banks, content overrides and JSON profile validation |
| `scripts/package.ps1` | Native archive, docs, license files and checksum; temporary staging cleaned after each run |
| `CONTROL-GUIDE.html` | Offline/print source for `CONTROL-GUIDE.pdf` |

[README.md](README.md) contains setup and build instructions. [MAPPING.md](MAPPING.md) defines current controls, OSC routes and operating limitations.
