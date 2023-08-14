use anyhow::Result;
use cake_engine::{context::Context, sprite::Sprite, texture::Texture, vec::Vec2};
use std::rc::Rc;

pub struct Dvd {
    sprite: Sprite,
    textures: Vec<Rc<Texture>>,
    texture_idx: usize,
    speed: Vec2,
}

impl Dvd {
    pub fn new(textures: Vec<Rc<Texture>>, position: Vec2, speed: Vec2) -> Dvd {
        let mut sprite = Sprite::new(textures.first().unwrap().clone());
        sprite.set_position(position);
        Dvd {
            sprite,
            textures,
            texture_idx: 0,
            speed,
        }
    }

    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }

    pub fn sprite_mut(&mut self) -> &mut Sprite {
        &mut self.sprite
    }

    pub fn process(&mut self, ctx: &mut dyn Context, dt: f32) -> Result<()> {
        let dt = if dt < 0.5 { dt } else { 0.5 };

        let mut pos = self.sprite.get_position();
        let size = self.sprite.get_absolute_size();

        let area = ctx.view().visible_area();
        let left = area.x;
        let top = area.y;
        let right = area.x + area.width - size.x;
        let bottom = area.y + area.height - size.y;

        let mut bounces = 0;

        pos.x += self.speed.x * dt;
        if pos.x >= right {
            pos.x = right * 2.0 - pos.x;
            self.speed.x = -self.speed.x;
            bounces += 1;
        }
        if pos.x < left {
            pos.x = left * 2.0 - pos.x;
            self.speed.x = -self.speed.x;
            bounces += 1;
        }

        pos.y += self.speed.y * dt;
        if pos.y >= bottom {
            pos.y = bottom * 2.0 - pos.y;
            self.speed.y = -self.speed.y;
            bounces += 1;
        }
        if pos.y < top {
            pos.y = top * 2.0 - pos.y;
            self.speed.y = -self.speed.y;
            bounces += 1;
        }

        for _ in 0..bounces {
            self.texture_idx = (self.texture_idx + 1) % self.textures.len();
            let t = self.textures[self.texture_idx].clone();
            self.sprite.set_static_texture(t);
        }

        self.sprite.set_position(pos);

        Ok(())
    }

    pub fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        self.sprite.render(ctx)
    }
}
