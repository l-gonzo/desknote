use chrono::{Local, Timelike};
use gtk::{gdk, gio, glib, prelude::*};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use note_core::{
    DesktopApp, NoteSettings, Translator, discover_apps,
    system::{
        bluetooth_enabled, brightness_percent, focus_toplevel, list_toplevels,
        set_bluetooth, set_brightness, set_volume, set_wifi, volume_percent, wifi_enabled,
    },
};
use std::{cell::RefCell, path::Path, process::{Command, Stdio}, rc::Rc};

const APP_ID: &str = "mx.note.desktop.shell";
const FALLBACK_CSS: &str = include_str!("../../../assets/style/shell.css");

#[derive(Default)]
struct ShellState {
    surfaces: Vec<gtk::ApplicationWindow>,
    overview: Option<gtk::ApplicationWindow>,
    control_center: Option<gtk::ApplicationWindow>,
    settings: NoteSettings,
    apps: Vec<DesktopApp>,
}

fn main() -> glib::ExitCode {
    if std::env::args().any(|arg| arg == "--version" || arg == "-V") {
        println!("note-shell {}", env!("CARGO_PKG_VERSION"));
        return glib::ExitCode::SUCCESS;
    }

    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::DEFAULT_FLAGS)
        .build();
    let state = Rc::new(RefCell::new(ShellState::default()));

    {
        let state = state.clone();
        app.connect_startup(move |app| {
            register_actions(app, state.clone());
        });
    }
    {
        let state = state.clone();
        app.connect_activate(move |app| {
            if state.borrow().surfaces.is_empty() {
                rebuild_shell(app, &state);
            }
        });
    }

    app.run()
}

fn register_actions(app: &gtk::Application, state: Rc<RefCell<ShellState>>) {
    let overview = gio::SimpleAction::new("overview", None);
    {
        let app = app.clone();
        let state = state.clone();
        overview.connect_activate(move |_, _| toggle_overview(&app, &state));
    }
    app.add_action(&overview);

    let control_center = gio::SimpleAction::new("control-center", None);
    {
        let app = app.clone();
        let state = state.clone();
        control_center.connect_activate(move |_, _| toggle_control_center(&app, &state));
    }
    app.add_action(&control_center);

    let reload = gio::SimpleAction::new("reload", None);
    {
        let app = app.clone();
        let state = state.clone();
        reload.connect_activate(move |_, _| rebuild_shell(&app, &state));
    }
    app.add_action(&reload);

    let hide_overview = gio::SimpleAction::new("hide-overview", None);
    {
        let state = state.clone();
        hide_overview.connect_activate(move |_, _| {
            if let Some(window) = state.borrow().overview.as_ref() {
                window.set_visible(false);
            }
        });
    }
    app.add_action(&hide_overview);
}

fn rebuild_shell(app: &gtk::Application, state: &Rc<RefCell<ShellState>>) {
    {
        let mut state = state.borrow_mut();
        for window in state.surfaces.drain(..) {
            window.close();
        }
        if let Some(window) = state.overview.take() {
            window.close();
        }
        if let Some(window) = state.control_center.take() {
            window.close();
        }
        state.settings = NoteSettings::load();
        state.apps = discover_apps(&state.settings.language);
        install_css(&state.settings);
        apply_gtk_preferences(&state.settings);
    }

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
        let panel = create_panel(app, state, None);
        let dock = create_dock(app, state, None);
        state.borrow_mut().surfaces.extend([panel, dock]);
        return;
    }

    for index in 0..monitors.n_items() {
        let Some(item) = monitors.item(index) else { continue };
        let Ok(monitor) = item.downcast::<gdk::Monitor>() else { continue };
        let panel = create_panel(app, state, Some(&monitor));
        let dock = create_dock(app, state, Some(&monitor));
        state.borrow_mut().surfaces.extend([panel, dock]);
    }
}

