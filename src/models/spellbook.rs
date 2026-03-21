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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spine_style_from_index_cycles_correctly() {
        assert_eq!(SpineStyle::from_index(0), SpineStyle::StarsAndDiamonds);
        assert_eq!(SpineStyle::from_index(1), SpineStyle::Celestial);
        assert_eq!(SpineStyle::from_index(2), SpineStyle::DotsAndTherefore);
        assert_eq!(SpineStyle::from_index(3), SpineStyle::Alchemy);
        assert_eq!(SpineStyle::from_index(4), SpineStyle::Geometric);
        assert_eq!(SpineStyle::from_index(5), SpineStyle::Minimal);
    }

    #[test]
    fn test_spine_style_from_index_wraps_at_6() {
        assert_eq!(SpineStyle::from_index(6), SpineStyle::StarsAndDiamonds);
        assert_eq!(SpineStyle::from_index(7), SpineStyle::Celestial);
        assert_eq!(SpineStyle::from_index(11), SpineStyle::Minimal);
        assert_eq!(SpineStyle::from_index(12), SpineStyle::StarsAndDiamonds);
    }

    #[test]
    fn test_spine_style_from_str_parses_correctly() {
        assert_eq!(SpineStyle::from_str("stars_and_diamonds"), SpineStyle::StarsAndDiamonds);
        assert_eq!(SpineStyle::from_str("celestial"), SpineStyle::Celestial);
        assert_eq!(SpineStyle::from_str("dots_and_therefore"), SpineStyle::DotsAndTherefore);
        assert_eq!(SpineStyle::from_str("alchemy"), SpineStyle::Alchemy);
        assert_eq!(SpineStyle::from_str("geometric"), SpineStyle::Geometric);
        assert_eq!(SpineStyle::from_str("minimal"), SpineStyle::Minimal);
    }

    #[test]
    fn test_spine_style_from_str_case_sensitive() {
        assert_eq!(SpineStyle::from_str("CELESTIAL"), SpineStyle::StarsAndDiamonds);
        assert_eq!(SpineStyle::from_str("Celestial"), SpineStyle::StarsAndDiamonds);
        assert_eq!(SpineStyle::from_str("minimal"), SpineStyle::Minimal);
    }

    #[test]
    fn test_spine_style_from_str_defaults_on_unknown() {
        assert_eq!(SpineStyle::from_str("unknown_style"), SpineStyle::StarsAndDiamonds);
        assert_eq!(SpineStyle::from_str(""), SpineStyle::StarsAndDiamonds);
        assert_eq!(SpineStyle::from_str("asdf"), SpineStyle::StarsAndDiamonds);
    }

    #[test]
    fn test_spine_style_to_str_produces_correct_strings() {
        assert_eq!(SpineStyle::StarsAndDiamonds.to_str(), "stars_and_diamonds");
        assert_eq!(SpineStyle::Celestial.to_str(), "celestial");
        assert_eq!(SpineStyle::DotsAndTherefore.to_str(), "dots_and_therefore");
        assert_eq!(SpineStyle::Alchemy.to_str(), "alchemy");
        assert_eq!(SpineStyle::Geometric.to_str(), "geometric");
        assert_eq!(SpineStyle::Minimal.to_str(), "minimal");
    }

    #[test]
    fn test_spine_style_roundtrip_from_str() {
        for style in [
            SpineStyle::StarsAndDiamonds,
            SpineStyle::Celestial,
            SpineStyle::DotsAndTherefore,
            SpineStyle::Alchemy,
            SpineStyle::Geometric,
            SpineStyle::Minimal,
        ] {
            assert_eq!(SpineStyle::from_str(style.to_str()), style);
        }
    }

    #[test]
    fn test_spine_style_default_is_stars_and_diamonds() {
        assert_eq!(SpineStyle::default(), SpineStyle::StarsAndDiamonds);
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
