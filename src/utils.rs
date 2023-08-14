// Макросы, позволяющие сократить код часто повторяемых операций

// Загрузка пиксельной текстуры из файла
macro_rules! tex {
    ( $ctx:expr, $filename:expr ) => {
        $ctx.load_texture(
            cake_engine::texture::TextureSource::File(std::path::PathBuf::from($filename)),
            cake_engine::texture::TextureOptions::PIXELATED,
        )?
    };
}

macro_rules! tex_lang {
    ( $ctx:expr, $filename:expr ) => {
        $ctx.load_texture(
            cake_engine::texture::TextureSource::LangFile(std::path::PathBuf::from($filename)),
            cake_engine::texture::TextureOptions::PIXELATED,
        )?
    };
}

// Создание спрайта с загрузкой пиксельной текстуры из файла
macro_rules! spr {
    // Статический спрайт
    ( $ctx:expr, $filename:expr ) => {
        cake_engine::sprite::Sprite::new(tex!($ctx, $filename))
    };

    // Анимированный с заданной сеткой кадров
    ( $ctx:expr, $filename:expr, $fps:expr, grid: $grid:expr ) => {
        cake_engine::sprite::Sprite::new_animated_grid(tex!($ctx, $filename), $fps, $grid)
    };

    // Анимированный с заданным размером кадров
    ( $ctx:expr, $filename:expr, $fps:expr, frame: $frame:expr ) => {
        cake_engine::sprite::Sprite::new_animated(tex!($ctx, $filename), $fps, $frame)
    };
}

// Создание типовой кнопки
macro_rules! btn_with_tex {
    ( $common_data:expr, $tex:expr, $text:expr, $pos:expr ) => {{
        let mut b = cake_engine::button::Button::new($tex, $pos);
        b.set_color($common_data.color_norm);
        b.set_color_hover($common_data.color_over);
        b.set_label(Some(cake_engine::label::Label::new(
            $common_data.font_button.clone(),
            cake_engine::color::Color::WHITE,
        )));
        if let Some(l) = b.label_mut() {
            l.set_text($text);
            l.set_origin(cake_engine::vec::Vec2::new(0.5, 0.6));
        }
        b
    }};
}

macro_rules! btn {
    ( $common_data:expr, $text:expr, $pos:expr ) => {
        $crate::utils::btn_with_tex!($common_data, $common_data.button.clone(), $text, $pos)
    };
}

macro_rules! btn_small {
    ( $common_data:expr, $text:expr, $pos:expr ) => {
        $crate::utils::btn_with_tex!($common_data, $common_data.button_small.clone(), $text, $pos)
    };
}

pub(crate) use btn;
pub(crate) use btn_small;
pub(crate) use btn_with_tex;
pub(crate) use spr;
pub(crate) use tex;
pub(crate) use tex_lang;
