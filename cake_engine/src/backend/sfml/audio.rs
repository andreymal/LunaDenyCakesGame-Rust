// Sound в SFML не владеет звуковыми данными, которые должны храниться где-то отдельно, из-за чего
// возникает вопрос — а где? Похоже, единственный рабочий не-static вариант — сделать
// self-referential struct

use self_cell::self_cell;
use sfml::{
    audio::{Sound, SoundBuffer},
    SfBox,
};
use std::rc::Rc;

self_cell!(
    pub(super) struct SfmlSound {
        owner: Rc<SfBox<SoundBuffer>>,

        #[covariant]
        dependent: Sound,
    }
);
