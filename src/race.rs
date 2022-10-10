use crate::{
    Animation, AnimationState, GameAssets, GameState, LocalPlayerHandle, Player,
    PlayerLocal, PlayerTarget, PlayerTimer,
};
use benimator::FrameRate;
use bevy::{prelude::*, time::Stopwatch, utils::Duration};

const SPAWN_X: f32 = 1920.0 * -0.25;
const TIMINGS: [f32; 5] = [10.0, 5.0, 3.0, 2.0, 1.0];

#[derive(Debug, Eq, PartialEq)]
pub enum Alert {
    Miss,
    Ok,
    Good,
    Perfect,
}

#[derive(Component)]
pub struct AlertSprite;

#[derive(Component)]
pub struct IndicatorCursor;

#[derive(Component)]
pub struct StopwatchText;

pub fn setup(mut commands: Commands, game_assets: Res<GameAssets>, audio: Res<Audio>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(
                    TextBundle::from_section(
                        "0.00",
                        TextStyle {
                            font: game_assets.font.clone(),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_text_alignment(TextAlignment::TOP_CENTER)
                    .with_style(Style {
                        align_self: AlignSelf::FlexEnd,
                        position_type: PositionType::Relative,
                        position: UiRect {
                            top: Val::Px(20.0),
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            ..default()
                        },
                        ..Default::default()
                    }),
                )
                .insert(StopwatchText);
        });

    //Audio
    let _weak_handle = audio.play_with_settings(
        game_assets.music.clone(),
        PlaybackSettings::LOOP.with_volume(0.05),
    );
}

pub fn spawn_players(
    mut commands: Commands,
    assets: Res<GameAssets>,
    local_handle: Res<LocalPlayerHandle>,
    player_query: Query<Entity, With<Player>>,
) {
    //Despawn Old Players
    for player in player_query.iter() {
        commands.entity(player).despawn_recursive();
    }

    let num_players = 2;
    info!("Spawning {} players", num_players);

    let colors = vec![Color::WHITE, Color::CYAN, Color::GOLD, Color::GREEN];
    for i in 0..num_players {
        let spawn_y = -100.0 + 250.0 * (i as f32);
        let mut builder = commands.spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 0,
                color: colors[i],
                ..default()
            },
            texture_atlas: assets.snail_idle.clone(),
            transform: Transform::from_translation(Vec3::new(SPAWN_X, spawn_y, 2.0)),
            ..default()
        });

        builder
            .insert(Player {
                handle: i,
                cooldown_timer: Timer::from_seconds(0.5, false),
                on_cooldown: false,
                timing_index: 0,
            })
            .insert(PlayerTimer {
                timer: Timer::from_seconds(20.0, false),
                stopwatch: Stopwatch::new(),
            })
            .insert(PlayerTarget { x: SPAWN_X })
            .insert(Animation(benimator::Animation::from_indices(
                15..30,
                FrameRate::from_fps(12.0),
            )))
            .insert(AnimationState::default());

        if i == local_handle.0 {
            builder.insert(PlayerLocal);
            let player_id = builder.id();

            //Indicator Cursor
            let indicator_target_id = commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite { ..default() },
                    texture: assets.indicator_target.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, -90.0, -1.0)),
                    ..default()
                })
                .id();

            //Indicator Target
            let indicator_cursor_id = commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite { ..default() },
                    texture: assets.indicator_cursor.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, -90.0, -1.0)),
                    ..default()
                })
                .insert(IndicatorCursor)
                .id();

            commands.entity(player_id).add_child(indicator_target_id);
            commands.entity(player_id).add_child(indicator_cursor_id);
        }
    }
}

