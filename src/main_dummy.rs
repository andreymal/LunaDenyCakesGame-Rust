use anyhow::Result;
use luna_deny_cakes_game::{build_first_scene, data, get_conf};

pub fn main() -> Result<()> {
    data::init()?;
    cake_engine::dummy::main_dummy(get_conf()?, &build_first_scene)
}
