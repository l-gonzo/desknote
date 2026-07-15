use gtk::{gdk, glib, prelude::*};
use note_core::{AppearanceMode, NoteSettings, Translator};
use std::process::{Command, Stdio};

const APP_ID: &str = "mx.note.desktop.settings";
const FALLBACK_CSS: &str = include_str!("../../../assets/style/settings.css");

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| install_css());
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &gtk::Application) {
    if let Some(window) = app.active_window() {
        window.present();
        return;
    }

    let settings = NoteSettings::load();
    let tr = Translator::new(settings.language.clone());
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title(tr.text("settings-title"))
        .default_width(980)
        .default_height(680)
        .build();
    window.add_css_class("settings-window");

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = gtk::HeaderBar::new();
    header.set_title_widget(Some(&gtk::Label::new(Some(&tr.text("settings-title")))));
    root.append(&header);

    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    body.set_vexpand(true);
    let sidebar = gtk::ListBox::new();
    sidebar.add_css_class("settings-sidebar");
    sidebar.set_selection_mode(gtk::SelectionMode::Single);
    sidebar.set_size_request(230, -1);

    let stack = gtk::Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);

    let controls = build_appearance_page(&stack, &tr, &settings);
    add_page(&sidebar, &stack, "appearance", "preferences-desktop-theme-symbolic", &tr.text("appearance"), controls.page.clone());
    add_page(&sidebar, &stack, "language", "preferences-desktop-locale-symbolic", &tr.text("language-region"), build_language_page(&tr));
    add_page(&sidebar, &stack, "displays", "video-display-symbolic", &tr.text("displays"), build_launcher_page(&tr, "video-display-symbolic", "display-description", &["wdisplays", "arandr"]));
    add_page(&sidebar, &stack, "network", "network-wireless-symbolic", &tr.text("network"), build_launcher_page(&tr, "network-wireless-symbolic", "network-description", &["nm-connection-editor"]));
    add_page(&sidebar, &stack, "bluetooth", "bluetooth-active-symbolic", &tr.text("bluetooth"), build_launcher_page(&tr, "bluetooth-active-symbolic", "bluetooth-description", &["blueman-manager"]));
    add_page(&sidebar, &stack, "sound", "audio-volume-high-symbolic", &tr.text("sound"), build_launcher_page(&tr, "audio-volume-high-symbolic", "sound-description", &["pavucontrol"]));
    add_page(&sidebar, &stack, "power", "battery-good-symbolic", &tr.text("power"), build_info_page(&tr, "battery-good-symbolic", "power-description"));
    add_page(&sidebar, &stack, "keyboard", "input-keyboard-symbolic", &tr.text("keyboard"), build_info_page(&tr, "input-keyboard-symbolic", "keyboard-description"));
    add_page(&sidebar, &stack, "privacy", "security-high-symbolic", &tr.text("privacy"), build_info_page(&tr, "security-high-symbolic", "privacy-description"));
    add_page(&sidebar, &stack, "about", "help-about-symbolic", &tr.text("about"), build_about_page(&tr));

    let stack_clone = stack.clone();
    sidebar.connect_row_selected(move |_, row| {
        let Some(row) = row else { return };
        let name = row.widget_name();
        stack_clone.set_visible_child_name(name.as_str());
    });
    if let Some(row) = sidebar.row_at_index(0) {
        sidebar.select_row(Some(&row));
    }

    body.append(&sidebar);
    body.append(&stack);
    root.append(&body);

    let action_bar = gtk::ActionBar::new();
    let status = gtk::Label::new(None);
    status.add_css_class("save-status");
    action_bar.set_center_widget(Some(&status));
    let reset = gtk::Button::with_label(&tr.text("restore-defaults"));
    let save = gtk::Button::with_label(&tr.text("apply"));
    save.add_css_class("suggested-action");
    action_bar.pack_start(&reset);
    action_bar.pack_end(&save);
    root.append(&action_bar);

    {
        let controls = controls.clone();
        let status = status.clone();
        let tr = tr.clone();
        save.connect_clicked(move |_| {
            let mut new_settings = NoteSettings::load();
            new_settings.appearance = match controls.appearance.selected() {
                1 => AppearanceMode::Light,
                2 => AppearanceMode::Dark,
                _ => AppearanceMode::Auto,
            };
            new_settings.accent = selected_string(&controls.accent).unwrap_or_else(|| "blue".into());
            new_settings.language = language_code(controls.language.selected()).to_owned();
            new_settings.dock_size = controls.dock_size.value().round() as i32;
            new_settings.panel_opacity = controls.panel_opacity.value();
            new_settings.dock_opacity = controls.dock_opacity.value();
            new_settings.animations = controls.animations.is_active();
            new_settings.natural_scroll = controls.natural_scroll.is_active();
            new_settings.workspaces = controls.workspaces.value() as u8;

            match new_settings.save().and_then(|_| new_settings.write_locale_environment()) {
                Ok(()) => {
                    status.set_text(&tr.text("settings-saved"));
                    spawn("note-apply-settings", &[]);
                    spawn("gapplication", &["action", "mx.note.desktop.shell", "reload"]);
                }
                Err(error) => status.set_text(&format!("{}: {error}", tr.text("settings-error"))),
            }
        });
    }

    {
        let controls = controls.clone();
        reset.connect_clicked(move |_| controls.apply(&NoteSettings::default()));
    }

    window.set_child(Some(&root));
    window.present();
}

