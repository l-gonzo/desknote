Name:           note-desktop
Version:        0.2.0
Release:        1%{?dist}
Summary:        Experimental Wayland desktop with a GNOME-like workflow
License:        GPL-3.0-or-later
URL:            https://example.invalid/note-desktop
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  pkgconfig(gtk4)
BuildRequires:  pkgconfig(gtk4-layer-shell-0)
Requires:       labwc
Requires:       xorg-x11-server-Xwayland
Requires:       greetd
Requires:       cage
Requires:       gtkgreet
Requires:       gtk4
Requires:       gtk4-layer-shell
Requires:       pipewire
Requires:       wireplumber
Requires:       NetworkManager
Requires:       bluez
Requires:       upower
Requires:       polkit
Requires:       mako
Requires:       swaybg
Requires:       foot
Requires:       thunar
Requires:       grim
Requires:       slurp
Requires:       wl-clipboard
Requires:       xdg-desktop-portal
Requires:       xdg-desktop-portal-wlr
Requires:       xdg-desktop-portal-gtk

%description
Note Desktop provides a Rust/GTK4 panel, dock, overview, control center,
settings application and Wayland session. This alpha uses labwc as its
temporary compositor.

%prep
%autosetup

%build
cargo build --release --locked

%install
DESTDIR=%{buildroot} ./scripts/install-files.sh %{buildroot}

%files
%license LICENSE
%doc README.md docs/
/usr/bin/note-*
/usr/libexec/note-desktop/
/usr/lib/systemd/user/note-*
/usr/share/applications/note-settings.desktop
/usr/share/icons/hicolor/scalable/apps/note-desktop-symbolic.svg
/usr/share/note-desktop/
/usr/share/themes/Note/
/usr/share/wayland-sessions/note.desktop
/usr/share/xdg-desktop-portal/note-portals.conf
%config(noreplace) /etc/xdg/note/
%config(noreplace) /etc/greetd/note-config.toml
%config(noreplace) /etc/greetd/note-greet.css
%config(noreplace) /etc/note-desktop/gpu.conf.example

%changelog
* Tue Jul 14 2026 Note Desktop Project <maintainers@invalid.example> - 0.2.0-1
- Initial alpha package
