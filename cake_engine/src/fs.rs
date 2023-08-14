//! Кроссплатформенные функции для чтения ассетов (в том числе локализованных).
//!
//! Как и откуда конкретно загружаются ассеты, зависит от платформы и бэкенда. На большинстве
//! систем это просто считываение файлов из указанного каталога, но, например, на Android
//! используется AssetManager.

use anyhow::Result;
use once_cell::sync::Lazy;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

static ASSETS_DIRECTORY: Lazy<Mutex<PathBuf>> = Lazy::new(|| Mutex::new(PathBuf::new()));
static LANG_SUFFIXES: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Путь, из которого загружаются ассеты (по умолчанию текущий каталог).
pub fn get_assets_directory() -> PathBuf {
    ASSETS_DIRECTORY.lock().unwrap().clone()
}

/// Устанавливает путь, из которого будут загружаться ассеты (по умолчанию текущий каталог).
pub fn set_assets_directory(path: PathBuf) {
    *ASSETS_DIRECTORY.lock().unwrap() = path;
}

/// Языки, для которых функции `read_lang_asset_to_bytes` и `read_lang_asset_to_string` будут
/// искать ассеты.
pub fn get_lang_suffixes() -> Vec<String> {
    LANG_SUFFIXES.lock().unwrap().clone()
}

/// Изменяет языки, для которых функции `read_lang_asset_to_bytes` и `read_lang_asset_to_string`
/// будут искать ассеты.
pub fn set_lang_suffixes(suffixes: Vec<String>) {
    *LANG_SUFFIXES.lock().unwrap() = suffixes;
}

/// Путь к указанному ассету внутри каталога ассетов, установленного функцией
/// `set_assets_directory`.
pub fn asset_path<P: AsRef<Path>>(path: P) -> PathBuf {
    get_assets_directory().join(path)
}

/// Считывает ассет в массив байт.
pub fn read_asset_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    crate::backend::fs::read_asset_to_bytes(path)
}

/// Считывает ассет в строку (содержимое файла должно быть в кодировке UTF-8).
pub fn read_asset_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    crate::backend::fs::read_asset_to_string(path)
}

/// Ищет ассет с учётом текущего языка и считывает его в массив байт.
///
/// Перебирает все языки, установленные функцией `set_lang_suffixes`, и возвращает первый успешно
/// прочитанный файл, содержащий запрошенный язык в своём имени.
///
/// Язык добавляется в имя файла перед расширением через точку. Например, если вы установили
/// `set_lang_suffixes(vec!["ru".to_string(), "en".to_string()])`, и пытаетесь загрузить ассет
/// `logo.png`, то функция попытается прочитать файлы `logo.ru.png`, `logo.en.png` и `logo.png`
/// и вернёт первый прочитавшийся.
pub fn read_lang_asset_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut p = path.as_ref().to_path_buf();
    let filename = if let Some(f) = p.file_name() {
        f.to_str().unwrap().to_string()
    } else {
        return read_asset_to_bytes(path);
    };

    let (base, ext) = match filename.rfind('.') {
        Some(idx) => (&filename[..idx], &filename[idx..]),
        None => (filename.as_str(), ""),
    };

    let suffixes = LANG_SUFFIXES.lock().unwrap();

    for suffix in suffixes.iter() {
        p.set_file_name(format!("{}.{}{}", base, suffix, ext));
        match read_asset_to_bytes(&p) {
            Ok(result) => return Ok(result),
            Err(_) => {}
        }
    }

    read_asset_to_bytes(path)
}

/// Ищет ассет с учётом текущего языка и считывает его в строку (содержимое файла должно быть
/// в кодировке UTF-8).
///
/// Перебирает все языки, установленные функцией `set_lang_suffixes`, и возвращает первый успешно
/// прочитанный файл, содержащий запрошенный язык в своём имени.
///
/// Язык добавляется в имя файла перед расширением через точку. Например, если вы установили
/// `set_lang_suffixes(vec!["ru".to_string(), "en".to_string()])`, и пытаетесь загрузить ассет
/// `help.txt`, то функция попытается прочитать файлы `help.ru.txt`, `help.en.txt` и `help.txt`
/// и вернёт первый прочитавшийся. Файлы с неправильной кодировкой будут пропущены, даже если
/// существуют.
pub fn read_lang_asset_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut p = path.as_ref().to_path_buf();
    let filename = if let Some(f) = p.file_name() {
        f.to_str().unwrap().to_string()
    } else {
        return read_asset_to_string(path);
    };

    let (base, ext) = match filename.rfind('.') {
        Some(idx) => (&filename[..idx], &filename[idx..]),
        None => (filename.as_str(), ""),
    };

    let suffixes = LANG_SUFFIXES.lock().unwrap();

    for suffix in suffixes.iter() {
        p.set_file_name(format!("{}.{}{}", base, suffix, ext));
        match read_asset_to_string(&p) {
            Ok(result) => return Ok(result),
            Err(_) => {}
        }
    }

    read_asset_to_string(path)
}
