use crate::config::ThemeName;
use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Debug)]
pub struct AppTheme {
    pub name: &'static str,
    pub accent: Color,
    pub text: Color,
    pub muted_text: Color,
    pub title_style: Style,
}

#[derive(Default)]
pub struct ThemeCatalog;

impl ThemeCatalog {
    pub fn resolve(&self, theme: &ThemeName) -> AppTheme {
        match theme {
            ThemeName::DarkPlus => AppTheme {
                name: "DarkPlus",
                accent: Color::Cyan,
                text: Color::White,
                muted_text: Color::Gray,
                title_style: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            },
            ThemeName::Light => AppTheme {
                name: "Light",
                accent: Color::Blue,
                text: Color::Black,
                muted_text: Color::DarkGray,
                title_style: Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            },
            ThemeName::Monokai => AppTheme {
                name: "Monokai",
                accent: Color::Green,
                text: Color::White,
                muted_text: Color::LightGreen,
                title_style: Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            },
            ThemeName::SolarizedDark => AppTheme {
                name: "SolarizedDark",
                accent: Color::Rgb(38, 139, 210),
                text: Color::Rgb(131, 148, 150),
                muted_text: Color::Rgb(88, 110, 117),
                title_style: Style::default().fg(Color::Rgb(38, 139, 210)).add_modifier(Modifier::BOLD),
            },
            ThemeName::SolarizedLight => AppTheme {
                name: "SolarizedLight",
                accent: Color::Rgb(38, 139, 210),
                text: Color::Rgb(88, 110, 117),
                muted_text: Color::Rgb(147, 161, 161),
                title_style: Style::default().fg(Color::Rgb(38, 139, 210)).add_modifier(Modifier::BOLD),
            },
            ThemeName::Dracula => AppTheme {
                name: "Dracula",
                accent: Color::Magenta,
                text: Color::Rgb(248, 248, 242),
                muted_text: Color::Rgb(98, 114, 164),
                title_style: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            },
            ThemeName::OneDark => AppTheme {
                name: "OneDark",
                accent: Color::Rgb(97, 175, 239),
                text: Color::Rgb(171, 178, 191),
                muted_text: Color::Rgb(92, 99, 112),
                title_style: Style::default().fg(Color::Rgb(97, 175, 239)).add_modifier(Modifier::BOLD),
            },
            ThemeName::Nord => AppTheme {
                name: "Nord",
                accent: Color::Rgb(136, 192, 208),
                text: Color::Rgb(216, 222, 233),
                muted_text: Color::Rgb(129, 161, 193),
                title_style: Style::default().fg(Color::Rgb(136, 192, 208)).add_modifier(Modifier::BOLD),
            },
            ThemeName::Gruvbox => AppTheme {
                name: "Gruvbox",
                accent: Color::Rgb(215, 153, 33),
                text: Color::Rgb(235, 219, 178),
                muted_text: Color::Rgb(146, 131, 116),
                title_style: Style::default().fg(Color::Rgb(215, 153, 33)).add_modifier(Modifier::BOLD),
            },
            ThemeName::Peacocks => AppTheme {
                name: "Peacocks",
                accent: Color::Rgb(80, 220, 150),
                text: Color::Rgb(220, 240, 235),
                muted_text: Color::Rgb(120, 160, 150),
                title_style: Style::default().fg(Color::Rgb(80, 220, 150)).add_modifier(Modifier::BOLD),
            },
        }
    }
}
