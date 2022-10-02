use bevy::{prelude::*, render::camera::ScalingMode, sprite::Anchor};
use bevy_asset_loader::prelude::*;
use bevy_ggrs::*;
use ggrs::InputStatus;
use matchbox_socket::WebRtcSocket;

struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    // 4-directions + fire fits easily in a single byte
    type Input = u8;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are strings
    type Address = String;
}

struct LocalPlayerHandle(usize);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Loading,
    Matchmaking,
    InGame,
}

#[derive(AssetCollection)]
struct GameAssets {
    #[asset(path = "fonts/NotoSansMono-Bold.ttf")]
    font: Handle<Font>,
    #[asset(path = "textures/snailtest02.png")]
    snail: Handle<Image>,
    #[asset(path = "textures/ground-grass-seamless_00.png")]
    grass: Handle<Image>,
}

#[derive(Component)]
struct Player {
    handle: usize,
}

struct GameData {
    timer: Timer,
    threshold: f32,
}

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    GGRSPlugin::<GgrsConfig>::new()
        .with_input_system(input)
        .with_rollback_schedule(
            Schedule::default().with_stage(
                "ROLLBACK_STAGE",
                SystemStage::single_threaded()
                    .with_system_set(State::<GameState>::get_driver())
                    .with_system_set(SystemSet::on_update(GameState::InGame).with_system(update)),
            ),
        )
        .register_rollback_type::<Transform>()
        .build(&mut app);

    app.insert_resource(WindowDescriptor {
        title: "LD51".to_string(),
        ..Default::default()
    })
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(GameData {
        timer: Timer::from_seconds(20.0, true),
        threshold: 9.0,
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(bevy_web_resizer::Plugin)
    .add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Matchmaking)
            .with_collection::<GameAssets>(),
    )
    .add_state(GameState::Loading)
    .add_system_set(
        SystemSet::on_enter(GameState::Matchmaking)
            .with_system(start_matchbox_socket)
            .with_system(setup),
    )
    .add_system_set(SystemSet::on_update(GameState::Matchmaking).with_system(wait_for_players))
    .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(spawn_players))
    .run();
}

#[cfg(target_arch = "wasm32")]
fn start_matchbox_socket(mut commands: Commands) {
    use bevy::tasks::IoTaskPool;

    let room_url = "ws://150.230.44.222:3536/next_2";
    info!("Connecting to matchbox server: {:?}", room_url);

    let (socket, message_loop) = WebRtcSocket::new(room_url);
    let task_pool = IoTaskPool::get();
    task_pool.spawn(message_loop).detach();
    commands.insert_resource(Some(socket));
}

fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<Option<WebRtcSocket>>,
    mut state: ResMut<State<GameState>>,
) {
    let socket = socket.as_mut();

    if socket.is_none() {
        return;
    }

    socket.as_mut().unwrap().accept_new_connections();
    let players = socket.as_ref().unwrap().players();

    let num_players = 2;
    if players.len() < num_players {
        return;
    }

    info!("All players connected");

    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(num_players)
        .with_max_prediction_window(12)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        if player == ggrs::PlayerType::Local {
            commands.insert_resource(LocalPlayerHandle(i));
        }

        session_builder = session_builder
            .add_player(player.clone(), i)
            .expect("failed to add player");
    }

    // consume the socket (currently required because GGRS takes ownership of its socket)
    let socket = socket.take().unwrap();

    let session = session_builder
        .start_p2p_session(socket)
        .expect("failed to start session");

    commands.insert_resource(session);
    commands.insert_resource(SessionType::P2PSession);

    state.set(GameState::InGame).unwrap();
}

fn setup(
    mut commands: Commands, 
    game_assets: Res<GameAssets>
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedHorizontal(1920.0);
    commands.spawn_bundle(camera);

    for i in 0..=3 {
        commands.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..default()
            },
            texture: game_assets.grass.clone(), 
            transform: Transform::from_translation(Vec3::new(1920.0 * -0.5 + (i as f32) * 500.0, -540.0, 0.0)),
            ..default()
        });
    }

    let text_style = TextStyle {
        font: game_assets.font.clone(),
        font_size: 100.0,
        color: Color::WHITE,
    };

    commands.spawn_bundle(Text2dBundle {
        text: Text::from_section("0.00", text_style.clone()).with_alignment(TextAlignment::CENTER),
        transform: Transform::from_translation(Vec3::new(0.0, 540.0 - 60.0, 1.0)),
        ..default()
    });
}

fn spawn_players(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    assets: Res<GameAssets>,
    session: Res<ggrs::P2PSession<GgrsConfig>>,
    player_query: Query<Entity, With<Player>>,
) {
    //Despawn Old Players
    for player in player_query.iter() {
        commands.entity(player).despawn_recursive();
    }

    let num_players = session.num_players();
    info!("Spawning {} players", num_players);

    for i in 0..num_players {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    ..default()
                },
                texture: assets.snail.clone(), 
                transform: Transform::from_translation(Vec3::new(
                    1920.0 * -0.5 + 50.0,
                    250.0 * (i as f32),
                    1.0,
                )),
                ..default()
            })
            .insert(Player { handle: i })
            .insert(Rollback::new(rip.next_id()));
    }
}

const INPUT_SPACE: u8 = 1 << 0;

fn input(_: In<ggrs::PlayerHandle>, keys: Res<Input<KeyCode>>) -> u8 {
    let mut input = 0u8;

    if keys.just_pressed(KeyCode::Space) {
        input |= INPUT_SPACE;
    }

    input
}

fn update(
    mut _commands: Commands,
    time: Res<Time>,
    mut game_data: ResMut<GameData>,
    inputs: Res<Vec<(u8, InputStatus)>>,
    mut query: Query<(&mut Transform, &Player)>,
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

    for (mut transform, player) in &mut query {
        let input = match inputs[player.handle].1 {
            InputStatus::Confirmed => inputs[player.handle].0,
            InputStatus::Predicted => inputs[player.handle].0,
            InputStatus::Disconnected => 0,
        };

        if input & INPUT_SPACE != 0 {
            let distance = (game_data.timer.elapsed_secs() - 10.0).abs() % 10.0;
            info!("Distance: {}", distance);
            //transform.translation.x += 10.0 / distance.clamp(0.01, 10.0);
            transform.translation.x += 50.0;

            game_data.threshold -= 1.0;
            game_data.threshold = game_data.threshold.clamp(1.0, 9.0);
            game_data.timer.reset();
        }
    }
}
