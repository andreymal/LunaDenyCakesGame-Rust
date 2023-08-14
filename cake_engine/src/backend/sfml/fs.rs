use crate::fs::asset_path;
use anyhow::Result;
use std::path::Path;

pub fn read_asset_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let fullpath = asset_path(path);
    Ok(std::fs::read(fullpath)?)
}

pub fn read_asset_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let fullpath = asset_path(path);
    Ok(std::fs::read_to_string(fullpath)?)
}
