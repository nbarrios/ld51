use bevy::{prelude::*, render::{camera::ScalingMode, view::visibility}};
use iyes_loopless::prelude::*;
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "LD51".to_string(),
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GameData {
            timer: Timer::from_seconds(20.0, true),
            threshold: 9.0,
        })
        .add_plugins(DefaultPlugins)
        .add_loopless_state(GameStates::Loading)
        .add_loading_state(
            LoadingState::new(GameStates::Loading)
                .continue_to_state(GameStates::Run)
                .with_collection::<GameAssets>(),
        )
        .add_system(update.run_in_state(GameStates::Run))
        .add_enter_system(GameStates::Run, setup)
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameStates {
    Loading,
    Run,
    GameOver,
}

#[derive(AssetCollection)]
struct GameAssets {
    #[asset(path = "fonts/NotoSansMono-Bold.ttf")]
    font: Handle<Font>,
}

#[derive(Component)]
struct Player;

struct GameData {
    timer: Timer,
    threshold: f32,
}

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedHorizontal(1920.0);
    commands.spawn_bundle(camera);

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::DARK_GREEN,
            custom_size: Some(Vec2::new(1920.0, 10.0)),
            ..default()
        },
        ..default()
    });

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(100.0, 50.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                1920.0 * -0.5 + 50.0,
                25.0 + 5.0,
                1.0,
            )),
            ..default()
        })
        .insert(Player);

    let text_style = TextStyle {
        font: game_assets.font.clone(),
        font_size: 100.0,
        color: Color::WHITE,
    };

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("0.00", text_style.clone())
            .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_translation(Vec3::new(0.0, 300.0, 1.0)),
        ..default()
    });

}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut game_data: ResMut<GameData>,
    kb_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    mut text_query: Query<(&mut Text, &mut Visibility)>,
) {
    game_data.timer.tick(time.delta());
    let (mut text, mut visibility) = text_query.single_mut();
    text.sections[0].value = format!("{:.2}", game_data.timer.elapsed_secs() % 10.0);

    if game_data.timer.elapsed_secs() > game_data.threshold {
        visibility.is_visible = false;
    } else {
        visibility.is_visible = true;
    }

    let mut player_transform = query.single_mut();
    if kb_input.just_pressed(KeyCode::Space) {
        let distance = (game_data.timer.elapsed_secs() - 10.0).abs() % 10.0;
        info!("Distance: {}", distance);
        player_transform.translation.x += 10.0 / distance.clamp(0.01, 10.0);

        game_data.threshold -= 1.0;
        game_data.threshold = game_data.threshold.clamp(1.0, 9.0);
        game_data.timer.reset();
    }
}
