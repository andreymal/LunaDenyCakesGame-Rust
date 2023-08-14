use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Sequence)]
pub enum Action {
    Switch,
    Apply,
    SelTeleport,
    SelLaser,
    SelChicken,
    SelShield,
    Left,
    Right,
}

impl Action {
    pub fn code(&self) -> &'static str {
        match self {
            Action::Switch => "switch",
            Action::Apply => "apply",
            Action::SelTeleport => "sel_teleport",
            Action::SelLaser => "sel_laser",
            Action::SelChicken => "sel_chicken",
            Action::SelShield => "sel_shield",
            Action::Left => "left",
            Action::Right => "right",
        }
    }
}
