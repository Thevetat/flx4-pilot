# Mapping — 0.3.3

Default control assignments for the DDJ-FLX4 → AUTO/PILOT bridge. The bridge does not modify AP mapping files.

`D` means the selected deck; `CHD` its main mixer channel. Addresses below start `/AutoPilot/`.

## Deck selection

| Side | Startup deck | Gesture | Selection |
|---|---|---|---|
| Left | 1 | Hold left Shift, tap left LOAD, release Shift | 1 ↔ 3 |
| Right | 2 | Hold right Shift, tap right LOAD, release Shift | 2 ↔ 4 |

Selection is independent per side. It changes the bridge's target, not AP's playback state, content or tempo-master assignment. All four decks can continue playing. Use AP's four-deck view for decks 3/4; the bridge does not change that view.

Supported deck actions and pads follow the selection. Mixer controls address its corresponding **main channel**: selecting Deck 3 makes the left channel fader control `Mixer/CH3/Volume`, not an arrangement's internal track trim. LEDs/meters follow the selected deck/channel. Normal LOAD targets the selected deck; Shift+LOAD never loads.

The console/window title is the deck-selection reference. Hardware mode lights identify pad banks, not deck numbers. Selection requests fresh feedback and re-arms mixer/tempo pickup; controls can wait until the hardware crosses the reported value. Per-deck pad banks and repeat timing are recalled when returning to a deck. Existing held actions retain their original release destination.

Restart selects decks 1/2. **Shift+mode changes a bank, not the selected deck.** Browser navigation and headphone MIX remain shared.

## Fixed controls

| Physical control | OSC route | Behaviour |
|---|---|---|
| Left / right Shift+LOAD | Bridge-only | Select 1↔3 / 2↔4; never load |
| LOAD | `Browser/LD` | Load selected item into selected deck |
| PLAY | `Deck/D/PlayPause` | Trigger |
| CUE | `Deck/D/Cue` | Press 1, release 0 |
| Shift+CUE | `Deck/D/GridStart` | Trigger |
| SYNC tap / hold | `Deck/D/TempoSync`, `TempoMaster` | Native short/long messages; Shift+SYNC unassigned |
| IN / OUT | `Deck/D/LoopIn`, `LoopOut` | Set points; halve/double while looping |
| 4BEAT/EXIT | `Deck/D/Loop` | Toggle AP's default loop |
| Tempo fader | `Deck/D/TempoSlider` | 14-bit MIDI → bipolar float −1..+1; 0 native tempo; pickup |
| Channel fader / trim | `Mixer/CHD/Volume`, `Trim` | 14-bit normalized, pickup |
| High / Mid / Low | `Mixer/CHD/EQ/High`, `Mid`, `Low` | 14-bit normalized, pickup |
| Filter / same-side Shift+filter | `Mixer/CHD/Filter`, `Filter/Resonance` | 14-bit normalized, pickup |
| Headphone CUE | `Mixer/CHD/Cue` | Toggle from confirmed feedback |
| Headphone MIX | `Header/CueMix` | Shared/direct, not deck-switched |
| Browse rotation | `Browser/Up`, `Down` | Signed relative steps; counterclockwise up |
| Browse press / Shift+press | `Browser/Enter`, `Back` | Trigger |

Browse rotation uses channel 7 CC64 signed relative counts, not the mixer-knob decoder. AP documents hardware navigation under **Settings → General → Browser Mode → CONTROL MODE** (§4.6).

Each side tracks its own Shift. Cutoff/resonance pickup resets on Shift press/release; the next full knob report establishes a baseline. Enable both feedback subscriptions. Mixer knobs address main mixer channels, not arrangement-internal track trims. AP may require the corresponding controls to be visible.

**Tempo contract:** requires AP's bipolar `TempoSlider` input AND feedback: **−1 minimum, 0 native tempo, +1 maximum**. Output maps physical 0–1 to `2 * position - 1`, with the two central MIDI codes sending exact 0. Feedback maps back to `(value + 1) / 2` for pickup; mixer values remain 0–1. Non-finite/out-of-range tempo feedback is rejected and re-arms pickup. Missing feedback blocks normal tempo output. There is no automatic detection or support for legacy 0–1 tempo APIs. `--direct-tempo` bypasses pickup but still sends bipolar values and can cause jumps.

