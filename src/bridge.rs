use crate::{
    mapping::{self, Player, MODES},
    profile::{Action, Profile, SlipTiming},
};
use anyhow::{anyhow, Result};
use midir::MidiOutputConnection;
use rosc::{OscMessage, OscPacket, OscType};
use std::{
    collections::{HashMap, HashSet},
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

type Key = (u8, u8);

pub struct Bridge {
    pub osc: UdpSocket,
    pub destination: SocketAddr,
    midi: MidiOutputConnection,
    profile: Profile,
    selected: [usize; 2],
    banks: [usize; 4],
    players: [Player; 4],
    pressed: HashSet<Key>,
    // Press order matters: the latest held slip-loop pad owns its deck.
    held: Vec<(Key, Action)>,
    // Bridge-owned timing selection, retained independently of held pads and deck selection.
    slip_modes: HashMap<String, SlipTiming>,
    msb: HashMap<Key, u8>,
    positions: HashMap<Key, u16>,
    values: HashMap<String, f32>,
    pending_cue: HashMap<String, (f32, Instant)>,
    captured: HashSet<String>,
    warnings: HashSet<String>,
    leds: HashMap<Key, u8>,
    direct_tempo: bool,
    verbose: bool,
}

impl Bridge {
    pub fn new(
        profile: Profile,
        midi: MidiOutputConnection,
        osc: UdpSocket,
        destination: SocketAddr,
        direct_tempo: bool,
        verbose: bool,
    ) -> Self {
        Self {
            osc,
            destination,
            midi,
            profile,
            selected: [0, 1],
            banks: [0; 4],
            players: [Player::Unknown; 4],
            pressed: HashSet::new(),
            held: Vec::new(),
            slip_modes: HashMap::new(),
            msb: HashMap::new(),
            positions: HashMap::new(),
            values: HashMap::new(),
            pending_cue: HashMap::new(),
            captured: HashSet::new(),
            warnings: HashSet::new(),
            leds: HashMap::new(),
            direct_tempo,
            verbose,
        }
    }

    pub fn send(&self, address: &str, value: OscType) -> Result<()> {
        if self.verbose {
            println!("OSC {address} {value:?}");
        }
        let packet = OscPacket::Message(OscMessage {
            addr: address.into(),
            args: vec![value],
        });
        self.osc
            .send_to(&rosc::encoder::encode(&packet)?, self.destination)?;
        Ok(())
    }

    fn warn(&mut self, message: String) {
        if self.warnings.insert(message.clone()) {
            eprintln!("{message}");
        }
    }

    pub fn start(&mut self) -> Result<()> {
        for side in 0..2 {
            self.mode_feedback(side)?;
            self.refresh_leds(side)?;
        }
        self.show_selection();
        self.send("/AutoPilot/Feedback/Resend", OscType::Int(1))
    }

    pub fn keepalive(&mut self) -> Result<()> {
        self.midi
            .send(&mapping::KEEPALIVE)
            .map_err(|e| anyhow!("FLX4 keepalive: {e}"))
    }

    fn show_selection(&self) {
        let sides = ["LEFT", "RIGHT"]
            .iter()
            .enumerate()
            .map(|(side, label)| {
                let deck = self.selected[side];
                let timing = if matches!(self.players[deck], Player::Track | Player::Arrangement) {
                    format!(
                        " / Repeat {:?}",
                        self.selected_timing(&format!("/AutoPilot/Deck/{}", deck + 1))
                    )
                } else {
                    String::new()
                };
                format!(
                    "{label}: Deck {} / {:?} / {}{timing}",
                    deck + 1,
                    self.players[deck],
                    self.profile.banks[self.banks[deck]].name(self.players[deck].name())
                )
            })
            .collect::<Vec<_>>()
            .join("   |   ");
        let title: String = format!("FLX4 Pilot | {sides}")
            .chars()
            .filter(|c| !c.is_control())
            .collect();
        // Keep selection visible even when warnings or verbose traffic scroll past it.
        #[cfg(windows)]
        {
            #[link(name = "Kernel32")]
            extern "system" {
                fn SetConsoleTitleW(title: *const u16) -> i32;
            }
            let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
            // The buffer is NUL-terminated and remains alive for the synchronous call.
            unsafe {
                SetConsoleTitleW(wide.as_ptr());
            }
        }
        #[cfg(not(windows))]
        {
            use std::io::IsTerminal;
            if std::io::stdout().is_terminal() {
                print!("\x1b]0;{title}\x07");
            }
        }
        println!("{sides}");
    }

    fn mode_feedback(&mut self, side: usize) -> Result<()> {
        let bank = self.banks[self.selected[side]];
        self.midi
            .send(&[0x90 + side as u8, MODES[bank], 127])
            .map_err(|e| anyhow!("Mode feedback: {e}"))
    }

    fn switch_deck(&mut self, side: usize) -> Result<()> {
        self.selected[side] ^= 2;
        let deck = self.selected[side];
        // Require a fresh snapshot for the newly selected deck, not an old cached position.
        let deck_prefix = format!("/autopilot/deck/{}/", deck + 1);
        let mixer_prefix = format!("/autopilot/mixer/ch{}/", deck + 1);
        self.values
            .retain(|k, _| !k.starts_with(&deck_prefix) && !k.starts_with(&mixer_prefix));
        self.captured
            .retain(|k| !k.starts_with(&deck_prefix) && !k.starts_with(&mixer_prefix));
        self.players[deck] = Player::Unknown;
        self.warnings.clear();
        self.mode_feedback(side)?;
        self.refresh_leds(side)?;
        self.show_selection();
        self.send("/AutoPilot/Feedback/Resend", OscType::Int(1))
    }

    fn address(&mut self, address: String) -> Option<String> {
        let Some(rest) = address.strip_prefix("/AutoPilot/Deck/") else {
            return Some(address);
        };
        let (number, route) = rest.split_once('/')?;
        let deck = number.parse::<usize>().ok()?.checked_sub(1)?;
        let player = *self.players.get(deck)?;
        match player.route(route) {
            Some(route) => Some(format!("/AutoPilot/Deck/{}/{route}", deck + 1)),
            None => {
                let reason = match player {
                    Player::Unknown => "PlayerType feedback is required.",
                    Player::Empty => "Load content first.",
                    _ => "No supported OSC equivalent in this layout.",
                };
                self.warn(format!(
                    "Deck {} ({player:?}): {route} unavailable. {reason}",
                    deck + 1
                ));
                None
            }
        }
    }

    fn active_slip(&self, deck: &str) -> Option<&Action> {
        self.held
            .iter()
            .rev()
            .map(|(_, action)| action)
            .find(|action| action.slip_deck() == Some(deck))
    }

    fn selected_timing(&self, deck: &str) -> SlipTiming {
        self.slip_modes
            .get(deck)
            .copied()
            .unwrap_or(SlipTiming::Straight)
    }

    fn toggle_slip(&mut self, address: &str) -> Result<()> {
        let (deck, route) = address
            .rsplit_once('/')
            .ok_or_else(|| anyhow!("Invalid slip toggle"))?;
        let requested = match route {
            "SlipDotted" => SlipTiming::Dotted,
            "SlipTriplet" => SlipTiming::Triplet,
            _ => return Err(anyhow!("Unsupported toggle route: {address}")),
        };
        let next = if self.selected_timing(deck) == requested {
            SlipTiming::Straight
        } else {
            requested
        };
        // Record ownership before sending, so error cleanup can clear a partial update.
        self.slip_modes.insert(deck.into(), next);
        if let Some(active) = self.active_slip(deck) {
            if active.slip_timing == Some(SlipTiming::Selected) {
                self.send(&active.address, OscType::Int(0))?;
                self.start_slip(active)?;
            }
            // A fixed-timing shortcut temporarily overrides the selection until release.
        } else {
            self.slip_timing(deck, next)?;
        }
        self.show_selection();
        Ok(())
    }

    fn slip_timing(&self, deck: &str, timing: SlipTiming) -> Result<()> {
        let timing = if timing == SlipTiming::Selected {
            self.selected_timing(deck)
        } else {
            timing
        };
        // Always clear both before enabling either; no unreported prior state is assumed.
        self.send(&format!("{deck}/SlipDotted"), OscType::Int(0))?;
        self.send(&format!("{deck}/SlipTriplet"), OscType::Int(0))?;
        let route = match timing {
            SlipTiming::Dotted => "SlipDotted",
            SlipTiming::Triplet => "SlipTriplet",
            _ => return Ok(()),
        };
        self.send(&format!("{deck}/{route}"), OscType::Int(1))
    }

    fn start_slip(&self, action: &Action) -> Result<()> {
        if let (Some(deck), Some(timing)) = (action.slip_deck(), action.slip_timing) {
            self.slip_timing(deck, timing)?;
        }
        self.send(&action.address, OscType::Int(action.press))
    }

    fn press(&mut self, key: Key, mut action: Action) -> Result<()> {
        let Some(address) = self.address(action.address.clone()) else {
            return Ok(());
        };
        action.address = address;
        if action.toggle {
            return self.toggle_slip(&action.address);
        }
        let previous = action
            .slip_deck()
            .and_then(|deck| self.active_slip(deck))
            .cloned();
        // Record before sending, so error cleanup also covers a partial modifier sequence.
        if action.release.is_some() {
            self.held.push((key, action.clone()));
        }
        if let Some(previous) = previous {
            self.send(&previous.address, OscType::Int(0))?;
        }
        self.start_slip(&action)
    }

    fn release(&mut self, key: Key) -> Result<()> {
        self.pressed.remove(&key);
        let Some(index) = self.held.iter().position(|(held, _)| *held == key) else {
            return Ok(());
        };
        let active = self.held[index].1.slip_deck().is_some_and(|deck| {
            !self.held[index + 1..]
                .iter()
                .any(|(_, action)| action.slip_deck() == Some(deck))
        });
        let action = &self.held[index].1;
        if let Some(deck) = action.slip_deck() {
            if active {
                self.send(&action.address, OscType::Int(0))?;
                if let Some((_, previous)) = self.held[..index]
                    .iter()
                    .rev()
                    .find(|(_, held)| held.slip_deck() == Some(deck))
                {
                    self.start_slip(previous)?;
                } else {
                    self.slip_timing(deck, SlipTiming::Selected)?;
                }
            }
        } else if !self
            .held
            .iter()
            .enumerate()
            .any(|(i, (_, held))| i != index && held.address == action.address)
        {
            if let Some(value) = action.release {
                self.send(&action.address, OscType::Int(value))?;
            }
        }
        // Retain the record until all sends succeed, including modifier release.
        self.held.remove(index);
        Ok(())
    }

    fn toggle_cue(&mut self, deck: usize) -> Result<()> {
        let address = format!("/AutoPilot/Mixer/CH{}/Cue", deck + 1);
        let key = address.to_ascii_lowercase();
        if let Some((_, sent)) = self.pending_cue.get(&key) {
            if sent.elapsed() >= Duration::from_secs(1) {
                self.pending_cue.remove(&key);
                self.values.remove(&key);
                self.send("/AutoPilot/Feedback/Resend", OscType::Int(1))?;
            }
            self.warn(format!(
                "CH{} CUE: waiting for confirmed feedback; press again after it arrives.",
                deck + 1
            ));
            return Ok(());
        }
        let Some(current) = self.values.get(&key) else {
            self.warn(format!(
                "CH{} CUE needs its Cue feedback subscription; no state assumed.",
                deck + 1
            ));
            return Ok(());
        };
        let next = if *current >= 0.5 { 0 } else { 1 };
        self.send(&address, OscType::Int(next))?;
        self.pending_cue.insert(key, (next as f32, Instant::now()));
        Ok(())
    }

    pub fn input(&mut self, bytes: &[u8]) -> Result<()> {
        if self.verbose {
            println!("MIDI IN {:02X?}", bytes);
        }
        if bytes.len() != 3 || bytes[1] > 127 || bytes[2] > 127 {
            return Ok(());
        }
        let (kind, channel, note, value) = (bytes[0] & 0xf0, bytes[0] & 0x0f, bytes[1], bytes[2]);
        if kind == 0xb0 {
            return self.cc(channel, note, value);
        }
        if kind != 0x90 && kind != 0x80 {
            return Ok(());
        }
        let key = mapping::key(channel, note);
        if channel <= 1 && note == 0x3f {
            let changed = if kind == 0x90 && value != 0 {
                self.pressed.insert(key)
            } else {
                self.pressed.remove(&key)
            };
            if changed {
                let deck = self.selected[channel as usize] + 1;
                for route in ["filter", "filter/resonance"] {
                    self.captured
                        .remove(&format!("/autopilot/mixer/ch{deck}/{route}"));
                }
                // Do not join a pre-Shift MSB with a post-Shift LSB or use an old crossing.
                let knob = (6, 0x17 + channel);
                self.msb.remove(&knob);
                self.positions.remove(&knob);
                self.send("/AutoPilot/Feedback/Resend", OscType::Int(1))?;
            }
            return Ok(());
        }
        if kind == 0x80 || value == 0 {
            return self.release(key);
        }
        if !self.pressed.insert(key) {
            return Ok(());
        }

        if channel <= 1 {
            let side = channel as usize;
            let deck = self.selected[side];
            if let Some(bank) = MODES.iter().position(|n| *n == note) {
                self.banks[deck] = bank;
                self.mode_feedback(side)?;
                self.show_selection();
            } else if note == 0x54 {
                self.toggle_cue(deck)?;
            } else if let Some(action) = mapping::deck_button(note) {
                self.press(key, action.resolved(deck))?;
            }
        } else if channel == 6 {
            match note {
                0x68 => self.switch_deck(0)?,
                0x7a => self.switch_deck(1)?,
                0x46 | 0x47 => {
                    let deck = self.selected[(note - 0x46) as usize];
                    self.send(
                        &format!("/AutoPilot/Browser/L{}", deck + 1),
                        OscType::Int(1),
                    )?;
                }
                0x41 => self.send("/AutoPilot/Browser/Enter", OscType::Int(1))?,
                0x42 => self.send("/AutoPilot/Browser/Back", OscType::Int(1))?,
                _ => {}
            }
        } else if (7..=10).contains(&channel) && note & 0x0f < 8 {
            let side = ((channel - 7) / 2) as usize;
            let deck = self.selected[side];
            let bank = &self.profile.banks[self.banks[deck]];
            let pad = (note & 0x0f) as usize;
            let action = bank
                .action(self.players[deck].name(), pad, (channel - 7) % 2 != 0)
                .cloned();
            if let Some(action) = action {
                self.press(key, action.resolved(deck))?;
            }
        }
        Ok(())
    }

    fn cc(&mut self, channel: u8, number: u8, value: u8) -> Result<()> {
        // B6 40: signed relative counts (01 clockwise, 7F counterclockwise).
        // Unlike the mixer knobs, this encoder is not a 14-bit absolute control.
        if channel == 6 && number == 0x40 {
            let steps = if value < 0x40 {
                value as i16
            } else {
                value as i16 - 128
            };
            let address = if steps > 0 {
                "/AutoPilot/Browser/Down"
            } else {
                "/AutoPilot/Browser/Up"
            };
            for _ in 0..steps.unsigned_abs() {
                self.send(address, OscType::Int(1))?;
            }
            return Ok(());
        }
        if number >= 0x40 {
            return Ok(());
        }
        let base = number & 0x1f;
        let shifted = [
            self.pressed.contains(&(0, 0x3f)),
            self.pressed.contains(&(1, 0x3f)),
        ];
        let Some(address) = mapping::analog_address(channel, base, self.selected, shifted) else {
            return Ok(());
        };
        let physical = (channel, base);
        if number < 0x20 {
            self.msb.insert(physical, value);
            return Ok(());
        }
        let Some(high) = self.msb.get(&physical) else {
            return Ok(());
        };
        let full = ((*high as u16) << 7) | value as u16;
        let previous = self.positions.insert(physical, full);
        // The keepalive can report positions. Neither startup reports nor unchanged values are movement.
        let Some(previous) = previous.filter(|previous| *previous != full) else {
            return Ok(());
        };
        let Some(address) = self.address(address) else {
            return Ok(());
        };
        let key = address.to_ascii_lowercase();
        let position = full as f32 / 16383.0;
        let tempo = key.ends_with("/temposlider");
        let direct = key == "/autopilot/header/cuemix" || (self.direct_tempo && tempo);
        if !direct && !self.captured.contains(&key) {
            let Some(target) = self.values.get(&key) else {
                let hint = if tempo {
                    "Enable TempoSlider feedback (or Toggle All). Requires AP's bipolar -1..+1 tempo API."
                } else {
                    "Enable this OSC feedback subscription."
                };
                self.warn(format!("Pickup blocked: {address}. {hint}"));
                return Ok(());
            };
            let previous = previous as f32 / 16383.0;
            let crossed = (previous - target) * (position - target) <= 0.0;
            if !crossed && (position - target).abs() > 2.0 / 16383.0 {
                return Ok(());
            }
            self.captured.insert(key);
        }
        // Mixer/pickup stay normalized; only tempo uses bipolar OSC values.
        let output = if tempo {
            // Either middle 14-bit code represents the native-tempo detent.
            if matches!(full, 8191 | 8192) {
                0.0
            } else {
                position * 2.0 - 1.0
            }
        } else {
            position
        };
        self.send(&address, OscType::Float(output))
    }

    fn led(&mut self, status: u8, note: u8, value: u8) -> Result<()> {
        if self.leds.get(&(status, note)) != Some(&value) {
            self.midi
                .send(&[status, note, value])
                .map_err(|e| anyhow!("MIDI output: {e}"))?;
            self.leds.insert((status, note), value);
        }
        Ok(())
    }

    fn refresh_leds(&mut self, side: usize) -> Result<()> {
        let deck = self.selected[side] + 1;
        for (address, note) in [
            (format!("/autopilot/deck/{deck}/playing"), 0x0b),
            (format!("/autopilot/deck/{deck}/sync"), 0x58),
            (format!("/autopilot/deck/{deck}/loop"), 0x4d),
            (format!("/autopilot/mixer/ch{deck}/cue"), 0x54),
        ] {
            let on = self.values.get(&address).is_some_and(|v| *v >= 0.5);
            self.led(0x90 + side as u8, note, if on { 127 } else { 0 })?;
        }
        let meter = self
            .values
            .get(&format!("/autopilot/mixer/ch{deck}/meter"))
            .copied()
            .unwrap_or(0.0);
        self.led(
            0xb0 + side as u8,
            0x02,
            (meter.clamp(0.0, 1.0) * 127.0).round() as u8,
        )
    }

    /// Returns whether the packet contained usable control state, not just valid OSC.
    pub fn feedback(&mut self, packet: OscPacket, depth: u8) -> Result<bool> {
        if depth > 8 {
            return Ok(false);
        }
        match packet {
            OscPacket::Bundle(bundle) => {
                let mut accepted = false;
                for packet in bundle.content {
                    accepted |= self.feedback(packet, depth + 1)?;
                }
                Ok(accepted)
            }
            OscPacket::Message(message) => {
                let address = message.addr.to_ascii_lowercase();
                // Keep only known control-state namespaces; ignore other OSC traffic.
                if !address.starts_with("/autopilot/") {
                    return Ok(false);
                }
                if let [OscType::String(value)] = message.args.as_slice() {
                    for deck in 0..4 {
                        if address == format!("/autopilot/deck/{}/playertype", deck + 1) {
                            let player = Player::parse(&value.to_ascii_lowercase());
                            if self.players[deck] != player {
                                self.players[deck] = player;
                                self.captured
                                    .remove(&format!("/autopilot/deck/{}/temposlider", deck + 1));
                                if self.selected.contains(&deck) {
                                    self.show_selection();
                                }
                            }
                            return Ok(player != Player::Unknown);
                        }
                    }
                    return Ok(false);
                }
                let value = match message.args.as_slice() {
                    [OscType::Int(value)] => *value as f32,
                    [OscType::Float(value)] => *value,
                    _ => return Ok(false),
                };
                // Bound the cache to the routes actually used by this bridge.
                let scope = address
                    .strip_prefix("/autopilot/deck/")
                    .map(|rest| (rest, false))
                    .or_else(|| {
                        address
                            .strip_prefix("/autopilot/mixer/ch")
                            .map(|rest| (rest, true))
                    });
                let Some((rest, mixer)) = scope else {
                    return Ok(false);
                };
                let Some((number, route)) = rest.split_once('/') else {
                    return Ok(false);
                };
                let Ok(number @ 1..=4) = number.parse::<usize>() else {
                    return Ok(false);
                };
                let used = if mixer {
                    matches!(
                        route,
                        "volume"
                            | "trim"
                            | "eq/high"
                            | "eq/mid"
                            | "eq/low"
                            | "filter"
                            | "filter/resonance"
                            | "cue"
                            | "meter"
                    )
                } else {
                    matches!(route, "playing" | "sync" | "loop" | "temposlider")
                };
                if !used {
                    return Ok(false);
                }
                let tempo = !mixer && route == "temposlider";
                if !value.is_finite() || (tempo && !(-1.0..=1.0).contains(&value)) {
                    if tempo {
                        self.values.remove(&address);
                        self.captured.remove(&address);
                        self.warn(format!("Invalid TempoSlider feedback for Deck {number}; expected a finite value in -1..+1. Pickup re-armed."));
                    }
                    return Ok(false);
                }
                // The pickup cache always uses 0..1, regardless of OSC wire scale.
                let value = if tempo { (value + 1.0) * 0.5 } else { value };
                self.values.insert(address.clone(), value);
                if self
                    .pending_cue
                    .get(&address)
                    .is_some_and(|(expected, _)| (*expected - value).abs() < 0.01)
                {
                    self.pending_cue.remove(&address);
                }
                for side in 0..2 {
                    if self.selected[side] + 1 == number {
                        self.refresh_leds(side)?;
                    }
                }
                Ok(true)
            }
        }
    }

    pub fn cleanup(&mut self) {
        let mut slip_decks: HashSet<String> = self.slip_modes.keys().cloned().collect();
        for (_, action) in std::mem::take(&mut self.held) {
            if let Some(value) = action.release {
                if let Err(e) = self.send(&action.address, OscType::Int(value)) {
                    eprintln!("Release failed: {e}");
                }
            }
            if let Some(deck) = action.slip_deck() {
                slip_decks.insert(deck.into());
            }
        }
        for deck in slip_decks {
            for route in ["SlipDotted", "SlipTriplet"] {
                if let Err(e) = self.send(&format!("{deck}/{route}"), OscType::Int(0)) {
                    eprintln!("Slip modifier release failed: {e}");
                }
            }
        }
        self.slip_modes.clear();
        for side in 0..2 {
            for note in [0x0b, 0x58, 0x4d, 0x54] {
                let _ = self.midi.send(&[0x90 + side, note, 0]);
            }
            let _ = self.midi.send(&[0xb0 + side, 0x02, 0]);
            let _ = self.midi.send(&[0x90 + side, MODES[0], 127]);
        }
    }
}
