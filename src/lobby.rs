use bevy::prelude::*;
use bevy_ggrs::*;
use matchbox_socket::WebRtcSocket;
use crate::{GameState, MatchmakingTimer, LocalPlayerHandle};

pub struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    // 4-directions + fire fits easily in a single byte
    type Input = u8;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are strings
    type Address = String;
}

pub fn setup(
    mut commands: Commands
) {

}

#[cfg(target_arch = "wasm32")]
pub fn start_matchbox_socket(mut commands: Commands) {
    use bevy::tasks::IoTaskPool;

    let room_url = "wss://snails.nickspeaks.com/next_2";
    info!("Connecting to matchbox server: {:?}", room_url);

    let (socket, message_loop) = WebRtcSocket::new(room_url);
    let task_pool = IoTaskPool::get();
    task_pool.spawn(message_loop).detach();
    commands.insert_resource(Some(socket));
}

pub fn wait_for_players(
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
        .with_input_delay(0);

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