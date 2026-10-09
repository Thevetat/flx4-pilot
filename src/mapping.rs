use crate::profile::Action;

pub const MODES: [u8; 8] = [0x1b, 0x1e, 0x20, 0x22, 0x69, 0x6b, 0x6d, 0x6f];
pub const KEEPALIVE: [u8; 12] = [
    0xf0, 0x00, 0x40, 0x05, 0x00, 0x00, 0x04, 0x05, 0x00, 0x50, 0x02, 0xf7,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    Track,
    Arrangement,
    Clip,
    Sampler,
    Empty,
    Unknown,
}

impl Player {
    pub fn parse(value: &str) -> Self {
        match value {
            "track" => Self::Track,
            "arrangement" => Self::Arrangement,
            "clip" => Self::Clip,
            "sampler" => Self::Sampler,
            "empty" => Self::Empty,
            _ => Self::Unknown,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Track => "track",
            Self::Arrangement => "arrangement",
            Self::Clip => "clip",
            Self::Sampler => "sampler",
            Self::Empty => "empty",
            Self::Unknown => "unknown",
        }
    }

    // Only translate routes documented for the loaded player. No guessed aliases.
    pub fn route(self, route: &str) -> Option<String> {
        match self {
            Self::Track if matches!(route, "ControlPagePrevious" | "ControlPageNext") => None,
            Self::Track | Self::Arrangement
                if !route.starts_with("Sampler/") && !route.starts_with("Clip/") =>
            {
                Some(route.into())
            }
            Self::Clip => match route {
                "PlayPause" | "Cue" | "GridStart" | "Cue1" | "Cue2" | "Cue3" | "Cue4" | "Cue5"
                | "Cue6" | "Cue7" | "Cue8" | "PitchDown" | "PitchReset" | "PitchUp"
                | "PitchStep" | "PhaseMinus" | "PhasePlus" | "PhaseBend" | "PhaseReset" => {
                    Some(format!("Clip/{route}"))
                }
                _ if route.starts_with("Clip/") => Some(route.into()),
                _ => None,
            },
            Self::Sampler => match route {
                "PlayPause" | "PhaseBend" | "PhaseReset" | "Speed" | "Quantize" | "FaderStart" => {
                    Some(format!("Sampler/{route}"))
                }
                _ if route.starts_with("Sampler/") => Some(route.into()),
                _ => None,
            },
            _ => None,
        }
    }
}

pub fn deck_button(note: u8) -> Option<Action> {
    let (route, release) = match note {
        0x0b => ("PlayPause", None),
        0x0c => ("Cue", Some(0)),
        0x48 => ("GridStart", None),
        0x10 => ("LoopIn", None),
        0x11 => ("LoopOut", None),
        0x4d => ("Loop", None),
        0x58 => ("TempoSync", None),
        0x5c => ("TempoMaster", None),
        _ => return None,
    };
    Some(Action::deck(route, 1, release))
}

// Use physical keys so a release is still recognised after Shift or deck changes.
pub fn key(channel: u8, note: u8) -> (u8, u8) {
    let note = match (channel, note) {
        (0..=1, 0x0e) => 0x0b,
        (0..=1, 0x48) => 0x0c,
        (0..=1, 0x4c) => 0x10,
        (0..=1, 0x4e) => 0x11,
        (0..=1, 0x50) => 0x4d,
        (0..=1, 0x5c | 0x60) => 0x58,
        (0..=1, 0x68) => 0x54,
        (0..=1, 0x69) => 0x1b,
        (0..=1, 0x6b) => 0x1e,
        (0..=1, 0x6d) => 0x20,
        (0..=1, 0x6f) => 0x22,
        (6, 0x68) => 0x46,
        (6, 0x7a) => 0x47,
        (6, 0x42) => 0x41,
        (7..=10, _) => return (7 + ((channel - 7) / 2) * 2, note & 0x0f),
        _ => note,
    };
    (channel, note)
}

pub fn analog_address(
    channel: u8,
    cc: u8,
    selected: [usize; 2],
    shifted: [bool; 2],
) -> Option<String> {
    let mixer =
        |side: usize, route: &str| format!("/AutoPilot/Mixer/CH{}/{route}", selected[side] + 1);
    Some(match (channel, cc) {
        (0..=1, 0x00) => format!(
            "/AutoPilot/Deck/{}/TempoSlider",
            selected[channel as usize] + 1
        ),
        (0..=1, 0x13) => mixer(channel as usize, "Volume"),
        (0..=1, 0x04) => mixer(channel as usize, "Trim"),
        (0..=1, 0x07) => mixer(channel as usize, "EQ/High"),
        (0..=1, 0x0b) => mixer(channel as usize, "EQ/Mid"),
        (0..=1, 0x0f) => mixer(channel as usize, "EQ/Low"),
        (6, 0x17..=0x18) => {
            let side = (cc - 0x17) as usize;
            mixer(
                side,
                if shifted[side] {
                    "Filter/Resonance"
                } else {
                    "Filter"
                },
            )
        }
        (6, 0x0c) => "/AutoPilot/Header/CueMix".into(),
        _ => return None,
    })
}
