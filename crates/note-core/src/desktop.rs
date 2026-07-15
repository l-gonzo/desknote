use std::{
    collections::{HashMap, HashSet},
    env,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug, Clone)]
pub struct DesktopApp {
    pub desktop_id: String,
    pub name: String,
    pub generic_name: Option<String>,
    pub icon: String,
    pub exec: String,
    pub app_id: String,
    pub terminal: bool,
    pub categories: Vec<String>,
}

impl DesktopApp {
    pub fn launch(&self) -> std::io::Result<()> {
        let command = strip_field_codes(&self.exec);
        if command.trim().is_empty() {
            return Ok(());
        }
        let final_command = if self.terminal {
            format!("note-terminal -e {}", shell_quote(&command))
        } else {
            command
        };
        Command::new("sh")
            .arg("-lc")
            .arg(final_command)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(())
    }
}

pub fn discover_apps(locale: &str) -> Vec<DesktopApp> {
    let mut dirs = Vec::new();
    if let Some(home) = env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }
    dirs.extend([
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from("/usr/share/applications"),
    ]);

    let mut seen = HashSet::new();
    let mut apps = Vec::new();
    for dir in dirs {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("desktop") {
                continue;
            }
            let Some(id) = path.file_name().and_then(|name| name.to_str()).map(str::to_owned) else {
                continue;
            };
            if !seen.insert(id.clone()) {
                continue;
            }
            if let Some(app) = parse_desktop_file(&path, &id, locale) {
                apps.push(app);
            }
        }
    }
    apps.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    apps
}

fn parse_desktop_file(path: &Path, desktop_id: &str, locale: &str) -> Option<DesktopApp> {
    let content = fs::read_to_string(path).ok()?;
    let mut in_entry = false;
    let mut values = HashMap::<String, String>::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            values.entry(key.to_owned()).or_insert_with(|| value.to_owned());
        }
    }

    if values.get("Type").map(String::as_str).unwrap_or("Application") != "Application"
        || bool_value(values.get("NoDisplay"))
        || bool_value(values.get("Hidden"))
    {
        return None;
    }

    let name = localized(&values, "Name", locale)?;
    let exec = values.get("Exec")?.to_owned();
    let desktop_base = desktop_id.trim_end_matches(".desktop").to_owned();
    let app_id = values
        .get("StartupWMClass")
        .cloned()
        .unwrap_or_else(|| desktop_base.clone());
    Some(DesktopApp {
        desktop_id: desktop_id.to_owned(),
        name,
        generic_name: localized(&values, "GenericName", locale),
        icon: values
            .get("Icon")
            .cloned()
            .unwrap_or_else(|| "application-x-executable-symbolic".into()),
        exec,
        app_id,
        terminal: bool_value(values.get("Terminal")),
        categories: values
            .get("Categories")
            .map(|value| value.split(';').filter(|item| !item.is_empty()).map(str::to_owned).collect())
            .unwrap_or_default(),
    })
}

fn localized(values: &HashMap<String, String>, key: &str, locale: &str) -> Option<String> {
    let full = format!("{key}[{locale}]");
    let short_locale = locale.split(['-', '_']).next().unwrap_or(locale);
    let short = format!("{key}[{short_locale}]");
    values
        .get(&full)
        .or_else(|| values.get(&short))
        .or_else(|| values.get(key))
        .cloned()
}

fn bool_value(value: Option<&String>) -> bool {
    value
        .map(|value| value.eq_ignore_ascii_case("true") || value == "1")
        .unwrap_or(false)
}

fn strip_field_codes(exec: &str) -> String {
    let mut output = Vec::new();
    for token in shell_words(exec) {
        if token.starts_with('%') && token.len() == 2 {
            continue;
        }
        output.push(token.replace("%%", "%"));
    }
    output.join(" ")
}

fn shell_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    for character in input.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        match quote {
            Some(active) if character == active => quote = None,
            Some(_) => current.push(character),
            None if character == '\'' || character == '"' => quote = Some(character),
            None if character.is_whitespace() => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            None => current.push(character),
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
