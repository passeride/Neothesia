//! Persistent presentation preferences. All themes are dark to keep notation and
//! the physical piano visually consistent; semantic feedback colors stay stable.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Midnight,
    Graphite,
    Warm,
    HighContrast,
}
impl Theme {
    pub fn name(self) -> &'static str {
        match self {
            Self::Midnight => "Midnight",
            Self::Graphite => "Graphite",
            Self::Warm => "Warm",
            Self::HighContrast => "High Contrast",
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Midnight => Self::Graphite,
            Self::Graphite => Self::Warm,
            Self::Warm => Self::HighContrast,
            Self::HighContrast => Self::Midnight,
        }
    }
    pub fn note_colors(self) -> &'static [super::ColorSchemaV1] {
        use super::ColorSchemaV1 as C;
        match self {
            Self::Midnight => &[
                C {
                    base: (124, 189, 205),
                    dark: (79, 132, 150),
                },
                C {
                    base: (172, 165, 207),
                    dark: (119, 111, 157),
                },
                C {
                    base: (178, 198, 163),
                    dark: (119, 145, 106),
                },
                C {
                    base: (215, 179, 139),
                    dark: (157, 121, 86),
                },
            ],
            Self::Graphite => &[
                C {
                    base: (177, 185, 211),
                    dark: (118, 127, 153),
                },
                C {
                    base: (149, 186, 183),
                    dark: (94, 131, 129),
                },
                C {
                    base: (201, 181, 167),
                    dark: (143, 122, 108),
                },
                C {
                    base: (190, 166, 186),
                    dark: (132, 109, 129),
                },
            ],
            Self::Warm => &[
                C {
                    base: (215, 179, 130),
                    dark: (157, 120, 77),
                },
                C {
                    base: (176, 193, 156),
                    dark: (119, 137, 102),
                },
                C {
                    base: (207, 160, 149),
                    dark: (150, 105, 95),
                },
                C {
                    base: (162, 184, 193),
                    dark: (105, 129, 140),
                },
            ],
            Self::HighContrast => &[
                C {
                    base: (145, 215, 255),
                    dark: (55, 145, 205),
                },
                C {
                    base: (255, 220, 110),
                    dark: (195, 151, 35),
                },
                C {
                    base: (125, 240, 160),
                    dark: (40, 165, 80),
                },
                C {
                    base: (235, 160, 245),
                    dark: (165, 90, 180),
                },
            ],
        }
    }
    pub fn palette(self) -> Palette {
        let (background, bottom, surface, button, accent) = match self {
            Self::Midnight => (
                [13, 20, 31],
                [23, 34, 49],
                [24, 35, 49],
                [36, 49, 65],
                [124, 189, 205],
            ),
            Self::Graphite => (
                [19, 21, 25],
                [35, 38, 44],
                [31, 34, 40],
                [45, 49, 57],
                [177, 185, 211],
            ),
            Self::Warm => (
                [26, 22, 21],
                [47, 37, 32],
                [39, 32, 29],
                [56, 45, 39],
                [215, 179, 130],
            ),
            Self::HighContrast => (
                [0, 0, 0],
                [0, 0, 0],
                [12, 12, 12],
                [36, 36, 36],
                [145, 215, 255],
            ),
        };
        Palette {
            background,
            bottom,
            surface,
            button,
            accent,
            text: [235, 239, 244],
            muted: [162, 174, 190],
            staff: if self == Self::HighContrast {
                [220, 226, 235]
            } else {
                [133, 149, 169]
            },
            success: [112, 218, 169],
            warning: [244, 174, 112],
            target: [236, 201, 139],
        }
    }
}
#[derive(Clone, Copy)]
pub struct Palette {
    pub background: [u8; 3],
    pub bottom: [u8; 3],
    pub surface: [u8; 3],
    pub button: [u8; 3],
    pub accent: [u8; 3],
    pub text: [u8; 3],
    pub muted: [u8; 3],
    pub staff: [u8; 3],
    pub success: [u8; 3],
    pub warning: [u8; 3],
    pub target: [u8; 3],
}
impl Palette {
    /// GPU render targets are sRGB; direct shader inputs must be linear.
    pub fn linear(color: [u8; 3], alpha: f32) -> [f32; 4] {
        let c = color.map(|v| {
            let c = v as f32 / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        });
        [c[0], c[1], c[2], alpha]
    }

