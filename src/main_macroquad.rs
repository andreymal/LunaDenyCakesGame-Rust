#![windows_subsystem = "windows"]

use anyhow::Result;
use luna_deny_cakes_game::{build_first_scene, data, get_conf};

fn window_conf() -> macroquad::prelude::Conf {
    data::init().unwrap();
    get_conf().unwrap().into()
}

#[macroquad::main(window_conf)]
async fn main() -> Result<()> {
    cake_engine::main_macroquad(get_conf()?, &build_first_scene).await?;

    #[cfg(target_os = "android")]
    std::process::exit(0);

    Ok(())
}