fn create_panel(
    app: &gtk::Application,
    state: &Rc<RefCell<ShellState>>,
    monitor: Option<&gdk::Monitor>,
) -> gtk::ApplicationWindow {
    let settings = state.borrow().settings.clone();
    let tr = Translator::new(settings.language.clone());
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

    let left = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    left.set_margin_start(8);

    let note_button = gtk::Button::builder()
        .label("●")
        .tooltip_text(tr.text("overview"))
        .build();
    note_button.add_css_class("note-logo");
    {
        let app = app.clone();
        note_button.connect_clicked(move |_| app.activate_action("overview", None));
    }
    left.append(&note_button);

    let active_app = gtk::Label::new(Some(&tr.text("desktop")));
    active_app.add_css_class("active-application");
    active_app.set_xalign(0.0);
    left.append(&active_app);
    update_active_application(&active_app, &tr);
    {
        let active_app = active_app.clone();
        let tr = tr.clone();
        glib::timeout_add_seconds_local(1, move || {
            update_active_application(&active_app, &tr);
            glib::ControlFlow::Continue
        });
    }

    let clock_button = gtk::MenuButton::new();
    clock_button.add_css_class("clock-button");
    let clock = gtk::Label::new(None);
    clock.add_css_class("panel-clock");
    update_clock(&clock);
    clock_button.set_child(Some(&clock));
    clock_button.set_popover(Some(&calendar_popover(&tr)));
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

    let status_button = gtk::Button::new();
    status_button.add_css_class("status-cluster");
    status_button.set_tooltip_text(Some(&tr.text("control-center")));
    let status_box = gtk::Box::new(gtk::Orientation::Horizontal, 7);
    for icon in [
        "network-wireless-signal-excellent-symbolic",
        "audio-volume-high-symbolic",
        "battery-good-symbolic",
    ] {
        let image = gtk::Image::from_icon_name(icon);
        image.set_pixel_size(15);
        status_box.append(&image);
    }
    status_button.set_child(Some(&status_box));
    {
        let app = app.clone();
        status_button.connect_clicked(move |_| app.activate_action("control-center", None));
    }
    right.append(&status_button);

    let center = gtk::CenterBox::new();
    center.add_css_class("panel");
    center.set_start_widget(Some(&left));
    center.set_center_widget(Some(&clock_button));
    center.set_end_widget(Some(&right));
    window.set_child(Some(&center));
    window.present();
    window
}

fn create_dock(
    app: &gtk::Application,
    state: &Rc<RefCell<ShellState>>,
    monitor: Option<&gdk::Monitor>,
) -> gtk::ApplicationWindow {
    let settings = state.borrow().settings.clone();
    let apps = state.borrow().apps.clone();
    let tr = Translator::new(settings.language.clone());
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Note Dock")
        .decorated(false)
        .resizable(false)
        .build();
    configure_layer_window(&window, monitor, Layer::Top, "note-dock");
    window.set_anchor(Edge::Bottom, true);
    window.set_margin(Edge::Bottom, 14);
    window.set_keyboard_mode(KeyboardMode::None);
    window.set_exclusive_zone(0);

    let dock = gtk::Box::new(gtk::Orientation::Horizontal, 7);
    dock.add_css_class("dock");

    let overview = dock_action_button(
        "view-app-grid-symbolic",
        &tr.text("overview"),
        settings.dock_size,
    );
    {
        let app = app.clone();
        overview.connect_clicked(move |_| app.activate_action("overview", None));
    }
    dock.append(&overview);
    dock.append(&dock_separator());

    let mut indicators: Vec<(String, gtk::Label)> = Vec::new();
    for favorite in &settings.favorites {
        let Some(desktop_app) = apps.iter().find(|candidate| &candidate.desktop_id == favorite).cloned() else {
            continue;
        };
        let (button, indicator) = dock_app_button(&desktop_app, settings.dock_size);
        indicators.push((desktop_app.app_id.clone(), indicator));
        dock.append(&button);
    }

    dock.append(&dock_separator());
    let settings_button = dock_action_button(
        "preferences-system-symbolic",
        &tr.text("settings"),
        settings.dock_size,
    );
    settings_button.connect_clicked(|_| spawn("note-settings", &[]));
    dock.append(&settings_button);

    update_running_indicators(&indicators);
    glib::timeout_add_seconds_local(2, move || {
        update_running_indicators(&indicators);
        glib::ControlFlow::Continue
    });

    window.set_child(Some(&dock));
    window.present();
    window
}