pub fn cleanup(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    //Despawn Old Players
    for player in player_query.iter() {
        commands.entity(player).despawn_recursive();
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

pub fn update_rollback(
    mut _commands: Commands,
    inputs: Res<Input<KeyCode>>,
    mut query: Query<(&mut PlayerTarget, &mut PlayerTimer, &mut Player)>,
    mut ev_alert: EventWriter<Alert>,
) {
    for (mut target, mut player_timer, mut player) in &mut query {
        if inputs.just_pressed(KeyCode::Space) && !player.on_cooldown {
            let threshold = match (player_timer.timer.percent_left() - 0.5).abs() {
                x if x < 0.01 => Alert::Perfect,
                x if x < 0.05 => Alert::Good,
                x if x < 0.1 => Alert::Ok,
                _ => Alert::Miss,
            };

            target.x += match threshold {
                Alert::Perfect => 500.0,
                Alert::Good => 250.0,
                Alert::Ok => 125.0,
                Alert::Miss => 0.0,
            };

            ev_alert.send(threshold);

            if (player_timer.timer.percent_left() - 0.5).abs() < 0.05 {
                player.timing_index += 1;
            } else {
                player.timing_index = 0;
            }
            player.timing_index = player.timing_index.clamp(0, TIMINGS.len() - 1);
            let next_timer = TIMINGS[player.timing_index];

            player_timer
                .timer
                .set_duration(Duration::from_secs_f32(next_timer * 2.0));
            player_timer.timer.reset();
            player.on_cooldown = true;
        }
    }
}

pub fn update(
    local_handle: Res<LocalPlayerHandle>,
    mut state: ResMut<State<GameState>>,
    mut query: Query<(&mut Transform, &PlayerTarget, &mut PlayerTimer, &mut Player)>,
    mut cursor: Query<&mut Transform, (With<IndicatorCursor>, Without<Player>)>,
    mut text_query: Query<&mut Text, With<StopwatchText>>,
    mut ev_alert: EventWriter<Alert>,
) {
    for (mut transform, target, mut player_timer, mut player) in &mut query {
        if player.handle == local_handle.0 {
            let elapsed = player_timer.stopwatch.elapsed_secs();
            let mut text = text_query.single_mut();
            text.sections[0].value = format!("{:.2}", elapsed);

            let mut cursor_transform = cursor.single_mut();
            cursor_transform.translation.x =
                -1920.0 + 1920.0 * 2.0 * player_timer.timer.percent_left();

            if player_timer.timer.percent_left() < 0.35 {
                player.timing_index = 0;
                let next_timer = TIMINGS[player.timing_index];
                player_timer
                    .timer
                    .set_duration(Duration::from_secs_f32(next_timer * 2.0));
                player_timer.timer.reset();

                ev_alert.send(Alert::Miss);
            }
        }

        transform.translation = transform.translation.lerp(
            Vec3::new(target.x, transform.translation.y, transform.translation.z),
            0.05,
        );

        if transform.translation.x > 10300.0 {
            player_timer.stopwatch.pause();
            state.set(GameState::End).unwrap();
        }
    }
}

pub fn feedback_spawn(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut ev_alert: EventReader<Alert>,
    player_query: Query<&Transform, With<PlayerLocal>>,
    audio: Res<Audio>,
) {
    let player_transform = player_query.single();
    for ev in ev_alert.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite { ..default() },
                texture: match ev {
                    Alert::Perfect => assets.alert_perfect.clone(),
                    Alert::Good => assets.alert_good.clone(),
                    Alert::Ok => assets.alert_ok.clone(),
                    Alert::Miss => assets.alert_miss.clone(),
                },
                transform: Transform::from_translation(Vec3::new(
                    player_transform.translation.x + 600.0,
                    100.0,
                    1.0,
                )),
                ..default()
            })
            .insert(AlertSprite);

        if ev != &Alert::Miss {
            let _weak_handle = audio.play_with_settings(
                assets.match_sound.clone(),
                PlaybackSettings::ONCE.with_volume(0.5),
            );
        }
    }
}

pub fn feedback_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite), With<AlertSprite>>,
) {
    for (entity, mut sprite) in query.iter_mut() {
        let alpha = sprite.color.a();
        sprite.color.set_a(alpha - 1.0 * time.delta_seconds());
        if sprite.color.a() <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn tick_timers(time: Res<Time>, mut timer_query: Query<(&mut PlayerTimer, &mut Player)>) {
    for (mut timer, mut player) in &mut timer_query {
        timer.timer.tick(time.delta());
        timer.stopwatch.tick(time.delta());

        if player.on_cooldown {
            player.cooldown_timer.tick(time.delta());
            if player.cooldown_timer.just_finished() {
                player.on_cooldown = false;
                player.cooldown_timer.reset();
            }
        }
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