#[derive(Clone)]
struct AppearanceControls {
    page: gtk::Widget,
    appearance: gtk::DropDown,
    accent: gtk::DropDown,
    language: gtk::DropDown,
    dock_size: gtk::Scale,
    panel_opacity: gtk::Scale,
    dock_opacity: gtk::Scale,
    animations: gtk::Switch,
    natural_scroll: gtk::Switch,
    workspaces: gtk::SpinButton,
}

impl AppearanceControls {
    fn apply(&self, settings: &NoteSettings) {
        self.appearance.set_selected(match settings.appearance {
            AppearanceMode::Auto => 0,
            AppearanceMode::Light => 1,
            AppearanceMode::Dark => 2,
        });
        let accents = ["blue", "purple", "pink", "red", "orange", "green", "teal"];
        let accent_index = accents.iter().position(|value| *value == settings.accent).unwrap_or(0);
        self.accent.set_selected(accent_index as u32);
        let locales = ["es-MX", "en-US", "pt-BR", "fr-FR", "de-DE"];
        let language_index = locales.iter().position(|value| *value == settings.language).unwrap_or(0);
        self.language.set_selected(language_index as u32);
        self.dock_size.set_value(settings.dock_size as f64);
        self.panel_opacity.set_value(settings.panel_opacity);
        self.dock_opacity.set_value(settings.dock_opacity);
        self.animations.set_active(settings.animations);
        self.natural_scroll.set_active(settings.natural_scroll);
        self.workspaces.set_value(settings.workspaces as f64);
    }
}

