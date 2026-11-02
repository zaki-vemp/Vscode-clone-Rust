//! JSON formatting and an expandable native JSON/JSONC tree.
use eframe::egui;
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Default)]
pub struct JsonView {
    pub tree: bool,
    source: Option<(PathBuf, u64)>,
    value: Option<Result<Value, String>>,
}

pub fn format(text: &str, spaces: usize, compact: bool) -> Result<String, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if compact {
        return serde_json::to_string(&value).map_err(|e| e.to_string());
    }
    let mut bytes = Vec::new();
    let indent = vec![b' '; spaces.clamp(1, 8)];
    let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent);
    let mut serializer = serde_json::Serializer::with_formatter(&mut bytes, formatter);
    value
        .serialize(&mut serializer)
        .map_err(|e| e.to_string())?;
    let mut formatted = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    if text.ends_with('\n') {
        formatted.push('\n');
    }
    Ok(formatted)
}

impl JsonView {
    pub fn show(&mut self, ui: &mut egui::Ui, path: PathBuf, revision: u64, text: &str) {
        let key = (path, revision);
        if self.source.as_ref() != Some(&key) {
            self.value = Some(crate::vscode_theme::parse_jsonc(text).map_err(|e| e.to_string()));
            self.source = Some(key);
        }
        egui::ScrollArea::both()
            .id_salt("json-tree")
            .show(ui, |ui| match &self.value {
                Some(Ok(value)) => {
                    node(ui, "$", value, 0);
                }
                Some(Err(error)) => {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
                None => {}
            });
    }
}

fn node(ui: &mut egui::Ui, label: &str, value: &Value, depth: usize) {
    match value {
        Value::Object(object) => {
            egui::CollapsingHeader::new(format!("{label}  {{ {} keys }}", object.len()))
                .default_open(depth == 0)
                .show(ui, |ui| {
                    for (key, value) in object.iter().take(500) {
                        ui.push_id(key, |ui| node(ui, key, value, depth + 1));
                    }
                    if object.len() > 500 {
                        ui.label("First 500 keys shown; use source view for the complete object.");
                    }
                });
        }
        Value::Array(array) => {
            egui::CollapsingHeader::new(format!("{label}  [ {} items ]", array.len()))
                .default_open(depth == 0)
                .show(ui, |ui| {
                    for (index, value) in array.iter().enumerate().take(500) {
                        ui.push_id(index, |ui| node(ui, &index.to_string(), value, depth + 1));
                    }
                    if array.len() > 500 {
                        ui.label("First 500 items shown; use source view for the complete array.");
                    }
                });
        }
        _ => {
            ui.horizontal_wrapped(|ui| {
                ui.strong(format!("{label}:"));
                ui.add(egui::Label::new(value.to_string()).selectable(true));
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formatting_round_trips_and_rejects_invalid_input() {
        let input = "{\"message\":\"a,b\\n中文\",\"items\":[1,true,null]}\n";
        let formatted = format(input, 2, false).unwrap();
        assert!(formatted.contains("\n  \"items\""));
        assert!(formatted.ends_with('\n'));
        assert_eq!(
            serde_json::from_str::<Value>(&formatted).unwrap(),
            serde_json::from_str::<Value>(input).unwrap()
        );
        assert!(!format(input, 4, true).unwrap().contains('\n'));
        assert!(format("{not-json}", 2, false).is_err());
    }
}