## Pad banks

Pads 1–4 are the top row; 5–8 the bottom. Press a mode button for its **normal bank**, or Shift+mode for its **alternate bank**, then release Shift. Each logical deck remembers its bank. Shift+pad uses that bank's shifted layer, described below.

### Track / arrangement layout

| Selector | Bank | Top row: pads 1–4 | Bottom row: pads 5–8 |
|---|---|---|---|
| HOT CUE | Hot cues | Cue 1, 2, 3, 4 | Cue 5, 6, 7, 8 |
| PAD FX | Slip loops / timing | Hold 1/32, 1/16, 1/8, 1/4 bars | Hold 1/2 bar; unassigned; toggle Dotted; toggle Triplet |
| BEAT JUMP | Beat jumps | −1, +1, −2, +2 beats | −4, +4, −8, +8 beats |
| SAMPLER | Sampler lanes | Inactive unless sampler loaded | Inactive unless sampler loaded |
| Shift+HOT CUE | Pitch / formant | Pitch −12, −1, +1, +12 | Pitch reset, formant −, reset, + |
| Shift+PAD FX | Loop tools | Move −1/+1 beat, −1/+1 bar | Move −4/+4 bars, IN/halve, OUT/double |
| Shift+BEAT JUMP | Deck tools | Hold half speed, double speed, reverse; Grid Start | Hold phase −, +; phase reset; normal speed |
| Shift+SAMPLER | Navigation / locks | Zoom out/in; marker page previous/next | Slip ON/OFF; pitch preservation ON/OFF |

- **Arrangement HOT CUE override:** pads 1–5 are the five visible markers; pad 6 Grid Start; pads 7/8 previous/next marker page. This replaces the eight-cue layout only while `PlayerType=arrangement`.
- **Hot Cue Shift+pad 1:** hold straight 1/32-bar slip loop on tracks/arrangements. Shift+pads 2–8 unassigned.
- **Beat Jump held-Shift layer:** top −16,+16,−32,+32; bottom −64,+64,−128,+128 beats. Release Shift to return. No remembered range cycling.
- Other shifted pads reuse their bank's normal actions, except the sampler step overrides below.
- Pitch needs P (pitch preservation) enabled; formant needs **Formant Mode → Manual**. The bridge does not change AP settings or automatically enable P. Clip pitch transposes MIDI notes; formant pads are blocked on clips.
- IN/OUT halve/double only while looping; otherwise they set loop endpoints. Loop-move amounts are signed whole beats/bars. These pads move the loop; they do not set an absolute loop length.
- Marker paging is arrangement-only. Zoom commands only work where AP exposes the waveform controls.
- Speed/reverse pads set on while held and off on release; they do not restore an unreported pre-existing speed/reverse state. Avoid simultaneous conflicting speed pads or mouse edits during a hold. Normal speed explicitly sets `Speed=1`.

### Slip-loop timing and ownership

PAD FX pads 1–5 use the deck's **selected timing**. Pad 7 toggles Dotted; pad 8 toggles Triplet. Selecting either clears the other; pressing the active one again selects straight timing. Pad 6 is unassigned, including Shift+pad 6. Holding Shift does not change this bank's pad actions.

Timing starts straight and is remembered independently per logical deck until changed or the bridge restarts. It stays selected after releasing all repeat pads and across bank/deck selection. The bridge sends explicit `SlipDotted` / `SlipTriplet` switch values. It clears both before setting either and reasserts its timing when starting a repeat. Console output/window title show the selected repeat timing.

The **latest held length wins per logical deck**: release the previous loop before starting a new one. Releasing the latest restarts the previous still-held loop, now using the current timing. Releasing an older pad does not interrupt the active one. Changing D/T while holding a PAD FX length stops/retriggers that loop with the new timing. Releasing the final length stops repeating but **does not reset D/T**.

