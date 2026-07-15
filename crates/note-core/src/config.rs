use serde::{Deserialize, Serialize};
use std::{
    env,
    fs,
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AppearanceMode {
    Auto,
    Light,
    Dark,
}

impl Default for AppearanceMode {
    fn default() -> Self {
        Self::Dark
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NoteSettings {
    pub language: String,
    pub appearance: AppearanceMode,
    pub accent: String,
    pub dock_size: i32,
    pub panel_opacity: f64,
    pub dock_opacity: f64,
    pub animations: bool,
    pub natural_scroll: bool,
    pub workspaces: u8,
    pub favorites: Vec<String>,
}

impl Default for NoteSettings {
    fn default() -> Self {
        Self {
            language: "es-MX".to_owned(),
            appearance: AppearanceMode::Dark,
            accent: "blue".to_owned(),
            dock_size: 50,
            panel_opacity: 0.86,
            dock_opacity: 0.78,
            animations: true,
            natural_scroll: true,
            workspaces: 4,
            favorites: vec![
                "foot.desktop".into(),
                "thunar.desktop".into(),
                "org.gnome.Epiphany.desktop".into(),
                "org.gnome.TextEditor.desktop".into(),
                "org.gnome.Settings.desktop".into(),
            ],
        }
    }
}

impl NoteSettings {
    pub fn load() -> Self {
        let path = settings_path();
        match fs::read_to_string(path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = toml::to_string_pretty(self)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        fs::write(path, data)
    }

    pub fn write_locale_environment(&self) -> io::Result<()> {
        let path = environment_dir().join("90-note-locale.conf");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let locale = locale_to_posix(&self.language);
        fs::write(
            path,
            format!(
                "LANG={locale}.UTF-8\nLANGUAGE={}\nLC_MESSAGES={locale}.UTF-8\n",
                locale
            ),
        )
    }

    pub fn css_variables(&self) -> String {
        let (r, g, b) = accent_rgb(&self.accent);
        format!(
            "@define-color note_accent rgb({r}, {g}, {b});\n\
             @define-color note_panel_bg rgba(20, 22, 28, {:.3});\n\
             @define-color note_dock_bg rgba(26, 28, 35, {:.3});\n",
            self.panel_opacity.clamp(0.25, 1.0),
            self.dock_opacity.clamp(0.25, 1.0),
        )
    }
}

pub fn settings_path() -> PathBuf {
    config_home().join("note-desktop/settings.toml")
}

pub fn config_home() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"))
}

pub fn environment_dir() -> PathBuf {
    config_home().join("environment.d")
}

pub fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new("/").to_path_buf())
}

fn locale_to_posix(locale: &str) -> String {
    locale.replace('-', "_")
}

fn accent_rgb(accent: &str) -> (u8, u8, u8) {
    match accent {
        "purple" => (175, 112, 255),
        "pink" => (255, 99, 160),
        "red" => (255, 86, 86),
        "orange" => (255, 150, 62),
        "green" => (79, 210, 132),
        "teal" => (52, 205, 198),
        _ => (84, 151, 255),
    }
}