fn build_appearance_page(stack: &gtk::Stack, tr: &Translator, settings: &NoteSettings) -> AppearanceControls {
    let page = page_container(&tr.text("appearance"), &tr.text("appearance-description"));
    let group = settings_group();

    let appearance = dropdown(&[&tr.text("automatic"), &tr.text("light"), &tr.text("dark")]);
    group.append(&setting_row(&tr.text("color-mode"), &tr.text("color-mode-description"), &appearance));

    let accent = dropdown(&["blue", "purple", "pink", "red", "orange", "green", "teal"]);
    group.append(&setting_row(&tr.text("accent-color"), &tr.text("accent-color-description"), &accent));

    let language = dropdown(&["Español (México)", "English (US)", "Português (Brasil)", "Français", "Deutsch"]);
    group.append(&setting_row(&tr.text("interface-language"), &tr.text("language-relogin"), &language));

    let dock_size = scale(38.0, 72.0, settings.dock_size as f64);
    group.append(&setting_row(&tr.text("dock-size"), &tr.text("dock-size-description"), &dock_size));

    let panel_opacity = scale(0.35, 1.0, settings.panel_opacity);
    panel_opacity.set_digits(2);
    group.append(&setting_row(&tr.text("panel-opacity"), &tr.text("opacity-description"), &panel_opacity));

    let dock_opacity = scale(0.35, 1.0, settings.dock_opacity);
    dock_opacity.set_digits(2);
    group.append(&setting_row(&tr.text("dock-opacity"), &tr.text("opacity-description"), &dock_opacity));

    let animations = gtk::Switch::new();
    group.append(&setting_row(&tr.text("animations"), &tr.text("animations-description"), &animations));

    let natural_scroll = gtk::Switch::new();
    group.append(&setting_row(&tr.text("natural-scroll"), &tr.text("natural-scroll-description"), &natural_scroll));

    let workspaces = gtk::SpinButton::with_range(1.0, 9.0, 1.0);
    group.append(&setting_row(&tr.text("workspaces"), &tr.text("workspaces-description"), &workspaces));

    if let Ok(page_box) = page.clone().downcast::<gtk::Box>() {
        page_box.append(&group);
    }
    stack.add_named(&page, Some("appearance"));

    let controls = AppearanceControls {
        page,
        appearance,
        accent,
        language,
        dock_size,
        panel_opacity,
        dock_opacity,
        animations,
        natural_scroll,
        workspaces,
    };
    controls.apply(settings);
    controls
}

fn build_language_page(tr: &Translator) -> gtk::Widget {
    let page = page_container(&tr.text("language-region"), &tr.text("language-description"));
    let group = settings_group();
    group.append(&info_row(&tr.text("current-language"), tr.locale()));
    group.append(&info_row(&tr.text("locale-file"), "~/.config/environment.d/90-note-locale.conf"));
    group.append(&info_row(&tr.text("available-languages"), "es-MX · en-US · pt-BR · fr-FR · de-DE"));
    if let Ok(page_box) = page.clone().downcast::<gtk::Box>() { page_box.append(&group); }
    page
}

fn build_launcher_page(tr: &Translator, icon: &str, description_key: &str, commands: &'static [&'static str]) -> gtk::Widget {
    let page = build_info_page(tr, icon, description_key);
    if let Ok(page_box) = page.clone().downcast::<gtk::Box>() {
        let button = gtk::Button::with_label(&tr.text("open-configuration"));
        button.add_css_class("suggested-action");
        button.set_halign(gtk::Align::Start);
        button.connect_clicked(move |_| spawn_first(commands));
        page_box.append(&button);
    }
    page
}

fn build_info_page(tr: &Translator, icon: &str, description_key: &str) -> gtk::Widget {
    let page = page_container(&tr.text(description_key.trim_end_matches("-description")), &tr.text(description_key));
    if let Ok(page_box) = page.clone().downcast::<gtk::Box>() {
        let image = gtk::Image::from_icon_name(icon);
        image.set_pixel_size(96);
        image.add_css_class("page-hero-icon");
        page_box.append(&image);
    }
    page
}

fn build_about_page(tr: &Translator) -> gtk::Widget {
    let page = page_container(&tr.text("about"), &tr.text("about-description"));
    if let Ok(page_box) = page.clone().downcast::<gtk::Box>() {
        let logo = gtk::Image::from_icon_name("note-desktop-symbolic");
        logo.set_pixel_size(96);
        page_box.append(&logo);
        let version = gtk::Label::new(Some(&format!("Note Desktop {}", env!("CARGO_PKG_VERSION"))));
        version.add_css_class("about-version");
        page_box.append(&version);
        let doctor = gtk::Button::with_label(&tr.text("run-diagnostics"));
        doctor.connect_clicked(|_| spawn("note-terminal", &["-e", "note-doctor"]));
        page_box.append(&doctor);
    }
    page
}

