use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    pub locale: String,
    pub ask_shortcut: String,
    pub dictation_shortcut: String,
    pub stop_shortcut: String,
    pub reduced_motion: bool,
    pub dictation_preview: bool,
    pub optional_telemetry: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            locale: "vi".to_owned(),
            ask_shortcut: "CommandOrControl+Shift+Space".to_owned(),
            dictation_shortcut: "CommandOrControl+Shift+D".to_owned(),
            stop_shortcut: "CommandOrControl+Shift+Escape".to_owned(),
            reduced_motion: false,
            dictation_preview: true,
            optional_telemetry: false,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct SettingsPatch {
    pub locale: Option<String>,
    pub ask_shortcut: Option<String>,
    pub dictation_shortcut: Option<String>,
    pub stop_shortcut: Option<String>,
    pub reduced_motion: Option<bool>,
    pub dictation_preview: Option<bool>,
    pub optional_telemetry: Option<bool>,
}

impl AppSettings {
    pub fn apply(&mut self, patch: SettingsPatch) -> Result<(), &'static str> {
        if let Some(locale) = patch.locale {
            if locale != "vi" && locale != "en" {
                return Err("unsupported locale");
            }
            self.locale = locale;
        }
        let mut shortcuts = [
            patch.ask_shortcut.as_deref().unwrap_or(&self.ask_shortcut),
            patch
                .dictation_shortcut
                .as_deref()
                .unwrap_or(&self.dictation_shortcut),
            patch
                .stop_shortcut
                .as_deref()
                .unwrap_or(&self.stop_shortcut),
        ];
        shortcuts.sort_unstable();
        if shortcuts[0] == shortcuts[1] || shortcuts[1] == shortcuts[2] {
            return Err("shortcut conflict");
        }
        if let Some(value) = patch.ask_shortcut {
            validate_shortcut(&value)?;
            self.ask_shortcut = value;
        }
        if let Some(value) = patch.dictation_shortcut {
            validate_shortcut(&value)?;
            self.dictation_shortcut = value;
        }
        if let Some(value) = patch.stop_shortcut {
            validate_shortcut(&value)?;
            self.stop_shortcut = value;
        }
        if let Some(value) = patch.reduced_motion {
            self.reduced_motion = value;
        }
        if let Some(value) = patch.dictation_preview {
            self.dictation_preview = value;
        }
        if let Some(value) = patch.optional_telemetry {
            self.optional_telemetry = value;
        }
        Ok(())
    }
}

fn validate_shortcut(value: &str) -> Result<(), &'static str> {
    if value.len() < 3 || value.len() > 80 || value.contains('\n') {
        Err("invalid shortcut")
    } else {
        Ok(())
    }
}
