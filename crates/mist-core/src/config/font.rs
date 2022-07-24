use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

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
    /// Get the path to the font file, and the index of the font, as determined by `font_kit`.
    pub fn get_bytes(&self) -> Result<(Arc<Vec<u8>>, u32), String> {
        self.ty.get_bytes()
    }
    pub fn get_path(&self) -> Result<(PathBuf, u32), String> {
        self.ty.get_path()
    }
    /// Get the size of the font.
    pub fn size(&self) -> u16 {
        self.size
    }
    /// Get the default font and size for the timer.
    pub fn timer_default() -> Self {
        Self {
            ty: FontType::timer_default(),
            size: 60,
        }
    }
    /// Get the default font and size for the splits.
    pub fn splits_default() -> Self {
        Self {
            ty: FontType::splits_default(),
            size: 25,
        }
    }
}

impl FontType {
    fn get_bytes(&self) -> Result<(Arc<Vec<u8>>, u32), String> {
        match self {
            Self::File { path } => {
                let mut buf = vec![];
                let mut f = std::fs::File::open(path).unwrap();
                f.read_to_end(&mut buf).unwrap();
                Ok((Arc::new(buf), 0))
            }
            Self::System {
                name,
                style,
                weight,
            } => {
                let props = Properties {
                    style: style.into(),
                    weight: weight.into(),
                    ..Default::default()
                };
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
                match handle {
                    Handle::Path { path, font_index } => {
                        let mut buf = vec![];
                        let mut f = std::fs::File::open(path).unwrap();
                        f.read_to_end(&mut buf).unwrap();
                        Ok((Arc::new(buf), font_index))
                    }
                    Handle::Memory { bytes, font_index } => Ok((bytes, font_index)),
                }
            }
        }
    }
    fn get_path(&self) -> Result<(PathBuf, u32), String> {
        match self {
            Self::File { path } => Ok((path.into(), 0)),
            Self::System {
                name,
                style,
                weight,
            } => {
                let props = Properties {
                    style: style.into(),
                    weight: weight.into(),
                    ..Default::default()
                };
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
                match handle {
                    Handle::Path { path, font_index } => Ok((path, font_index)),
                    _ => Err(String::from("Font not accessible as a path")),
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
