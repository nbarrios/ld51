use crate::{
    input, Animation, AnimationState, GameAssets, GameData, GgrsConfig, LocalPlayerHandle, Player,
    PlayerTimer, PlayerTarget, PlayerLocal
};
use benimator::FrameRate;
use bevy::{prelude::*, render::camera::ScalingMode, sprite::Anchor};
use bevy_ggrs::*;
use ggrs::InputStatus;

const SPAWN_X: f32 = 1920.0 * -0.5 + 100.0;

pub fn setup(mut commands: Commands, game_assets: Res<GameAssets>) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedHorizontal(1920.0);
    let camera_id = commands.spawn_bundle(camera).id();

    for i in 0..=9 {
        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..default()
            },
            texture: game_assets.grass.clone(),
            transform: Transform::from_translation(Vec3::new(
                1920.0 * -0.5 + (i as f32) * 500.0,
                -540.0,
                0.0,
            )),
            ..default()
        });
    }

    let text_style = TextStyle {
        font: game_assets.font.clone(),
        font_size: 100.0,
        color: Color::WHITE,
    };

    let text_id = commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("0.00", text_style.clone()).with_alignment(TextAlignment::CENTER),
        transform: Transform::from_translation(Vec3::new(0.0, 540.0 - 60.0, 1.0)),
        ..default()
    }).id();
}

pub fn spawn_players(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    assets: Res<GameAssets>,
    session: Res<ggrs::P2PSession<GgrsConfig>>,
    local_handle: Res<LocalPlayerHandle>,
    player_query: Query<Entity, With<Player>>,
) {
    //Despawn Old Players
    for player in player_query.iter() {
        commands.entity(player).despawn_recursive();
    }

    let num_players = session.num_players();
    info!("Spawning {} players", num_players);

    let colors = vec![Color::WHITE, Color::CYAN, Color::GOLD, Color::GREEN];
    for i in 0..num_players {
        let mut builder = commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: 0,
                    color: colors[i],
                    ..default()
                },
                texture_atlas: assets.snail_idle.clone(),
                transform: Transform::from_translation(Vec3::new(
                    SPAWN_X,
                    -100.0 + 250.0 * (i as f32),
                    1.0,
                )),
                ..default()
            });

        builder.insert(Player { handle: i })
            .insert(PlayerTimer {
                timer: Timer::from_seconds(20.0, true),
            })
            .insert(PlayerTarget {
                x: SPAWN_X,
            })
            .insert(Animation(benimator::Animation::from_indices(
                0..15,
                FrameRate::from_fps(12.0),
            )))
            .insert(AnimationState::default())
            .insert(Rollback::new(rip.next_id()));

        if i == local_handle.0 {
            builder.insert(PlayerLocal);
        }
    }
}

pub fn animate(
    time: Res<Time>,
    mut query: Query<(&mut AnimationState, &mut TextureAtlasSprite, &Animation)>,
) {
    for (mut state, mut sprite, animation) in &mut query {
        state.update(animation, time.delta());
        sprite.index = state.frame_index();
    }
}

pub fn update(
    mut _commands: Commands,
    time: Res<Time>,
    mut game_data: ResMut<GameData>,
    inputs: Res<Vec<(u8, InputStatus)>>,
    local_handle: Res<LocalPlayerHandle>,
    mut query: Query<(&mut Transform, &mut PlayerTarget, &mut PlayerTimer, &Player)>,
    mut text_query: Query<(&mut Text, &mut Visibility)>,
) {
    // if game_data.timer.elapsed_secs() > game_data.threshold {
    //     visibility.is_visible = false;
    // } else {
    //     visibility.is_visible = true;
    // }

    if game_data.on_cooldown {
        game_data.cooldown_timer.tick(time.delta());
        if game_data.cooldown_timer.just_finished() {
            game_data.on_cooldown = false;
            game_data.cooldown_timer.reset();
        }
    }

    for (mut transform, mut target, mut player_timer, player) in &mut query {
        player_timer.timer.tick(time.delta());

        if player.handle == local_handle.0 {
            let (mut text, mut visibility) = text_query.single_mut();
            text.sections[0].value = format!("{:.2}", player_timer.timer.elapsed_secs() % 10.0);
        }

        let input = match inputs[player.handle].1 {
            InputStatus::Confirmed => inputs[player.handle].0,
            InputStatus::Predicted => inputs[player.handle].0,
            InputStatus::Disconnected => 0,
        };

        if input & input::INPUT_SPACE != 0 && !game_data.on_cooldown {
            let distance = (player_timer.timer.elapsed_secs() - 10.0).abs() % 10.0;
            info!("Distance: {}", distance);
            target.x += match distance {
                x if x < 0.1 => 500.0,
                x if x < 0.5 => 250.0,
                x if x < 1.0 => 125.0,
                x if x < 2.0 => 50.0,
                _ => 0.0,
            };

            game_data.threshold -= 1.0;
            game_data.threshold = game_data.threshold.clamp(1.0, 9.0);
            player_timer.timer.reset();
            game_data.on_cooldown = true;
        }

        transform.translation = transform.translation.lerp(
            Vec3::new(target.x, transform.translation.y, transform.translation.z),
            0.05,
        );
    }
}

pub fn camera_control(
    mut query: Query<&mut Transform, (With<Camera>, Without<PlayerLocal>)>,
    player_query: Query<&Transform, With<PlayerLocal>>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = query.single_mut();

    camera_transform.translation = camera_transform.translation.lerp(
        Vec3::new(
            player_transform.translation.x - SPAWN_X,
            camera_transform.translation.y,
            camera_transform.translation.z,
        ),
        0.025,
    );
}