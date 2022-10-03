use bevy::{prelude::*};

pub const INPUT_SPACE: u8 = 1 << 0;

pub fn input(_: In<ggrs::PlayerHandle>, keys: Res<Input<KeyCode>>) -> u8 {
    let mut input = 0u8;

    if keys.pressed(KeyCode::Space) {
        input |= INPUT_SPACE;
    }

    input
}