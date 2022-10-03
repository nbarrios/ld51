mod input;
mod race;

use bevy::{prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_ggrs::*;
use bevy_heterogeneous_texture_atlas_loader::*;
use ggrs::PlayerHandle;
use matchbox_socket::WebRtcSocket;

pub struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    // 4-directions + fire fits easily in a single byte
    type Input = u8;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are strings
    type Address = String;
}

pub struct LocalPlayerHandle(PlayerHandle);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Loading,
    Matchmaking,
    InGame,
}

#[derive(AssetCollection)]
pub struct GameAssets {
    #[asset(path = "fonts/waterst2.ttf")]
    font: Handle<Font>,
    #[asset(path = "textures/ground-grass-seamless_00.png")]
    grass: Handle<Image>,
    #[asset(path = "snail_idle.ron")]
    snail_idle: Handle<TextureAtlas>,
}

#[derive(Component)]
pub struct Player {
    handle: usize,
}

#[derive(Component, Default, Reflect)]
pub struct PlayerTimer {
    timer: Timer,
}

#[derive(Component, Default, Reflect)]
pub struct PlayerTarget {
    x: f32,
}

#[derive(Component)]
pub struct PlayerLocal;

pub struct GameData {
    cooldown_timer: Timer,
    on_cooldown: bool,
    threshold: f32,
}

pub struct MatchmakingTimer {
    timer: Timer,
}

#[derive(Component, Deref)]
pub struct Animation(benimator::Animation);

#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(benimator::State);

fn main() {
    // When building for WASM, print panics to the browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    GGRSPlugin::<GgrsConfig>::new()
        .with_input_system(input::input)
        .with_rollback_schedule(
            Schedule::default().with_stage(
                "ROLLBACK_STAGE",
                SystemStage::single_threaded()
                    .with_system_set(State::<GameState>::get_driver())
                    .with_system_set(SystemSet::on_update(GameState::InGame)
                        .with_system(race::update)
                        .with_system(race::animate)
                    ),
            ),
        )
        .register_rollback_type::<Transform>()
        .register_rollback_type::<PlayerTimer>()
        .build(&mut app);

    app.insert_resource(WindowDescriptor {
        title: "LD51".to_string(),
        ..Default::default()
    })
    .insert_resource(ClearColor(Color::Rgba { red: 185.0 / 255.0, green: 229.0 / 255.0, blue: 241.0 / 255.0, alpha: 1.0 }))
    .insert_resource(GameData {
        cooldown_timer: Timer::from_seconds(3.0, false),
        on_cooldown: false,
        threshold: 9.0,
    })
    .insert_resource(MatchmakingTimer {
        timer: Timer::from_seconds(5.0, false),
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(bevy_web_resizer::Plugin)
    .add_plugin(TextureAtlasLoaderPlugin)
    .add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Matchmaking)
            .with_collection::<GameAssets>(),
    )
    .add_state(GameState::Loading)
    .add_system_set(
        SystemSet::on_enter(GameState::Matchmaking)
            .with_system(start_matchbox_socket)
            .with_system(race::setup),
    )
    .add_system_set(SystemSet::on_update(GameState::Matchmaking).with_system(wait_for_players))
    .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(race::spawn_players))
    .add_system_set(SystemSet::on_update(GameState::InGame).with_system(race::camera_control))
    .run();
}

#[cfg(target_arch = "wasm32")]
fn start_matchbox_socket(mut commands: Commands) {
    use bevy::tasks::IoTaskPool;

    let room_url = "wss://snails.nickspeaks.com/next_2";
    info!("Connecting to matchbox server: {:?}", room_url);

    let (socket, message_loop) = WebRtcSocket::new(room_url);
    let task_pool = IoTaskPool::get();
    task_pool.spawn(message_loop).detach();
    commands.insert_resource(Some(socket));
}

fn wait_for_players(
    time: Res<Time>,
    mut timer: ResMut<MatchmakingTimer>,
    mut commands: Commands,
    mut socket: ResMut<Option<WebRtcSocket>>,
    mut state: ResMut<State<GameState>>,
) {
    timer.timer.tick(time.delta());

    let socket = socket.as_mut();

    if socket.is_none() {
        return;
    }

    socket.as_mut().unwrap().accept_new_connections();
    let players = socket.as_ref().unwrap().players();

    let num_players = 3;
    if !timer.timer.finished() {
        if players.len() < num_players {
            return;
        }
    }

    info!("All players connected");

    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(players.len())
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
