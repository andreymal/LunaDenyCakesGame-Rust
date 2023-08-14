use anyhow::Result;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

static TEXTS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn load_from_file(filename: &str) -> Result<()> {
    let texts_json = cake_engine::fs::read_asset_to_string(filename)?;
    let texts: HashMap<String, String> = serde_json::from_str(&texts_json)?;
    TEXTS.lock().unwrap().extend(texts);
    Ok(())
}

pub fn load_from_lang_file(filename: &str) -> Result<()> {
    let texts_json = cake_engine::fs::read_lang_asset_to_string(filename)?;
    let texts: HashMap<String, String> = serde_json::from_str(&texts_json)?;
    TEXTS.lock().unwrap().extend(texts);
    Ok(())
}

pub fn get_text(key: &str) -> String {
    match TEXTS.lock().unwrap().get(key) {
        Some(x) => x.clone(),
        None => format!("[{}]", key),
    }
}
