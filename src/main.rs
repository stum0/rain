use std::f32::consts::PI;

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_touch_stick::{prelude::*, TouchStickUiKnob, TouchStickUiOutline};
use leafwing_input_manager::prelude::*;

/// Marker type for our touch stick
#[derive(Default, Reflect, Hash, Clone, PartialEq, Eq)]
enum Stick {
    #[default]
    Left,
    Right,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Move,
    Look,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((
            DefaultPlugins,
            // add an inspector for easily changing settings at runtime
            #[cfg(debug_assertions)]
            WorldInspectorPlugin::default(),
            // add the plugin
            TouchStickPlugin::<Stick>::default(),
            // add leafwing plugin
            InputManagerPlugin::<Action>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

#[derive(Component)]
struct Player {
    max_speed: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 5.0),
        ..default()
    });

    // spawn a player
    commands
        .spawn((
            Player { max_speed: 150. },
            InputManagerBundle::<Action> {
                // Stores "which actions are currently activated"
                action_state: ActionState::default(),
                // Describes how to convert from player inputs into those actions
                input_map: InputMap::default()
                    .insert(Action::Move, DualAxis::left_stick())
                    .build(),
            },
            SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0., 0., 0.),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::ORANGE,
                    custom_size: Some(Vec2::new(15., 25.)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // pointy "nose" for player
            parent.spawn(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(5., 0., 0.),
                    rotation: Quat::from_rotation_z(PI / 4.),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::ORANGE,
                    custom_size: Some(Vec2::splat(25. / f32::sqrt(2.))),
                    ..default()
                },
                ..default()
            });
        });

    // spawn a move stick
    commands
        .spawn((
            // map this stick as a left gamepad stick (through bevy_input)
            // leafwing will register this as a normal gamepad
            TouchStickGamepadMapping::LEFT_STICK,
            TouchStickUiBundle {
                stick: TouchStick {
                    id: Stick::Left,
                    radius: 10.0,
                    ..default()
                },
                style: Style {
                    width: Val::Percent(100.0),  // Width of the touchstick area
                    height: Val::Percent(100.0), // Height of the touchstick area
                    position_type: PositionType::Absolute,
                    bottom: Val::Percent(-25.0), // At the bottom of the screen
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TouchStickUiKnob,
                ImageBundle {
                    image: asset_server.load("knob.png").into(),
                    style: Style {
                        width: Val::Px(75.),
                        height: Val::Px(75.),
                        ..default()
                    },
                    ..default()
                },
            ));
            parent.spawn((
                TouchStickUiOutline,
                ImageBundle {
                    image: asset_server.load("outline.png").into(),
                    style: Style {
                        width: Val::Px(150.),
                        height: Val::Px(150.),
                        ..default()
                    },
                    ..default()
                },
            ));
        });
}

fn move_player(
    mut players: Query<(&mut Transform, &ActionState<Action>, &Player)>,
    time: Res<Time>,
) {
    let (mut player_transform, action_state, player) = players.single_mut();

    if action_state.pressed(&Action::Move) {
        let axis_value = action_state.clamped_axis_pair(&Action::Move).unwrap().xy();

        info!("moving: {axis_value}");

        let mut move_delta = axis_value * player.max_speed * time.delta_seconds();
        let length = (move_delta.x.powi(2) + move_delta.y.powi(2)).sqrt();
        if length > 0.0 {
            move_delta.x /= length;
            move_delta.y /= length;
        }

        move_delta *= player.max_speed * time.delta_seconds();
        player_transform.translation += move_delta.extend(0.);

        if axis_value != Vec2::ZERO {
            let dir = Vec2::angle_between(Vec2::X, axis_value.normalize());
            player_transform.rotation = Quat::from_rotation_z(dir);
        }
    }
}
