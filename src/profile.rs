use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlipTiming {
    Straight,
    Dotted,
    Triplet,
    /// Use the deck's latched timing instead of a fixed per-pad override.
    Selected,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    /// OSC address; {deck} expands to the selected deck (1–4).
    pub address: String,
    pub press: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release: Option<i32>,
    /// Fixed timing or the deck's selected timing for a held slip loop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slip_timing: Option<SlipTiming>,
    /// Bridge-owned, mutually exclusive SlipDotted/SlipTriplet toggle.
    #[serde(default, skip_serializing_if = "is_false")]
    pub toggle: bool,
}

fn is_false(value: &bool) -> bool {
    !value
}

impl Action {
    pub fn deck(route: &str, press: i32, release: Option<i32>) -> Self {
        Self {
            address: format!("/AutoPilot/Deck/{{deck}}/{route}"),
            press,
            release,
            toggle: false,
            slip_timing: route
                .starts_with("SlipLoop/")
                .then_some(SlipTiming::Straight),
        }
    }

    fn slip(length: &str, timing: SlipTiming) -> Self {
        Self {
            slip_timing: Some(timing),
            ..Self::deck(&format!("SlipLoop/{length}"), 1, Some(0))
        }
    }

    fn toggle(route: &str) -> Self {
        Self {
            toggle: true,
            ..Self::deck(route, 1, None)
        }
    }

    pub fn resolved(&self, deck: usize) -> Self {
        Self {
            address: self.address.replace("{deck}", &(deck + 1).to_string()),
            ..self.clone()
        }
    }

