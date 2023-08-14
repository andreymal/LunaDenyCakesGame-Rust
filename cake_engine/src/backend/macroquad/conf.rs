use crate::conf::Conf;
use macroquad::{
    miniquad::conf::{LinuxBackend, LinuxX11Gl, Platform},
    texture::Image,
};

impl Into<macroquad::prelude::Conf> for Conf {
    fn into(self) -> macroquad::prelude::Conf {
        let mq_icon = if let Some(icon) = self.icon.as_ref() {
            let data16 = crate::fs::read_asset_to_bytes(&icon.path16).unwrap();
            let data32 = crate::fs::read_asset_to_bytes(&icon.path32).unwrap();
            let data64 = crate::fs::read_asset_to_bytes(&icon.path64).unwrap();

            let image16 = Image::from_file_with_format(&data16, None).unwrap();
            let image32 = Image::from_file_with_format(&data32, None).unwrap();
            let image64 = Image::from_file_with_format(&data64, None).unwrap();

            Some(macroquad::miniquad::conf::Icon {
                small: image16.bytes.try_into().unwrap(),
                medium: image32.bytes.try_into().unwrap(),
                big: image64.bytes.try_into().unwrap(),
            })
        } else {
            None
        };

        macroquad::prelude::Conf {
            window_title: self.title,
            icon: mq_icon,
            window_width: self.logical_size.x as i32,
            window_height: self.logical_size.y as i32,
            fullscreen: self.fullscreen,
            window_resizable: self.resizable,
            high_dpi: true,
            platform: Platform {
                linux_x11_gl: LinuxX11Gl::EGLWithGLXFallback,
                linux_backend: LinuxBackend::X11Only,
                swap_interval: None,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
