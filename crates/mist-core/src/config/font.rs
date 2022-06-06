use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
/// A font as represented in the config file.
pub struct Font {
    ty: FontType,
    size: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FontType {
    System {
        name: String,
        style: Style,
        weight: Weight,
    },
    File {
        path: PathBuf,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Weight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl From<&Weight> for font_kit::properties::Weight {
    fn from(w: &Weight) -> Self {
        use Weight::*;
        match w {
            Thin => Self::THIN,
            ExtraLight => Self::EXTRA_LIGHT,
            Light => Self::LIGHT,
            Normal => Self::NORMAL,
            Medium => Self::MEDIUM,
            SemiBold => Self::SEMIBOLD,
            Bold => Self::BOLD,
            ExtraBold => Self::EXTRA_BOLD,
            Black => Self::BLACK,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Style {
    Normal,
    Italic,
    Oblique,
}

impl From<&Style> for font_kit::properties::Style {
    fn from(s: &Style) -> Self {
        use Style::*;
        match s {
            Normal => Self::Normal,
            Italic => Self::Italic,
            Oblique => Self::Oblique,
        }
    }
}

impl Font {
    pub fn get_path(&self) -> Result<(PathBuf, u32), String> {
        self.ty.get_path()
    }
    pub fn size(&self) -> u16 {
        self.size
    }
    pub fn timer_default() -> Self {
        Self {
            ty: FontType::timer_default(),
            size: 60,
        }
    }
    pub fn splits_default() -> Self {
        Self {
            ty: FontType::splits_default(),
            size: 25,
        }
    }
}

impl FontType {
    fn get_path(&self) -> Result<(PathBuf, u32), String> {
        match self {
            Self::File { path } => Ok((path.to_owned(), 0)),
            Self::System {
                name,
                style,
                weight,
            } => {
                let mut props = Properties::default();
                props.style = style.into();
                props.weight = weight.into();
                let family_name = match name.to_lowercase().as_str() {
                    "serif" => FamilyName::Serif,
                    "sansserif" => FamilyName::SansSerif,
                    "monospace" => FamilyName::Monospace,
                    "cursive" => FamilyName::Cursive,
                    "fantasy" => FamilyName::Fantasy,
                    _ => FamilyName::Title(name.to_owned()),
                };
                let handle = SystemSource::new()
                    .select_best_match(&[family_name], &props)
                    .map_err(|_| format!("Could not locate font {name} {style:?}"))?;
                if let Handle::Path { path, font_index } = handle {
                    Ok((path, font_index))
                } else {
                    Err("How did we get here?".into())
                }
            }
        }
    }
    fn timer_default() -> Self {
        Self::System {
            name: "SansSerif".into(),
            style: Style::Normal,
            weight: Weight::Bold,
        }
    }
    fn splits_default() -> Self {
        Self::System {
            name: "SansSerif".into(),
            style: Style::Normal,
            weight: Weight::Medium,
        }
    }
}
