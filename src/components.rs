use bevy::{prelude::*, time::Stopwatch};
use bevy_asset_loader::prelude::*;

#[derive(Component)]
pub struct Player {
    pub cooldown_timer: Timer,
    pub on_cooldown: bool,
    pub timing_index: usize,
}

#[derive(Component, Default, Reflect)]
pub struct PlayerTimer {
    pub timer: Timer,
    pub stopwatch: Stopwatch
}

#[derive(Component, Default, Reflect)]
pub struct PlayerTarget {
    pub x: f32,
}

#[derive(Component)]
pub struct PlayerLocal;

#[derive(Component, Deref)]
pub struct Animation(pub benimator::Animation);

#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(pub benimator::State);

//Resources
pub struct LocalPlayerHandle(usize);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Loading,
    Matchmaking,
    InGame,
    End,
}

pub struct GameData {
    pub camera: Option<Entity>
}

#[derive(AssetCollection)]
pub struct GameAssets {
    #[asset(path = "fonts/Courier New Bold.ttf")]
    pub font: Handle<Font>,
    #[asset(path = "textures/ground-grass-seamless_00.png")]
    pub grass: Handle<Image>,
    #[asset(path = "textures/ground-finish-seamless_00.png")]
    pub finish: Handle<Image>,
    #[asset(path = "textures/match-indicator_target.png")]
    pub indicator_target: Handle<Image>,
    #[asset(path = "textures/match-indicator_cursor.png")]
    pub indicator_cursor: Handle<Image>,
    #[asset(path = "textures/alert_miss.png")]
    pub alert_miss: Handle<Image>,
    #[asset(path = "textures/alert_ok.png")]
    pub alert_ok: Handle<Image>,
    #[asset(path = "textures/alert_good.png")]
    pub alert_good: Handle<Image>,
    #[asset(path = "textures/alert_perfect.png")]
    pub alert_perfect: Handle<Image>,
    #[asset(path = "textures/menu-tryagain.png")]
    pub try_again: Handle<Image>,
    #[asset(path = "snail_idle.ron")]
    pub snail_idle: Handle<TextureAtlas>,
    #[asset(path = "hidethesalt.ogg")]
    pub music: Handle<AudioSource>,
    #[asset(path = "match.ogg")]
    pub match_sound: Handle<AudioSource>,
    #[asset(path = "cheering crowd.ogg")]
    pub crowd_sound: Handle<AudioSource>,
}