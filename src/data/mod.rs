use crate::data::options::{Options, OPTIONS};
use anyhow::Result;
use cake_engine::fs;
use std::path::PathBuf;

pub mod options;
pub mod texts;

pub fn data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    {
        #[cfg(not(feature = "macroquad"))]
        compile_error!("Android is currently supported only for macroquad build");

        let files_dir_string = cake_engine::android::get_files_dir();
        return Some(PathBuf::from(files_dir_string));
    }

    #[cfg(not(target_os = "android"))]
    if let Some(p) = dirs::data_dir() {
        Some(p.join("LunaDenyCakesGame-Rust"))
    } else {
        None
    }
}

pub fn init_assets_directory() {
    if cfg!(target_os = "android") {
        return;
    }

    let args: Vec<String> = std::env::args().collect();

    let path = match args.get(1) {
        Some(x) => x,
        None => "data",
    };

    fs::set_assets_directory(path.into());
}

pub fn init_options() -> Result<()> {
    let languages_json = fs::read_asset_to_string("languages.json")?;
    let languages: Vec<String> = serde_json::from_str(&languages_json)?;

    let mut options = OPTIONS.lock().unwrap();
    options.set_available_languages(&languages);

    if let Err(e) = options.load() {
        let path = Options::path();
        if path.is_none() || path.unwrap().exists() {
            cake_engine::log::error!("Failed to load options: {:?}", e);
        }
        options.set_current_language(Options::get_system_language());
    }

    Ok(())
}

pub fn init() -> Result<()> {
    init_assets_directory();
    init_options()?;
    reload_lang(&OPTIONS.lock().unwrap())?;
    Ok(())
}

pub fn reload_lang(options: &Options) -> Result<()> {
    let mut suffixes = Vec::new();

    if !options.get_current_language().is_empty() {
        suffixes.push(options.get_current_language().to_string());
    }

    if let Some(l) = options.get_available_languages().first() {
        if !suffixes.iter().any(|l2| l2 == l) {
            suffixes.push(l.clone());
        }
    }

    fs::set_lang_suffixes(suffixes);
    texts::load_from_lang_file("strings.json")?;
    Ok(())
}
