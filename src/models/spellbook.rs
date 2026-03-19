use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub enum SpineStyle {
    StarsAndDiamonds,
    Celestial,
    DotsAndTherefore,
    Alchemy,
    Geometric,
    Minimal,
}

impl Default for SpineStyle {
    fn default() -> Self {
        SpineStyle::StarsAndDiamonds
    }
}

impl SpineStyle {
    pub fn from_index(index: usize) -> Self {
        match index % 6 {
            0 => SpineStyle::StarsAndDiamonds,
            1 => SpineStyle::Celestial,
            2 => SpineStyle::DotsAndTherefore,
            3 => SpineStyle::Alchemy,
            4 => SpineStyle::Geometric,
            5 => SpineStyle::Minimal,
            _ => SpineStyle::StarsAndDiamonds,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "stars_and_diamonds" => SpineStyle::StarsAndDiamonds,
            "celestial" => SpineStyle::Celestial,
            "dots_and_therefore" => SpineStyle::DotsAndTherefore,
            "alchemy" => SpineStyle::Alchemy,
            "geometric" => SpineStyle::Geometric,
            "minimal" => SpineStyle::Minimal,
            _ => SpineStyle::StarsAndDiamonds,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            SpineStyle::StarsAndDiamonds => "stars_and_diamonds",
            SpineStyle::Celestial => "celestial",
            SpineStyle::DotsAndTherefore => "dots_and_therefore",
            SpineStyle::Alchemy => "alchemy",
            SpineStyle::Geometric => "geometric",
            SpineStyle::Minimal => "minimal",
        }
    }
}

impl<'de> Deserialize<'de> for SpineStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SpineStyle::from_str(&s))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Spellbook {
    pub name: String,
    pub cover: String,
    pub sigil: String,
    /// References spells by ID (populated after load)
    #[serde(default)]
    pub spell_ids: Vec<u64>,
    /// References spells by name (from TOML file)
    #[serde(default)]
    pub spells: Vec<String>,
    /// Spine decoration style (auto-assigned by default)
    #[serde(default)]
    pub style: Option<SpineStyle>,
}
