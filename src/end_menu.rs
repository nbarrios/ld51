use crate::{
    GameAssets, GameState
};

use bevy::{prelude::*};



pub fn setup(
    mut commands: Commands,
    assets: Res<GameAssets>,
    audio: Res<Audio>
) {
    info!("Ending");
    commands.spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
    .with_children(|parent| {
        parent.spawn_bundle(ImageBundle {
            image: UiImage(assets.try_again.clone()),
            style: Style {
                size: Size::new(Val::Px(500.0), Val::Auto),
                ..default()
            },
            ..default()
        });
    });

    audio.play_with_settings(
        assets.crowd_sound.clone(),
        PlaybackSettings::ONCE.with_volume(0.5),
    );
}

pub fn update(
    keys: Res<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
) {
    if keys.just_pressed(KeyCode::Return) {
        state.set(GameState::Matchmaking).unwrap();
    }
}