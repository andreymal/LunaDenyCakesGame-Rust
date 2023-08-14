pub mod achievements;
pub mod action;
pub mod balance;
pub mod common_data;
pub mod data;
pub mod dvd;
pub mod game;
pub mod gameaction;
pub mod scene;
pub mod touchui;

mod utils;

use crate::{common_data::CommonData, scene::menu::SceneMenu};
use anyhow::Result;
use cake_engine::{
    conf::{Conf, WindowIcon},
    context::Context,
    rect::Rect,
    scene::Scene,
};
use std::path::PathBuf;

pub fn get_conf() -> Result<Conf> {
    let options = data::options::OPTIONS.lock().unwrap();

    Ok(Conf {
        title: data::texts::get_text("gametitle"),
        icon: Some(WindowIcon {
            path16: PathBuf::from("images/icon_16x16.png"),
            path32: PathBuf::from("images/icon_32x32.png"),
            path64: PathBuf::from("images/icon_64x64.png"),
        }),
        logical_size: options.get_window_size(),
        view: Some(Rect::new(0.0, 0.0, 1024.0, 768.0)),
        fullscreen: options.get_fullscreen(),
        vsync: options.get_vsync(),
        fps_limit: options.get_fps_limit(),
        resizable: true,
        mouse_cursor_visible: cfg!(target_os = "android"),
        ..Default::default()
    })
}

pub fn build_first_scene(ctx: &mut dyn Context) -> Result<Box<dyn Scene>> {
    let mut common_data = CommonData::new(ctx)?;

    if let Some(data_dir) = data::data_dir() {
        let apath = data_dir.join("achievements.json");
        if let Err(e) = common_data.achievements.load(&apath) {
            if apath.exists() {
                cake_engine::log::error!("Failed to load achievements: {:?}", e);
            }
        }
    }

    let first_scene = SceneMenu::new(common_data, ctx)?;
    Ok(Box::new(first_scene))
}