fn dock_app_button(app: &DesktopApp, size: i32) -> (gtk::Button, gtk::Label) {
    let icon = gtk::Image::from_icon_name(&app.icon);
    icon.set_pixel_size((size - 16).max(24));
    let indicator = gtk::Label::new(Some("•"));
    indicator.add_css_class("running-indicator");
    indicator.set_opacity(0.0);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.set_halign(gtk::Align::Center);
    content.append(&icon);
    content.append(&indicator);

    let button = gtk::Button::builder().child(&content).build();
    button.add_css_class("dock-item");
    button.set_tooltip_text(Some(&app.name));
    button.set_size_request(size, size + 6);

    let base_icon_size = (size - 16).max(24);
    let motion = gtk::EventControllerMotion::new();
    {
        let icon = icon.clone();
        let button = button.clone();
        motion.connect_enter(move |_, _, _| {
            icon.set_pixel_size(base_icon_size + 8);
            button.add_css_class("dock-item-hover");
        });
    }
    {
        let icon = icon.clone();
        let button = button.clone();
        motion.connect_leave(move |_| {
            icon.set_pixel_size(base_icon_size);
            button.remove_css_class("dock-item-hover");
        });
    }
    button.add_controller(motion);

    let app_info = app.clone();
    button.connect_clicked(move |_| {
        let open = list_toplevels(None);
        if let Some(toplevel) = open.into_iter().find(|toplevel| {
            toplevel.app_id.eq_ignore_ascii_case(&app_info.app_id)
                || toplevel.app_id.eq_ignore_ascii_case(app_info.desktop_id.trim_end_matches(".desktop"))
        }) {
            let _ = focus_toplevel(&toplevel);
        } else if let Err(error) = app_info.launch() {
            eprintln!("note-shell: no se pudo abrir {}: {error}", app_info.name);
        }
    });
    (button, indicator)
}

fn dock_action_button(icon_name: &str, tooltip: &str, size: i32) -> gtk::Button {
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_pixel_size((size - 18).max(24));
    let button = gtk::Button::builder().child(&icon).build();
    button.add_css_class("dock-item");
    button.set_tooltip_text(Some(tooltip));
    button.set_size_request(size, size + 6);
    button
}

fn dock_separator() -> gtk::Separator {
    let separator = gtk::Separator::new(gtk::Orientation::Vertical);
    separator.add_css_class("dock-separator");
    separator
}

fn toggle_overview(app: &gtk::Application, state: &Rc<RefCell<ShellState>>) {
    if let Some(window) = state.borrow().overview.as_ref() {
        if window.is_visible() {
            window.set_visible(false);
        } else {
            refresh_overview(window, state);
            window.present();
        }
        return;
    }

    let window = create_overview(app, state);
    window.present();
    state.borrow_mut().overview = Some(window);
}

fn create_overview(app: &gtk::Application, state: &Rc<RefCell<ShellState>>) -> gtk::ApplicationWindow {
    let settings = state.borrow().settings.clone();
    let tr = Translator::new(settings.language.clone());
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Note Overview")
        .decorated(false)
        .build();
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("note-overview"));
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(edge, true);
    }
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 22);
    root.add_css_class("overview");
    root.set_margin_top(56);
    root.set_margin_bottom(82);
    root.set_margin_start(72);
    root.set_margin_end(72);

    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some(&tr.text("search-placeholder")));
    search.add_css_class("overview-search");
    search.set_hexpand(true);
    root.append(&search);

    let content = gtk::Paned::new(gtk::Orientation::Horizontal);
    content.set_wide_handle(true);
    content.set_resize_start_child(false);
    content.set_shrink_start_child(false);

    let windows_box = gtk::Box::new(gtk::Orientation::Vertical, 9);
    windows_box.set_size_request(300, -1);
    let windows_title = gtk::Label::new(Some(&tr.text("open-windows")));
    windows_title.add_css_class("section-title");
    windows_title.set_xalign(0.0);
    windows_box.append(&windows_title);
    let windows_list = gtk::Box::new(gtk::Orientation::Vertical, 6);
    windows_list.set_widget_name("note-overview-window-list");
    windows_box.append(&windows_list);
    content.set_start_child(Some(&windows_box));

    let apps_box = gtk::Box::new(gtk::Orientation::Vertical, 9);
    let apps_title = gtk::Label::new(Some(&tr.text("applications")));
    apps_title.add_css_class("section-title");
    apps_title.set_xalign(0.0);
    apps_box.append(&apps_title);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    let grid = gtk::Grid::new();
    grid.set_column_spacing(14);
    grid.set_row_spacing(14);
    grid.set_widget_name("note-overview-app-grid");
    scroll.set_child(Some(&grid));
    apps_box.append(&scroll);
    content.set_end_child(Some(&apps_box));
    content.set_position(330);
    root.append(&content);

    let workspaces = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    workspaces.set_halign(gtk::Align::Center);
    for index in 1..=settings.workspaces.max(1) {
        let button = gtk::Button::with_label(&index.to_string());
        button.add_css_class("workspace-pill");
        button.connect_clicked(move |_| {
            spawn("note-workspace", &[&index.to_string()]);
        });
        workspaces.append(&button);
    }
    root.append(&workspaces);
    window.set_child(Some(&root));

    let apps = state.borrow().apps.clone();
    rebuild_app_grid(&grid, &apps, "", &window);
    rebuild_window_list(&windows_list, &tr, &window);
    {
        let grid = grid.clone();
        let apps = apps.clone();
        let window = window.clone();
        search.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            rebuild_app_grid(&grid, &apps, &query, &window);
        });
    }

    let controller = gtk::EventControllerKey::new();
    {
        let window = window.clone();
        controller.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                window.set_visible(false);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
    }
    window.add_controller(controller);
    window
}

