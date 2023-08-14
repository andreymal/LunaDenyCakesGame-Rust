use crate::{backend::sdl::SdlError, fs::asset_path};
use anyhow::Result;
use std::{io::Read, path::Path};

pub fn read_asset_to_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let fullpath = asset_path(path);
    let mut stream = sdl2::rwops::RWops::from_file(fullpath, "rb").map_err(SdlError)?;
    let mut result = Vec::new();
    stream.read_to_end(&mut result)?;
    Ok(result)
}

pub fn read_asset_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let fullpath = asset_path(path);
    let mut stream = sdl2::rwops::RWops::from_file(fullpath, "rb").map_err(SdlError)?;
    let mut result = String::new();
    stream.read_to_string(&mut result)?;
    Ok(result)
}