fn add_page(sidebar: &gtk::ListBox, stack: &gtk::Stack, name: &str, icon: &str, title: &str, page: gtk::Widget) {
    if stack.child_by_name(name).is_none() {
        stack.add_named(&page, Some(name));
    }
    let row = gtk::ListBoxRow::new();
    row.set_widget_name(name);
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(14);
    content.set_margin_end(14);
    let image = gtk::Image::from_icon_name(icon);
    image.set_pixel_size(20);
    content.append(&image);
    let label = gtk::Label::new(Some(title));
    label.set_xalign(0.0);
    content.append(&label);
    row.set_child(Some(&content));
    sidebar.append(&row);
}

fn page_container(title: &str, description: &str) -> gtk::Widget {
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 12);
    outer.add_css_class("settings-page");
    outer.set_margin_top(32);
    outer.set_margin_bottom(32);
    outer.set_margin_start(42);
    outer.set_margin_end(42);
    let title_label = gtk::Label::new(Some(title));
    title_label.add_css_class("page-title");
    title_label.set_xalign(0.0);
    outer.append(&title_label);
    let description_label = gtk::Label::new(Some(description));
    description_label.add_css_class("page-description");
    description_label.set_xalign(0.0);
    description_label.set_wrap(true);
    outer.append(&description_label);
    outer.upcast()
}

fn settings_group() -> gtk::Box {
    let group = gtk::Box::new(gtk::Orientation::Vertical, 0);
    group.add_css_class("settings-group");
    group
}

fn setting_row<T: IsA<gtk::Widget>>(title: &str, description: &str, control: &T) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 18);
    row.add_css_class("setting-row");
    let text = gtk::Box::new(gtk::Orientation::Vertical, 3);
    text.set_hexpand(true);
    let title = gtk::Label::new(Some(title));
    title.set_xalign(0.0);
    title.add_css_class("setting-title");
    text.append(&title);
    let description = gtk::Label::new(Some(description));
    description.set_xalign(0.0);
    description.set_wrap(true);
    description.add_css_class("setting-description");
    text.append(&description);
    row.append(&text);
    control.set_valign(gtk::Align::Center);
    row.append(control);
    row
}

fn info_row(title: &str, value: &str) -> gtk::Box {
    let value_label = gtk::Label::new(Some(value));
    value_label.add_css_class("setting-description");
    setting_row(title, "", &value_label)
}

fn dropdown(values: &[&str]) -> gtk::DropDown {
    let list = gtk::StringList::new(values);
    gtk::DropDown::builder().model(&list).build()
}

fn scale(min: f64, max: f64, value: f64) -> gtk::Scale {
    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, min, max, (max - min) / 100.0);
    scale.set_value(value);
    scale.set_size_request(230, -1);
    scale
}

fn selected_string(dropdown: &gtk::DropDown) -> Option<String> {
    dropdown
        .selected_item()?
        .downcast::<gtk::StringObject>()
        .ok()
        .map(|value| value.string().to_string())
}

fn language_code(index: u32) -> &'static str {
    match index {
        1 => "en-US",
        2 => "pt-BR",
        3 => "fr-FR",
        4 => "de-DE",
        _ => "es-MX",
    }
}

fn install_css() {
    let provider = gtk::CssProvider::new();
    let css = std::fs::read_to_string("/usr/share/note-desktop/style/settings.css")
        .unwrap_or_else(|_| FALLBACK_CSS.to_owned());
    provider.load_from_data(&css);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}

fn spawn(command: &str, args: &[&str]) {
    let _ = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn spawn_first(commands: &[&str]) {
    for command in commands {
        let exists = Command::new("sh")
            .args(["-lc", &format!("command -v {command} >/dev/null 2>&1")])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if exists {
            spawn(command, &[]);
            break;
        }
    }
}