fn refresh_overview(window: &gtk::ApplicationWindow, state: &Rc<RefCell<ShellState>>) {
    let Some(root) = window.child() else { return };
    if let Some(windows_list) = find_widget_by_name(&root, "note-overview-window-list")
        .and_then(|widget| widget.downcast::<gtk::Box>().ok())
    {
        let tr = Translator::new(state.borrow().settings.language.clone());
        rebuild_window_list(&windows_list, &tr, window);
    }
}

fn rebuild_window_list(list: &gtk::Box, tr: &Translator, overview: &gtk::ApplicationWindow) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let windows = list_toplevels(None);
    if windows.is_empty() {
        let empty = gtk::Label::new(Some(&tr.text("no-open-windows")));
        empty.add_css_class("muted");
        empty.set_wrap(true);
        list.append(&empty);
        return;
    }
    for toplevel in windows {
        if toplevel.app_id.contains("note-shell") {
            continue;
        }
        let row = gtk::Button::new();
        row.add_css_class("window-card");
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let icon = gtk::Image::from_icon_name("application-x-executable-symbolic");
        icon.set_pixel_size(28);
        content.append(&icon);
        let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let title = gtk::Label::new(Some(&toplevel.title));
        title.set_xalign(0.0);
        title.set_ellipsize(gtk::pango::EllipsizeMode::End);
        title.add_css_class("window-title");
        text.append(&title);
        let app_id = gtk::Label::new(Some(&toplevel.app_id));
        app_id.set_xalign(0.0);
        app_id.add_css_class("muted");
        text.append(&app_id);
        content.append(&text);
        row.set_child(Some(&content));
        let target = toplevel.clone();
        let overview = overview.clone();
        row.connect_clicked(move |_| {
            let _ = focus_toplevel(&target);
            overview.set_visible(false);
        });
        list.append(&row);
    }
}

fn rebuild_app_grid(
    grid: &gtk::Grid,
    apps: &[DesktopApp],
    query: &str,
    overview: &gtk::ApplicationWindow,
) {
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }
    let query = query.trim().to_lowercase();
    let filtered = apps.iter().filter(|app| {
        query.is_empty()
            || app.name.to_lowercase().contains(&query)
            || app.generic_name.as_deref().unwrap_or_default().to_lowercase().contains(&query)
            || app.categories.iter().any(|category| category.to_lowercase().contains(&query))
    });
    for (position, app) in filtered.take(80).enumerate() {
        let button = gtk::Button::new();
        button.add_css_class("app-tile");
        let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
        content.set_halign(gtk::Align::Center);
        let icon = gtk::Image::from_icon_name(&app.icon);
        icon.set_pixel_size(42);
        content.append(&icon);
        let label = gtk::Label::new(Some(&app.name));
        label.set_max_width_chars(15);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        content.append(&label);
        button.set_child(Some(&content));
        let app_info = app.clone();
        let overview = overview.clone();
        button.connect_clicked(move |_| {
            if let Err(error) = app_info.launch() {
                eprintln!("note-shell: no se pudo abrir {}: {error}", app_info.name);
            }
            overview.set_visible(false);
        });
        grid.attach(&button, (position % 5) as i32, (position / 5) as i32, 1, 1);
    }
}

fn toggle_control_center(app: &gtk::Application, state: &Rc<RefCell<ShellState>>) {
    if let Some(window) = state.borrow().control_center.as_ref() {
        if window.is_visible() { window.set_visible(false); } else { window.present(); }
        return;
    }
    let window = create_control_center(app, &state.borrow().settings);
    window.present();
    state.borrow_mut().control_center = Some(window);
}

