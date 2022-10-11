use bevy::{prelude::*, render::camera::ScalingMode, sprite::Anchor};
use matchbox_socket::WebRtcSocket;
use crate::{GameData, GameAssets, GameState};

#[derive(Component)]
pub struct ConnectedText;

pub struct Lobby {
    force: bool,
    node: Entity
}

pub fn setup(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut data: ResMut<GameData>,
) {
    //Setup Camera
    if data.camera.is_none() {
        let mut camera = Camera2dBundle::default();
        camera.projection.scaling_mode = ScalingMode::FixedVertical(1080.0);
        let cam_id = commands.spawn_bundle(camera);
        data.camera = Some(cam_id.id());

        //Grass Background
        for i in 0..=30 {
            commands.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                texture: match i {
                    26 => assets.finish.clone(),
                    _  => assets.grass.clone()
                },
                transform: Transform::from_translation(Vec3::new(
                    1920.0 * -1.5 + (i as f32) * 500.0,
                    -540.0,
                    0.0,
                )),
                ..default()
            });
        }
    }


    //Continue Button
    let mut node = commands.spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        });

    node.with_children(|parent| {
        parent.spawn_bundle(
            TextBundle::from_section(
                "Waiting for another player...",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            )
        );

        parent.spawn_bundle(
            TextBundle::from_section(
                "Connected: 1/2",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            )
        ).insert(ConnectedText);

        parent.spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(200.0), Val::Px(65.0)),
                // center button
                margin: UiRect::all(Val::Px(20.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::ORANGE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Play Now!",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });
    });

    let node_id = node.id().clone();
    commands.insert_resource(Lobby { force: false, node: node_id });
}

pub fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut force_start: ResMut<Lobby>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = Color::ORANGE_RED.into();
                force_start.force = true;
            }
            Interaction::Hovered => {
                *color = Color::GRAY.into();
            }
            Interaction::None => {
                *color = Color::ORANGE.into();
            }
        }
    }
}

pub fn start_matchbox_socket() {
    let room_url = "wss://snails.nickspeaks.com/next_2";
    info!("Connecting to matchbox server: {:?}", room_url);

    //let (socket, message_loop) = WebRtcSocket::new(room_url);
    //let task_pool = IoTaskPool::get();
    //task_pool.spawn(message_loop).detach();
    //commands.insert_resource(Some(socket));
}

pub fn wait_for_players(
    mut state: ResMut<State<GameState>>,
    mut text: Query<&mut Text, With<ConnectedText>>,
    lobby: Res<Lobby>,
) {
    if lobby.force == false {
        text.single_mut().sections[0].value = format!("Connected: {}/{}", 1, 1);
        return;
    }

    info!("All players connected");
    state.set(GameState::InGame).unwrap();
}

pub fn cleanup(
    mut commands: Commands,
    lobby: Res<Lobby>,

) {
    commands.remove_resource::<Option<WebRtcSocket>>();
    commands.remove_resource::<Lobby>();
    commands.entity(lobby.node).despawn_recursive();
}