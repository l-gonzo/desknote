pub mod config;
pub mod desktop;
pub mod i18n;
pub mod system;

pub use config::{AppearanceMode, NoteSettings};
pub use desktop::{DesktopApp, discover_apps};
pub use i18n::Translator;