fn create_control_center(app: &gtk::Application, settings: &NoteSettings) -> gtk::ApplicationWindow {
    let tr = Translator::new(settings.language.clone());
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Note Control Center")
        .decorated(false)
        .resizable(false)
        .default_width(360)
        .build();
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("note-control-center"));
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Right, true);
    window.set_margin(Edge::Top, 42);
    window.set_margin(Edge::Right, 12);
    window.set_keyboard_mode(KeyboardMode::OnDemand);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 14);
    root.add_css_class("control-center");

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let user_icon = gtk::Image::from_icon_name("avatar-default-symbolic");
    user_icon.set_pixel_size(34);
    header.append(&user_icon);
    let username = std::env::var("USER").unwrap_or_else(|_| tr.text("user"));
    let user_label = gtk::Label::new(Some(&username));
    user_label.add_css_class("control-title");
    user_label.set_hexpand(true);
    user_label.set_xalign(0.0);
    header.append(&user_label);
    let settings_button = icon_button("preferences-system-symbolic", &tr.text("settings"));
    settings_button.connect_clicked(|_| spawn("note-settings", &[]));
    header.append(&settings_button);
    root.append(&header);

    let quick = gtk::Grid::new();
    quick.set_column_homogeneous(true);
    quick.set_column_spacing(10);
    quick.set_row_spacing(10);
    quick.attach(&toggle_tile(
        "network-wireless-symbolic", &tr.text("wifi"), wifi_enabled(), set_wifi,
    ), 0, 0, 1, 1);
    quick.attach(&toggle_tile(
        "bluetooth-active-symbolic", &tr.text("bluetooth"), bluetooth_enabled(), set_bluetooth,
    ), 1, 0, 1, 1);
    let network = action_tile("network-vpn-symbolic", &tr.text("network-settings"));
    network.connect_clicked(|_| spawn("nm-connection-editor", &[]));
    quick.attach(&network, 0, 1, 1, 1);
    let displays = action_tile("video-display-symbolic", &tr.text("display-settings"));
    displays.connect_clicked(|_| spawn_first(&["wdisplays", "arandr"]));
    quick.attach(&displays, 1, 1, 1, 1);
    root.append(&quick);

    root.append(&slider_row(
        "audio-volume-high-symbolic",
        &tr.text("volume"),
        volume_percent() as f64,
        0.0,
        150.0,
        set_volume,
    ));
    if let Some(brightness) = brightness_percent() {
        root.append(&slider_row(
            "display-brightness-symbolic",
            &tr.text("brightness"),
            brightness as f64,
            1.0,
            100.0,
            set_brightness,
        ));
    }

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.set_homogeneous(true);
    for (icon, title, command) in [
        ("system-lock-screen-symbolic", tr.text("lock"), "note-lock"),
        ("system-log-out-symbolic", tr.text("logout"), "note-logout"),
        ("system-reboot-symbolic", tr.text("restart"), "note-reboot"),
        ("system-shutdown-symbolic", tr.text("power-off"), "note-poweroff"),
    ] {
        let button = icon_button(icon, &title);
        button.add_css_class("power-button");
        button.connect_clicked(move |_| spawn(command, &[]));
        actions.append(&button);
    }
    root.append(&actions);
    window.set_child(Some(&root));

    let controller = gtk::EventControllerKey::new();
    {
        let window = window.clone();
        controller.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                window.set_visible(false);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
    }
    window.add_controller(controller);
    window
}

fn toggle_tile(
    icon_name: &str,
    title: &str,
    active: bool,
    callback: fn(bool),
) -> gtk::Box {
    let tile = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    tile.add_css_class("quick-tile");
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_pixel_size(20);
    tile.append(&icon);
    let label = gtk::Label::new(Some(title));
    label.set_hexpand(true);
    label.set_xalign(0.0);
    tile.append(&label);
    let switch = gtk::Switch::new();
    switch.set_active(active);
    switch.connect_state_set(move |_, state| {
        callback(state);
        glib::Propagation::Proceed
    });
    tile.append(&switch);
    tile
}

fn action_tile(icon_name: &str, title: &str) -> gtk::Button {
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_pixel_size(20);
    content.append(&icon);
    let label = gtk::Label::new(Some(title));
    label.set_xalign(0.0);
    content.append(&label);
    let button = gtk::Button::builder().child(&content).build();
    button.add_css_class("quick-tile");
    button
}

fn slider_row(
    icon_name: &str,
    title: &str,
    value: f64,
    min: f64,
    max: f64,
    callback: fn(f64),
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("slider-row");
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_pixel_size(20);
    icon.set_tooltip_text(Some(title));
    row.append(&icon);
    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, min, max, 1.0);
    scale.set_value(value);
    scale.set_hexpand(true);
    scale.set_draw_value(false);
    scale.connect_value_changed(move |scale| callback(scale.value()));
    row.append(&scale);
    row
}

