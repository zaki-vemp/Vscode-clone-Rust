//! Load the color-theme contributions of real VS Code extension packages.
//! Packages are read in place; no JavaScript is executed or files extracted.
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::theme::Theme;
use crate::vscode_theme::{self, parse_jsonc};

const MAX_JSON: u64 = 4 * 1024 * 1024;

pub struct ExtensionTheme {
    pub id: String,
    pub label: String,
    pub theme: Theme,
}

pub struct Extension {
    pub path: PathBuf,
    pub id: String,
    pub display_name: String,
    pub publisher: String,
    pub version: String,
    pub description: String,
    pub themes: Vec<ExtensionTheme>,
    pub snippets: Vec<crate::snippets::Snippet>,
    pub needs_host: bool,
}

fn read_text(reader: impl Read) -> Result<String> {
    let mut text = String::new();
    reader.take(MAX_JSON + 1).read_to_string(&mut text)?;
    if text.len() as u64 > MAX_JSON {
        bail!("Theme file exceeds 4 MiB");
    }
    Ok(text)
}

pub fn load(path: &Path) -> Result<Extension> {
    let path = path
        .canonicalize()
        .with_context(|| format!("Cannot open {}", path.display()))?;
    let mut extension = if path.is_dir() {
        let root = path.clone();
        from_package(&mut |name| {
            let file = root.join(name).canonicalize()?;
            if !file.starts_with(&root) {
                bail!("Theme path escapes the extension folder");
            }
            read_text(File::open(file)?)
        })?
    } else {
        let mut archive = zip::ZipArchive::new(File::open(&path)?)
            .context("Choose a VSIX file or an unpacked extension folder")?;
        if archive.len() > 50_000 {
            bail!("Extension archive has too many entries");
        }
        from_package(&mut |name| {
            let file = archive.by_name(&format!("extension/{name}"))?;
            if file.size() > MAX_JSON {
                bail!("Theme file exceeds 4 MiB");
            }
            read_text(file)
        })?
    };
    extension.path = path;
    Ok(extension)
}

fn from_package(read: &mut dyn FnMut(&str) -> Result<String>) -> Result<Extension> {
    let manifest = parse_jsonc(&read("package.json")?)?;
    let translations = read("package.nls.json")
        .ok()
        .and_then(|text| parse_jsonc(&text).ok())
        .unwrap_or(Value::Null);
    let localize = |text: &str| {
        text.strip_prefix('%')
            .and_then(|key| key.strip_suffix('%'))
            .and_then(|key| translations[key].as_str())
            .unwrap_or(text)
            .to_string()
    };
    let string = |key: &str| manifest[key].as_str().unwrap_or_default().to_string();
    let name = string("name");
    let publisher = string("publisher");
    if name.is_empty() || publisher.is_empty() {
        bail!("Extension manifest needs name and publisher");
    }
    let id = format!("{publisher}.{name}");
    let needs_host = manifest.get("main").is_some() || manifest.get("browser").is_some();
    let mut themes = Vec::new();
    if let Some(contributions) = manifest["contributes"]["themes"].as_array() {
        if contributions.len() > 256 {
            bail!("Extension contributes too many themes");
        }
        for (index, item) in contributions.iter().enumerate() {
            let file = item["path"]
                .as_str()
                .context("Theme contribution is missing path")?;
            let file = vscode_theme::relative_path("package.json", file)?;
            let light = matches!(item["uiTheme"].as_str(), Some("vs" | "hc-light"));
            let mut theme = vscode_theme::load(&file, light, read)?;
            let label = localize(item["label"].as_str().unwrap_or(&theme.name));
            theme.name = label.clone();
            themes.push(ExtensionTheme {
                id: format!("{id}/{index}"),
                label,
                theme,
            });
        }
    }
    let mut snippets = Vec::new();
    if let Some(contributions) = manifest["contributes"]["snippets"].as_array() {
        for item in contributions.iter().take(256) {
            let file = item["path"]
                .as_str()
                .context("Snippet contribution is missing path")?;
            let file = vscode_theme::relative_path("package.json", file)?;
            let data = parse_jsonc(&read(&file)?)?;
            snippets.extend(crate::snippets::parse(
                &data,
                item["language"].as_str().unwrap_or_default(),
            )?);
        }
    }
    if themes.is_empty() && snippets.is_empty() {
        bail!("This extension requires features that are not supported yet. Only JSON color themes and basic snippets can be loaded; JavaScript extensions need a VS Code extension host.");
    }
    Ok(Extension {
        path: PathBuf::new(),
        id,
        publisher,
        version: string("version"),
        display_name: localize(manifest["displayName"].as_str().unwrap_or(&name)),
        description: string("description"),
        themes,
        snippets,
        needs_host,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_vsix_theme_loads_without_extraction_or_execution() {
        use std::io::{Cursor, Write};
        use zip::write::SimpleFileOptions;
        let dir = std::env::temp_dir().join(format!("oxide-vsix-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.vsix");
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, text) in [
            (
                "extension/package.json",
                r#"{"name":"theme","publisher":"test","displayName":"Test Theme","version":"1.0.0","main":"never-run.js","contributes":{"themes":[{"label":"Ocean","uiTheme":"vs","path":"themes/ocean.json"}]}}"#,
            ),
            (
                "extension/themes/ocean.json",
                r##"{"include":"base.json","colors":{"editor.background":"#fafafa"}}"##,
            ),
            (
                "extension/themes/base.json",
                r##"{"tokenColors":[{"scope":"string","settings":{"foreground":"#123456"}}]}"##,
            ),
        ] {
            zip.start_file(name, SimpleFileOptions::default()).unwrap();
            zip.write_all(text.as_bytes()).unwrap();
        }
        std::fs::write(&path, zip.finish().unwrap().into_inner()).unwrap();
        let ext = load(&path).unwrap();
        assert_eq!(ext.id, "test.theme");
        assert!(ext.needs_host);
        assert_eq!(ext.themes[0].label, "Ocean");
        assert!(!ext.themes[0].theme.dark);
        assert_eq!(
            ext.themes[0].theme.string,
            vscode_theme::color("#123456").unwrap()
        );
        assert!(!dir.join("extension").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn unsupported_extensions_and_escaping_paths_fail() {
        let mut read =
            |_: &str| Ok(r#"{"name":"tool","publisher":"test","main":"index.js"}"#.into());
        assert!(from_package(&mut read)
            .err()
            .unwrap()
            .to_string()
            .contains("extension host"));
        let mut read = |_: &str| {
            Ok(r#"{"name":"bad","publisher":"test","contributes":{"themes":[{"path":"../outside.json"}]}}"#.into())
        };
        assert!(from_package(&mut read).is_err());
    }
}
