use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use chrono::Local;
use gtk::{gdk, glib, prelude::*};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

const APP_ID: &str = "mx.note.desktop.shell";

fn main() -> glib::ExitCode {
    if std::env::args().any(|arg| arg == "--version" || arg == "-V") {
        println!("note-shell {}", env!("CARGO_PKG_VERSION"));
        return glib::ExitCode::SUCCESS;
    }

    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| install_css());
    app.connect_activate(build_shell);
    app.run()
}

fn build_shell(app: &gtk::Application) {
    let Some(display) = gdk::Display::default() else {
        eprintln!("note-shell: no se encontró un display Wayland");
        return;
    };

    if !gtk4_layer_shell::is_supported() {
        eprintln!("note-shell: el compositor no implementa wlr-layer-shell");
        return;
    }

    let monitors = display.monitors();
    if monitors.n_items() == 0 {
        create_panel(app, None);
        create_dock(app, None);
        return;
    }

    for index in 0..monitors.n_items() {
        let Some(item) = monitors.item(index) else {
            continue;
        };
        let Ok(monitor) = item.downcast::<gdk::Monitor>() else {
            continue;
        };
        create_panel(app, Some(&monitor));
        create_dock(app, Some(&monitor));
    }
}

fn create_panel(app: &gtk::Application, monitor: Option<&gdk::Monitor>) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Note Panel")
        .decorated(false)
        .resizable(false)
        .build();

    configure_layer_window(&window, monitor, Layer::Top, "note-panel");
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.auto_exclusive_zone_enable();

    let left = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    left.set_margin_start(10);

    let note_button = gtk::Button::with_label("◆ Note");
    note_button.add_css_class("note-menu");
    note_button.set_tooltip_text(Some("Abrir aplicaciones (Super + Espacio)"));
    note_button.connect_clicked(|_| spawn("/usr/local/bin/note-launcher", &[]));
    left.append(&note_button);

    let active_label = gtk::Label::new(Some("Escritorio"));
    active_label.add_css_class("panel-muted");
    left.append(&active_label);

    let clock = gtk::Label::new(None);
    clock.add_css_class("panel-clock");
    update_clock(&clock);
    {
        let clock = clock.clone();
        glib::timeout_add_seconds_local(1, move || {
            update_clock(&clock);
            glib::ControlFlow::Continue
        });
    }

    let right = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    right.set_margin_end(8);

    let battery = gtk::Label::new(None);
    battery.add_css_class("panel-status");
    update_battery(&battery);
    {
        let battery = battery.clone();
        glib::timeout_add_seconds_local(30, move || {
            update_battery(&battery);
            glib::ControlFlow::Continue
        });
    }
    right.append(&battery);

    right.append(&status_button(
        "network-wireless-signal-excellent-symbolic",
        "Red",
        "/usr/local/bin/note-network",
    ));
    right.append(&status_button(
        "audio-volume-high-symbolic",
        "Audio",
        "/usr/local/bin/note-audio",
    ));
    right.append(&power_menu());

    let center = gtk::CenterBox::new();
    center.add_css_class("panel");
    center.set_start_widget(Some(&left));
    center.set_center_widget(Some(&clock));
    center.set_end_widget(Some(&right));
    window.set_child(Some(&center));
    window.present();
}

fn create_dock(app: &gtk::Application, monitor: Option<&gdk::Monitor>) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Note Dock")
        .decorated(false)
        .resizable(false)
        .build();

    configure_layer_window(&window, monitor, Layer::Top, "note-dock");
    window.set_anchor(Edge::Bottom, true);
    window.set_margin(Edge::Bottom, 12);
    window.set_keyboard_mode(KeyboardMode::None);
    window.set_exclusive_zone(0);

    let dock = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    dock.add_css_class("dock");

    dock.append(&dock_button(
        "view-app-grid-symbolic",
        "Aplicaciones",
        "/usr/local/bin/note-launcher",
    ));
    dock.append(&separator());
    dock.append(&dock_button(
        "utilities-terminal-symbolic",
        "Terminal",
        "/usr/local/bin/note-terminal",
    ));
    dock.append(&dock_button(
        "system-file-manager-symbolic",
        "Archivos",
        "/usr/local/bin/note-files",
    ));
    dock.append(&dock_button(
        "web-browser-symbolic",
        "Navegador",
        "/usr/local/bin/note-browser",
    ));
    dock.append(&dock_button(
        "accessories-text-editor-symbolic",
        "Editor",
        "/usr/local/bin/note-editor",
    ));
    dock.append(&separator());
    dock.append(&dock_button(
        "system-lock-screen-symbolic",
        "Bloquear",
        "/usr/local/bin/note-lock",
    ));

    window.set_child(Some(&dock));
    window.present();
}

fn configure_layer_window(
    window: &gtk::ApplicationWindow,
    monitor: Option<&gdk::Monitor>,
    layer: Layer,
    namespace: &str,
) {
    window.init_layer_shell();
    window.set_layer(layer);
    window.set_namespace(Some(namespace));
    window.set_monitor(monitor);
}