Hot Cue Shift+pad 1 is an explicitly **straight** 1/32 shortcut. It temporarily overrides D/T, restoring the selected timing when released (or the previous held loop's timing when resuming it). While a fixed-timing shortcut is held, D/T toggles update the selection for subsequent normal repeats without changing that shortcut.

The bridge owns this state; AP provides no documented D/T feedback, so mouse/other-controller edits are not tracked. A resumed loop is retriggered, not guaranteed to preserve its earlier region. Graceful exit attempts loop releases and clears D/T for every deck whose timing it owns, including latched modes with no pad held. UDP loss, content changes and force-kills cannot guarantee delivery.

### Sampler layout

| Selector | Top row | Bottom row |
|---|---|---|
| SAMPLER | Select lanes 1–4 | Select lanes 5–8 |
| Shift+BEAT JUMP | Steps 1–4 | Steps 5–8 |
| Shift+SAMPLER | Steps 9–12 | Steps 13–16 |

Select a step bank, **release Shift**, then press a pad to enable its step in the currently selected lane. Hold Shift+pad to disable it. These send explicit `Sampler/StepN/Enabled` 1/0; they are not toggles and make no assumption about cached lane state. Select a different lane with SAMPLER, then return to the desired step bank. Lane selection also follows mouse changes because the step route targets AP's current lane. Save the sampler in AP to retain edits.

These are sequencer controls, **not sample-hit triggers**. No velocity editing, lane mute or direct hits are assigned. No extra feedback subscriptions are needed to set/clear steps; pad LEDs do not show step state.

## Content-aware routing

No deck has a prescribed role. `PlayerType` determines both compatible OSC routes and optional pad-layout overrides.

| Player type | Coverage |
|---|---|
| Track | Generic deck routes; cue taps, slips, jumps, pitch/formant, loops, speed/phase, zoom/locks |
| Arrangement | Track-style routes plus five-marker/page Hot Cue layout; no arrangement-internal mixing |
| Clip | PLAY, CUE, Grid Start, eight cue taps, pitch and phase via `Deck/D/Clip/…`; unsupported pads blocked |
| Sampler | PLAY, lane selection, two step banks via `Deck/D/Sampler/…`; explicit sampler routes supported. Custom profiles can also use translated Speed/Quantize/FaderStart/PhaseBend/PhaseReset |
| Empty / unknown | Deck actions blocked; mixer/browser remain available |

Clip `Loop` means restart-at-end, not the track loop toggle; the bridge does not map it to the track loop button. Clip/sampler SYNC/master/tempo are unsupported. Learned OSC mappings can provide additional controls, but must be configured separately in AP.

## Unsupported / unassigned

- Hot-cue held previews require learned OSC mappings; fixed cue routes are taps. No documented fixed cue-delete route.
- Direct sampler hits/mute and arrangement-internal trims/routing lack established fixed input routes. Internal feedback addresses are not evidence of input support.
- Jog/scratch, repeat scrubbing, crossfader, Shift+browse rotation, loop-call arrows, Beat FX, Smart CFX/Fader and other shifted transport buttons are unassigned.
- No EQ/filter-panel toggle or mouse automation.
- No legacy 0–1 tempo API support, hardware deck-number indicator or pad-occupancy/step LEDs.

## Feedback, profiles and safety

In AP's OSC Feedback settings, use **Toggle All** so all subscriptions are checked, including **TempoSlider**. Unused feedback is ignored; README lists the selective equivalent. Play/Sync/Loop/Cue LEDs and meters follow the selected deck. Deck selection requests a fresh snapshot before mixer/tempo pickup and type-dependent actions resume. The console prints both sides' deck number, player type and effective bank name (including marker/step overrides), and updates its window title so selection remains visible above scrolling logs where the terminal supports it.

Held actions keep their original resolved address across bank/deck selection. Content changes, app restarts, network loss and force-kills remain boundaries: stop/restart the bridge when state is uncertain. Profile overrides and `slip_timing` are documented in README.

## Protocol references

- [AlphaTheta MIDI list pp.2,5](https://downloads.support.alphatheta.com/software_info/dj-controllers/DDJ-FLX4/DDJ-FLX4_MIDI_message_List_E1.pdf): Shift+LOAD notes 104/122; native SYNC reports on release.
- [AP guide §13.4](https://autopilot.audio/user-guide/index.html?c=c13), **Track and arrangement decks**, **MIDI clip decks**, **Sampler decks**, **Feedback**: exact routes, switch/trigger/momentary semantics and feedback. §§6.6/6.9 describe loop and marker behaviour.

Hardware behaviour is unverified. See [CONTROL-GUIDE.pdf](CONTROL-GUIDE.pdf) for a printable layout.