    pub fn slip_deck(&self) -> Option<&str> {
        self.slip_timing?;
        self.address.rsplit_once("/SlipLoop/").map(|(deck, _)| deck)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PadLayout {
    pub name: String,
    pub pads: Vec<Option<Action>>,
    /// Omit to reuse pads; null explicitly leaves a shifted pad unassigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shifted_pads: Option<Vec<Option<Action>>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bank {
    pub name: String,
    pub pads: Vec<Option<Action>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shifted_pads: Option<Vec<Option<Action>>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub for_player: BTreeMap<String, PadLayout>,
}

impl Bank {
    pub fn name(&self, player: &str) -> &str {
        self.for_player
            .get(player)
            .map_or(&self.name, |layout| &layout.name)
    }

    pub fn action(&self, player: &str, pad: usize, shifted: bool) -> Option<&Action> {
        let (pads, shifted_pads) = self
            .for_player
            .get(player)
            .map_or((&self.pads, &self.shifted_pads), |layout| {
                (&layout.pads, &layout.shifted_pads)
            });
        if shifted {
            if let Some(pads) = shifted_pads {
                return pads[pad].as_ref();
            }
        }
        pads[pad].as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub version: u32,
    pub banks: Vec<Bank>,
}

impl Default for Profile {
    fn default() -> Self {
        let bank = |name: &str, pads: Vec<Action>| Bank {
            name: name.into(),
            pads: pads.into_iter().map(Some).collect(),
            shifted_pads: None,
            for_player: BTreeMap::new(),
        };
        let triggers = |route: &str, values: &[i32]| -> Vec<Action> {
            values
                .iter()
                .map(|v| Action::deck(route, *v, None))
                .collect()
        };
        let mut profile = Self {
            version: 1,
            banks: vec![
                bank(
                    "Hot cues",
                    (1..=8)
                        .map(|i| Action::deck(&format!("Cue{i}"), 1, None))
                        .collect(),
                ),
                bank(
                    "Slip loops",
                    vec![
                        Action::slip("ThirtySecond", SlipTiming::Selected),
                        Action::slip("Sixteenth", SlipTiming::Selected),
                        Action::slip("Eighth", SlipTiming::Selected),
                        Action::slip("Quarter", SlipTiming::Selected),
                        Action::slip("Half", SlipTiming::Selected),
                        Action::toggle("SlipDotted"),
                        Action::toggle("SlipTriplet"),
                    ],
                ),
                bank(
                    "Beat jumps",
                    triggers("BeatSkip", &[-1, 1, -2, 2, -4, 4, -8, 8]),
                ),
                bank(
                    "Sampler lanes",
                    triggers("Sampler/Lane", &[0, 1, 2, 3, 4, 5, 6, 7]),
                ),
                bank(
                    "Pitch / formant",
                    vec![
                        Action::deck("PitchStep", -12, None),
                        Action::deck("PitchStep", -1, None),
                        Action::deck("PitchStep", 1, None),
                        Action::deck("PitchStep", 12, None),
                        Action::deck("PitchReset", 1, None),
                        Action::deck("FormantDown", 1, None),
                        Action::deck("FormantReset", 1, None),
                        Action::deck("FormantUp", 1, None),
                    ],
                ),
                bank(
                    "Loop tools",
                    vec![
                        Action::deck("LoopMoveBeat", -1, None),
                        Action::deck("LoopMoveBeat", 1, None),
                        Action::deck("LoopMoveBar", -1, None),
                        Action::deck("LoopMoveBar", 1, None),
                        Action::deck("LoopMoveBar", -4, None),
                        Action::deck("LoopMoveBar", 4, None),
                        Action::deck("LoopIn", 1, None),
                        Action::deck("LoopOut", 1, None),
                    ],
                ),
                bank(
                    "Deck tools",
                    vec![
                        Action::deck("SpeedHalf", 1, Some(0)),
                        Action::deck("SpeedDouble", 1, Some(0)),
                        Action::deck("Reverse", 1, Some(0)),
                        Action::deck("GridStart", 1, None),
                        Action::deck("PhaseMinus", 1, Some(0)),
                        Action::deck("PhasePlus", 1, Some(0)),
                        Action::deck("PhaseReset", 1, None),
                        Action::deck("Speed", 1, None),
                    ],
                ),
                bank(
                    "Navigation / locks",
                    vec![
                        Action::deck("WaveformZoomOut", 1, None),
                        Action::deck("WaveformZoomIn", 1, None),
                        Action::deck("ControlPagePrevious", 1, None),
                        Action::deck("ControlPageNext", 1, None),
                        Action::deck("CueSlip", 1, None),
                        Action::deck("CueSlip", 0, None),
                        Action::deck("PitchPreserve", 1, None),
                        Action::deck("PitchPreserve", 0, None),
                    ],
                ),
            ],
        };
        profile.banks[1].pads.insert(5, None);
        profile.banks[0].shifted_pads = Some(vec![None; 8]);
        let mut markers: Vec<_> = (1..=5)
            .map(|i| Action::deck(&format!("Cue{i}"), 1, None))
            .collect();
        markers.extend(
            ["GridStart", "ControlPagePrevious", "ControlPageNext"]
                .map(|route| Action::deck(route, 1, None)),
        );
        profile.banks[0].for_player.insert(
            "arrangement".into(),
            PadLayout {
                name: "Markers / pages".into(),
                pads: markers.into_iter().map(Some).collect(),
                shifted_pads: Some(vec![None; 8]),
            },
        );
        profile.banks[2].shifted_pads = Some(
            triggers("BeatSkip", &[-16, 16, -32, 32, -64, 64, -128, 128])
                .into_iter()
                .map(Some)
                .collect(),
        );
        for (bank, first) in [(6, 1), (7, 9)] {
            let steps = |value| {
                (first..first + 8)
                    .map(|step| Action::deck(&format!("Sampler/Step{step}/Enabled"), value, None))
                    .collect::<Vec<_>>()
            };
            profile.banks[bank].for_player.insert(
                "sampler".into(),
                PadLayout {
                    name: format!("Steps {first}–{} (Shift clears)", first + 7),
                    pads: steps(1).into_iter().map(Some).collect(),
                    shifted_pads: Some(steps(0).into_iter().map(Some).collect()),
                },
            );
        }
        profile
    }
}

impl Profile {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let mut profile: Self = match path {
            Some(path) => serde_json::from_str(
                &std::fs::read_to_string(path)
                    .with_context(|| format!("Read profile {}", path.display()))?,
            )?,
            None => Self::default(),
        };
        if profile.version != 1 || profile.banks.len() != 8 {
            bail!("Profile must have version 1 and exactly eight banks");
        }
        for bank in &mut profile.banks {
            for player in bank.for_player.keys() {
                if !matches!(
                    player.as_str(),
                    "track" | "arrangement" | "clip" | "sampler"
                ) {
                    bail!("Unknown player override {player:?}");
                }
            }
            let layouts = std::iter::once((&bank.name, &mut bank.pads, &mut bank.shifted_pads))
                .chain(
                    bank.for_player
                        .values_mut()
                        .map(|layout| (&layout.name, &mut layout.pads, &mut layout.shifted_pads)),
                );
            for (name, pads, shifted) in layouts {
                if pads.len() != 8 || shifted.as_ref().is_some_and(|pads| pads.len() != 8) {
                    bail!("Layout {name:?} must have eight pads");
                }
                for action in pads
                    .iter_mut()
                    .chain(shifted.iter_mut().flatten())
                    .filter_map(Option::as_mut)
                {
                    if !action.address.starts_with("/AutoPilot/")
                        || action.address.chars().any(char::is_whitespace)
                    {
                        bail!("Invalid OSC address {:?}", action.address);
                    }
                    let expanded = action.address.replace("{deck}", "1");
                    if expanded.contains(['{', '}', '*', '?', '[', ']', ',', '\0']) {
                        bail!("Only {{deck}} substitution and literal OSC routes are supported");
                    }
                    if action.toggle {
                        let valid = expanded.rsplit_once('/').is_some_and(|(deck, route)| {
                            matches!(
                                deck,
                                "/AutoPilot/Deck/1"
                                    | "/AutoPilot/Deck/2"
                                    | "/AutoPilot/Deck/3"
                                    | "/AutoPilot/Deck/4"
                            ) && matches!(route, "SlipDotted" | "SlipTriplet")
                        });
                        if !valid
                            || action.press != 1
                            || action.release.is_some()
                            || action.slip_timing.is_some()
                        {
                            bail!("toggle requires SlipDotted/SlipTriplet, press 1 and no release/slip_timing");
                        }
                    }
                    // Old profiles' slip loops join the same ownership/release policy.
                    let slip = expanded.split_once("/SlipLoop/");
                    if let Some((deck, length)) = slip {
                        if !matches!(
                            deck,
                            "/AutoPilot/Deck/1"
                                | "/AutoPilot/Deck/2"
                                | "/AutoPilot/Deck/3"
                                | "/AutoPilot/Deck/4"
                        ) || !matches!(
                            length,
                            "ThirtySecond"
                                | "Sixteenth"
                                | "Eighth"
                                | "Quarter"
                                | "Half"
                                | "One"
                                | "Two"
                                | "Four"
                        ) || action.press != 1
                            || action.release != Some(0)
                        {
                            bail!(
                                "Slip loops require a deck SlipLoop route, press 1 and release 0"
                            );
                        }
                        action.slip_timing.get_or_insert(SlipTiming::Straight);
                    } else if action.slip_timing.is_some() {
                        bail!("slip_timing requires a SlipLoop route");
                    }
                }
            }
        }
        Ok(profile)
    }
}
