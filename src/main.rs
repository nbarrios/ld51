mod input;
mod lobby;
mod race;
mod end_menu;

use bevy::{prelude::*, time::Stopwatch};
use bevy_asset_loader::prelude::*;
use bevy_heterogeneous_texture_atlas_loader::*;

pub struct LocalPlayerHandle(usize);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Loading,
    Matchmaking,
    InGame,
    End,
}

#[derive(AssetCollection)]
pub struct GameAssets {
    #[asset(path = "fonts/Courier New Bold.ttf")]
    font: Handle<Font>,
    #[asset(path = "textures/ground-grass-seamless_00.png")]
    grass: Handle<Image>,
    #[asset(path = "textures/ground-finish-seamless_00.png")]
    finish: Handle<Image>,
    #[asset(path = "textures/match-indicator_target.png")]
    indicator_target: Handle<Image>,
    #[asset(path = "textures/match-indicator_cursor.png")]
    indicator_cursor: Handle<Image>,
    #[asset(path = "textures/alert_miss.png")]
    alert_miss: Handle<Image>,
    #[asset(path = "textures/alert_ok.png")]
    alert_ok: Handle<Image>,
    #[asset(path = "textures/alert_good.png")]
    alert_good: Handle<Image>,
    #[asset(path = "textures/alert_perfect.png")]
    alert_perfect: Handle<Image>,
    #[asset(path = "textures/menu-tryagain.png")]
    try_again: Handle<Image>,
    #[asset(path = "snail_idle.ron")]
    snail_idle: Handle<TextureAtlas>,
    #[asset(path = "hidethesalt.ogg")]
    music: Handle<AudioSource>,
    #[asset(path = "match.ogg")]
    match_sound: Handle<AudioSource>,
    #[asset(path = "cheering crowd.ogg")]
    crowd_sound: Handle<AudioSource>,
}

#[derive(Component)]
pub struct Player {
    cooldown_timer: Timer,
    on_cooldown: bool,
    timing_index: usize,
}

#[derive(Component, Default, Reflect)]
pub struct PlayerTimer {
    timer: Timer,
    stopwatch: Stopwatch
}

#[derive(Component, Default, Reflect)]
pub struct PlayerTarget {
    x: f32,
}

#[derive(Component)]
pub struct PlayerLocal;

pub struct GameData {
    camera: Option<Entity>
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

    app.insert_resource(WindowDescriptor {
        title: "LD51".to_string(),
        canvas: Some("#bevy".to_owned()),
        ..Default::default()
    })
    .insert_resource(ClearColor(Color::Rgba { red: 185.0 / 255.0, green: 229.0 / 255.0, blue: 241.0 / 255.0, alpha: 1.0 }))
    .insert_resource(GameData {
        camera: None,
    })
    .add_event::<race::Alert>()
    .add_plugins(DefaultPlugins)
    //.add_plugin(bevy_web_resizer::Plugin)
    .add_plugin(TextureAtlasLoaderPlugin)
    .add_loading_state(
        LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Matchmaking)
            .with_collection::<GameAssets>(),
    )
    .add_state(GameState::Loading)
    .add_system_set(
        SystemSet::on_enter(GameState::Matchmaking)
            .with_system(lobby::setup)
            .with_system(lobby::start_matchbox_socket)
    )
    .add_system_set(
        SystemSet::on_update(GameState::Matchmaking)
            .with_system(lobby::wait_for_players)
            .with_system(lobby::button_system)
    )
    .add_system_set(
        SystemSet::on_exit(GameState::Matchmaking)
            .with_system(lobby::cleanup)
    )
    .add_system_set(
        SystemSet::on_enter(GameState::InGame)
            .with_system(race::setup)
            .with_system(race::spawn_players))
    .add_system_set(
        SystemSet::on_update(GameState::InGame)
            .with_system(race::update)
            .with_system(race::update_rollback)
            .with_system(race::feedback_spawn)
            .with_system(race::feedback_update)
            .with_system(race::tick_timers)
            .with_system(race::camera_control)
    )
    .add_system_set(
        SystemSet::on_exit(GameState::InGame)
            .with_system(race::cleanup)
    )
    .add_system_set(SystemSet::on_enter(GameState::End).with_system(end_menu::setup))
    .add_system_set(SystemSet::on_update(GameState::End).with_system(end_menu::update))
    .add_system(race::animate)
    .run();
}