    pub fn rgba(color: [u8; 3], alpha: f32) -> [f32; 4] {
        [
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
            alpha,
        ]
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VisualSettings {
    pub theme: Theme,
    pub gradients: bool,
    pub soft_note_colors: bool,
    pub staff_opacity: u8,
    pub staff_size: u16,
    pub score_enabled: bool,
    pub score_scroll: bool,
    pub score_beats: u8,
    pub flats: bool,
    pub input_feedback: bool,
    pub duration_rails: bool,
    pub beat_grid: bool,
    pub practice_sound: bool,
}
impl Default for VisualSettings {
    fn default() -> Self {
        Self {
            theme: Theme::Midnight,
            gradients: true,
            soft_note_colors: true,
            staff_opacity: 88,
            staff_size: 125,
            score_enabled: false,
            score_scroll: true,
            score_beats: 8,
            flats: false,
            input_feedback: true,
            duration_rails: true,
            beat_grid: true,
            practice_sound: false,
        }
    }
}
impl VisualSettings {
    pub fn normalized(mut self) -> Self {
        self.staff_opacity = self.staff_opacity.clamp(55, 100);
        self.staff_size = self.staff_size.clamp(50, 250);
        if ![4, 8, 16].contains(&self.score_beats) {
            self.score_beats = 8;
        }
        self
    }
    pub fn panel_alpha(self) -> f32 {
        if self.theme == Theme::HighContrast {
            1.0
        } else {
            self.staff_opacity as f32 / 100.0
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_partial_preferences_get_defaults_and_round_trip() {
        let old: VisualSettings = ron::from_str("(staff_size:150)").unwrap();
        assert_eq!(old.staff_size, 150);
        assert_eq!(old.theme, Theme::Midnight);
        assert!(old.input_feedback);
        for theme in [
            Theme::Midnight,
            Theme::Graphite,
            Theme::Warm,
            Theme::HighContrast,
        ] {
            let v = VisualSettings {
                theme,
                practice_sound: true,
                ..old
            };
            assert_eq!(
                ron::from_str::<VisualSettings>(&ron::to_string(&v).unwrap()).unwrap(),
                v
            );
        }
    }
    #[test]
    fn legacy_appearance_keeps_device_independent_values() {
        let old: super::super::model::AppearanceConfigV1 =
            ron::from_str("(background_color:(12,23,34),glow:false)").unwrap();
        assert_eq!(old.background_color, (12, 23, 34));
        assert!(!old.glow);
        assert_eq!(old.visual, VisualSettings::default());
        for theme in [
            Theme::Midnight,
            Theme::Graphite,
            Theme::Warm,
            Theme::HighContrast,
        ] {
            assert!(!theme.note_colors().is_empty());
            let p = theme.palette();
            // Readable primary text against every theme's opaque panel.
            let luminance = |rgb: [u8; 3]| {
                rgb.map(|c| {
                    let c = c as f32 / 255.0;
                    if c <= 0.04045 {
                        c / 12.92
                    } else {
                        ((c + 0.055) / 1.055).powf(2.4)
                    }
                })
                .iter()
                .zip([0.2126, 0.7152, 0.0722])
                .map(|(c, w)| c * w)
                .sum::<f32>()
            };
            assert!((luminance(p.text) + 0.05) / (luminance(p.surface) + 0.05) >= 7.0);
        }
    }
    #[test]
    fn malformed_ranges_are_bounded_and_contrast_is_opaque() {
        let v = VisualSettings {
            staff_opacity: 0,
            staff_size: 999,
            score_beats: 0,
            ..Default::default()
        }
        .normalized();
        assert_eq!((v.staff_opacity, v.staff_size, v.score_beats), (55, 250, 8));
        let linear = Palette::linear([128, 255, 0], 0.5);
        assert!((linear[0] - 0.21586).abs() < 0.0001);
        assert_eq!(&linear[1..], &[1.0, 0.0, 0.5]);
        assert_eq!(
            VisualSettings {
                theme: Theme::HighContrast,
                ..v
            }
            .panel_alpha(),
            1.0
        );
    }
}
