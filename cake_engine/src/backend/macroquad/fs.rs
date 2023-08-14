// Мне лень переделывать весь движок под асинхронщину, поэтому костыляю через futures_executor.
// На большинстве платформ операция чтения файла реализована как синхронная, так что этот костыль
// худо-бедно работает, ну и ладно

// И да, мне пришлось отказаться от использования macroquad::file на не-Android устройствах,
// потому что он не позволяет прочитать параметры окна из ассета перед собственно созданием окна
// (спасибо что хоть на Android работает)

use crate::fs::asset_path;
use anyhow::Result;
use std::path::Path;

#[cfg(not(target_os = "android"))]
pub fn read_asset_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let fullpath = asset_path(path);
    Ok(std::fs::read(fullpath)?)
}

#[cfg(not(target_os = "android"))]
pub fn read_asset_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let fullpath = asset_path(path);
    Ok(std::fs::read_to_string(fullpath)?)
}

#[cfg(target_os = "android")]
pub fn read_asset_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let fullpath = asset_path(path);
    Ok(futures_executor::block_on(macroquad::prelude::load_file(
        fullpath.to_str().unwrap(),
    ))?)
}

#[cfg(target_os = "android")]
pub fn read_asset_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let fullpath = asset_path(path);
    Ok(futures_executor::block_on(
        macroquad::prelude::load_string(fullpath.to_str().unwrap()),
    )?)
}