fn calendar_popover(tr: &Translator) -> gtk::Popover {
    let popover = gtk::Popover::new();
    popover.add_css_class("calendar-popover");
    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(14);
    content.set_margin_bottom(14);
    content.set_margin_start(14);
    content.set_margin_end(14);
    let now = Local::now();
    let title = gtk::Label::new(Some(&now.format("%A, %d %B %Y").to_string()));
    title.add_css_class("calendar-title");
    content.append(&title);
    let calendar = gtk::Calendar::new();
    content.append(&calendar);
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.set_homogeneous(true);
    let notifications = gtk::Button::with_label(&tr.text("notifications"));
    notifications.connect_clicked(|_| spawn("makoctl", &["dismiss", "--all"]));
    actions.append(&notifications);
    let settings = gtk::Button::with_label(&tr.text("date-time-settings"));
    settings.connect_clicked(|_| spawn("note-settings", &[]));
    actions.append(&settings);
    content.append(&actions);
    popover.set_child(Some(&content));
    popover
}

fn icon_button(icon_name: &str, tooltip: &str) -> gtk::Button {
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_pixel_size(18);
    let button = gtk::Button::builder().child(&icon).tooltip_text(tooltip).build();
    button.add_css_class("icon-button");
    button
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
    if let Some(monitor) = monitor {
        window.set_monitor(Some(monitor));
    }
}

fn update_clock(label: &gtk::Label) {
    label.set_text(&Local::now().format("%a %d %b   %H:%M").to_string());
}

fn update_active_application(label: &gtk::Label, tr: &Translator) {
    let active = list_toplevels(Some("state:active"));
    if let Some(window) = active.first() {
        label.set_text(&window.app_id);
    } else {
        label.set_text(&tr.text("desktop"));
    }
}

fn update_running_indicators(indicators: &[(String, gtk::Label)]) {
    let open = list_toplevels(None);
    for (app_id, indicator) in indicators {
        let running = open.iter().any(|window| window.app_id.eq_ignore_ascii_case(app_id));
        indicator.set_opacity(if running { 1.0 } else { 0.0 });
    }
}

fn update_battery(label: &gtk::Label) {
    let capacity = std::fs::read_dir("/sys/class/power_supply")
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .find_map(|entry| {
            if !entry.file_name().to_string_lossy().starts_with("BAT") { return None; }
            std::fs::read_to_string(entry.path().join("capacity")).ok()
        })
        .and_then(|value| value.trim().parse::<u8>().ok());
    match capacity {
        Some(value) => { label.set_text(&format!("{value}%")); label.set_visible(true); }
        None => label.set_visible(false),
    }
}

fn install_css(settings: &NoteSettings) {
    let provider = gtk::CssProvider::new();
    let installed = "/usr/share/note-desktop/style/shell.css";
    let base = std::fs::read_to_string(installed).unwrap_or_else(|_| FALLBACK_CSS.to_owned());
    provider.load_from_data(&(settings.css_variables() + &base));
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn apply_gtk_preferences(settings: &NoteSettings) {
    if let Some(gtk_settings) = gtk::Settings::default() {
        let dark = match settings.appearance {
            note_core::AppearanceMode::Light => false,
            note_core::AppearanceMode::Dark => true,
            note_core::AppearanceMode::Auto => {
                let hour = Local::now().hour();
                !(7..19).contains(&hour)
            }
        };
        gtk_settings.set_property("gtk-application-prefer-dark-theme", dark);
        gtk_settings.set_property("gtk-enable-animations", settings.animations);
    }
}

fn find_widget_by_name(root: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
    if root.widget_name() == name { return Some(root.clone()); }
    let mut child = root.first_child();
    while let Some(widget) = child {
        if let Some(found) = find_widget_by_name(&widget, name) { return Some(found); }
        child = widget.next_sibling();
    }
    None
}

fn spawn(command: &str, args: &[&str]) {
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

fn spawn_first(commands: &[&str]) {
    if let Some(command) = commands.iter().find(|command| command_exists(command)) {
        spawn(command, &[]);
    }
}

fn command_exists(command: &str) -> bool {
    if command.contains('/') { return Path::new(command).exists(); }
    Command::new("sh")
        .args(["-lc", &format!("command -v {} >/dev/null 2>&1", command)])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