fn dock_button(icon: &str, tooltip: &str, command: &'static str) -> gtk::Button {
    let image = gtk::Image::from_icon_name(icon);
    image.set_pixel_size(28);

    let button = gtk::Button::builder().child(&image).build();
    button.add_css_class("dock-item");
    button.set_tooltip_text(Some(tooltip));
    button.connect_clicked(move |_| spawn(command, &[]));
    button
}

fn status_button(icon: &str, tooltip: &str, command: &'static str) -> gtk::Button {
    let image = gtk::Image::from_icon_name(icon);
    image.set_pixel_size(16);

    let button = gtk::Button::builder().child(&image).build();
    button.add_css_class("status-button");
    button.set_tooltip_text(Some(tooltip));
    button.connect_clicked(move |_| spawn(command, &[]));
    button
}

fn power_menu() -> gtk::MenuButton {
    let menu_button = gtk::MenuButton::builder()
        .icon_name("system-shutdown-symbolic")
        .tooltip_text("Sesión y energía")
        .build();
    menu_button.add_css_class("status-button");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);

    content.append(&power_action(
        "Bloquear",
        "system-lock-screen-symbolic",
        "/usr/local/bin/note-lock",
    ));
    content.append(&power_action(
        "Cerrar sesión",
        "system-log-out-symbolic",
        "/usr/local/bin/note-logout",
    ));
    content.append(&power_action(
        "Reiniciar",
        "system-reboot-symbolic",
        "/usr/local/bin/note-reboot",
    ));
    content.append(&power_action(
        "Apagar",
        "system-shutdown-symbolic",
        "/usr/local/bin/note-poweroff",
    ));

    let popover = gtk::Popover::new();
    popover.add_css_class("power-popover");
    popover.set_child(Some(&content));
    menu_button.set_popover(Some(&popover));
    menu_button
}

fn power_action(label: &str, icon: &str, command: &'static str) -> gtk::Button {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.append(&gtk::Image::from_icon_name(icon));
    row.append(&gtk::Label::new(Some(label)));

    let button = gtk::Button::builder().child(&row).build();
    button.add_css_class("power-action");
    button.connect_clicked(move |_| spawn(command, &[]));
    button
}

fn separator() -> gtk::Separator {
    let separator = gtk::Separator::new(gtk::Orientation::Vertical);
    separator.add_css_class("dock-separator");
    separator
}

fn update_clock(label: &gtk::Label) {
    label.set_text(&Local::now().format("%a %d %b   %H:%M").to_string());
}

fn update_battery(label: &gtk::Label) {
    let text = battery_capacity()
        .map(|capacity| format!("{}%", capacity))
        .unwrap_or_default();
    label.set_text(&text);
    label.set_visible(!text.is_empty());
}

fn battery_capacity() -> Option<u8> {
    let entries = fs::read_dir("/sys/class/power_supply").ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with("BAT") {
            continue;
        }
        let capacity = fs::read_to_string(entry.path().join("capacity")).ok()?;
        if let Ok(value) = capacity.trim().parse::<u8>() {
            return Some(value);
        }
    }
    None
}

fn spawn(command: &str, args: &[&str]) {
    if !Path::new(command).exists() {
        eprintln!("note-shell: no existe {command}");
        return;
    }

    if let Err(error) = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        eprintln!("note-shell: no se pudo ejecutar {command}: {error}");
    }
}

fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        r#"
        * {
            font-family: Inter, Cantarell, Sans;
            font-size: 13px;
        }

        window { background: transparent; }

        .panel {
            min-height: 35px;
            background: rgba(18, 20, 26, 0.93);
            color: #f8fafc;
            border-bottom: 1px solid rgba(255, 255, 255, 0.09);
        }

        .note-menu,
        .status-button {
            min-height: 26px;
            min-width: 30px;
            padding: 0 9px;
            border: 0;
            border-radius: 9px;
            background: transparent;
            color: #f8fafc;
        }

        .note-menu:hover,
        .status-button:hover {
            background: rgba(255, 255, 255, 0.11);
        }

        .panel-clock { font-weight: 650; }
        .panel-muted { color: rgba(248, 250, 252, 0.66); }
        .panel-status { color: rgba(248, 250, 252, 0.82); margin-right: 4px; }

        .dock {
            padding: 8px 10px;
            background: rgba(21, 24, 31, 0.88);
            border: 1px solid rgba(255, 255, 255, 0.13);
            border-radius: 18px;
            box-shadow: 0 10px 34px rgba(0, 0, 0, 0.42);
        }

        .dock-item {
            min-width: 46px;
            min-height: 46px;
            padding: 0;
            border: 0;
            border-radius: 13px;
            background: rgba(255, 255, 255, 0.055);
            color: #f8fafc;
        }

        .dock-item:hover {
            background: rgba(255, 255, 255, 0.15);
        }

        .dock-separator {
            margin: 7px 2px;
            background: rgba(255, 255, 255, 0.15);
        }

        .power-popover contents {
            background: rgba(27, 30, 38, 0.98);
            color: #f8fafc;
            border: 1px solid rgba(255, 255, 255, 0.10);
            border-radius: 14px;
        }

        .power-action {
            min-width: 170px;
            padding: 9px 12px;
            border: 0;
            border-radius: 9px;
            background: transparent;
            color: #f8fafc;
        }

        .power-action:hover { background: rgba(255, 255, 255, 0.11); }
        "#,
    );

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
