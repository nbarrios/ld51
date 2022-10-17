mod components;
mod input;
mod lobby;
mod race;
mod end_menu;

use bevy::{prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_heterogeneous_texture_atlas_loader::*;
use components::*;


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
    .add_event::<race::PlayerEvent>()
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
            .with_system(race::change_animation)
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
