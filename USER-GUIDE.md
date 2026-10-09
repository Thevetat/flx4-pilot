# User guide

[Setup and deck selection](README.md) · [Printable controls](CONTROL-GUIDE.pdf) · [Full mapping](MAPPING.md)

## Starting and stopping

Keep the bridge's console open while using the controller. It shows both selected decks, loaded content types, pad banks and repeat timing. Mode-button lights identify the bank, not the deck number.

Run only one bridge instance. **Ctrl+C** stops it and attempts to release held controls. Restart after unplugging the FLX4 or restarting AP; there is no automatic reconnection. Network loss, content changes, crashes and forced termination can prevent releases from reaching AP.

To return to AP's direct MIDI mapping, stop the bridge, then re-enable DDJ-FLX4 under **Settings → MIDI → INPUTS**. Leave your audio setup and saved mapping files unchanged.

## Why a knob or fader may not move AP immediately

This is **pickup**, which prevents jumps when two physical channel strips control four software decks.

If AP's channel volume is low but the hardware fader is high, the bridge waits. Move the fader to AP's displayed position; once it reaches or crosses that position, it takes control. The same principle applies to trim, EQ, cutoff, resonance and supported tempo faders.

Pickup re-arms on startup/deck selection. Cutoff/resonance also re-arm when that side's Shift changes. If the required feedback is missing, the control stays blocked and the console explains why.

Pickup does not re-arm after every external mouse/controller edit. Restart the bridge after AP restarts or if state is uncertain. The shared headphone MIX knob has no pickup. Leave the advanced `--direct-tempo` option **off**: it bypasses tempo pickup and can cause jumps.

## Pads and controls

**Mode button:** select its normal bank. **Shift+mode:** select its alternate bank, then release Shift. **Shift+pad:** use the selected bank's shifted action. **Shift+LOAD:** select a deck, not a bank.

| Control | Action |
|---|---|
| PLAY | Play/pause |
| LOAD | Load the Browser selection into the selected deck |
| Browse turn / press / Shift+press | Up/down / enter / back |
| SYNC tap / hold | Sync / Tempo Master, where supported |
| CUE / Shift+CUE | Hold cue / jump to Grid Start |
| Filter / same-side Shift+filter | Cutoff / resonance |
| Headphone CUE | Toggle headphone monitoring |
| HOT CUE | Track/clip cues 1–8; arrangement marker layout below. Shift+pads are unassigned |
| PAD FX | Repeat lengths and dotted/triplet timing |
| BEAT JUMP | ±1/2/4/8 beats; hold Shift for ±16/32/64/128 |
| SAMPLER | Select sampler lanes 1–8, not individual sample hits |
| Shift+HOT CUE | Pitch steps/reset and formant controls |
| Shift+PAD FX | Loop movement and IN/OUT (halve/double while looping) |
| Shift+BEAT JUMP | Deck tools; sampler steps 1–8 on sampler decks |
| Shift+SAMPLER | Navigation/locks; sampler steps 9–16 on sampler decks |

### Hot cues

Normal pads trigger AP's cues; they are not silent preparation jumps. Shift+pads 1–8 are unassigned: cue jumps without starting playback need a confirmed AP input route. This does not change **Shift+HOT CUE mode**, which still selects the pitch/formant bank. Custom profiles retain their own assignments.

### Beat repeats: PAD FX

Pads 1–4 are the top row; 5–8 are the bottom. Lengths are in **bars**, not beats.

| Pad 1 | Pad 2 | Pad 3 | Pad 4 |
|---|---|---|---|
| Hold 1/32 | Hold 1/16 | Hold 1/8 | Hold 1/4 |
| **Pad 5: hold 1/2** | **Pad 6: unassigned** | **Pad 7: toggle dotted** | **Pad 8: toggle triplet** |

Dotted and triplet are mutually exclusive. Tap the active one again for straight timing. The selection stays latched per deck. The latest held length takes priority; changing timing while holding a length retriggers that repeat.

For example, keep holding pad A, then press and hold B to hear B's repeat. Release B while still holding A to return to A's repeat. A resumed repeat is retriggered, not guaranteed to use its earlier loop region.

Timing starts straight. Mouse D/T edits are not tracked. Graceful exit clears bridge-owned timing switches.

### Arrangement markers

On an arrangement deck, HOT CUE pads **1–5** jump to the five visible markers. Pad **6** jumps to Grid Start; **7/8** select previous/next marker page. Create/edit markers in AP's arrangement editor.

### Sampler patterns

1. Press **SAMPLER** and choose a lane with pads 1–8.
2. Select **Shift+BEAT JUMP** for steps 1–8, or **Shift+SAMPLER** for steps 9–16.
3. **Release Shift.** Tap a pad to enable its step; hold Shift+pad to disable it.
4. Use PLAY to run/pause the pattern. **Save in AP** to retain edits.

These pads edit the selected lane's sequence; they do not play individual hits. For a four-on-the-floor pattern, enable steps 1, 5, 9 and 13 on the kick lane.

## Troubleshooting

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

For custom pad profiles, see the [developer reference](DEVELOPMENT.md#cli-and-profiles).
