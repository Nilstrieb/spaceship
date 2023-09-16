//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{
    audio::PlaybackMode,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                fire_thrusters,
                orbit_camera,
                apply_gravity,
                bevy::window::close_on_esc,
            ),
        )
        .run();
}

#[derive(Bundle)]
struct SpaceshipBundle {
    ship_marker: Spaceship,
    model: PbrBundle,
    vel: Velocity,
    body: RigidBody,
    collider: Collider,
    restitution: Restitution,
    mass: Mass,
    thrusters: Thrusters,
    thruster_force: ExternalForce,
}

#[derive(Component)]
struct Spaceship;

/// Mass in kg.
#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct Thrusters {
    /// Strength in some units
    strength: f32,
}

#[derive(Component)]
struct HasGravity {
    mass: f32,
}

#[derive(Component)]
struct OrbitCamera {
    radius: f32,
}

#[derive(Component)]
struct ThrusterSound;

fn fire_thrusters(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut ExternalForce, &Thrusters)>,
    sound_query: Query<&AudioSink, With<ThrusterSound>>,
    asset_server: Res<AssetServer>,
) {
    let (mut force, thrusters) = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(sound) = sound_query.get_single() {
            sound.play();
        } else {
            commands.spawn((
                AudioBundle {
                    source: asset_server.load("thrusters_loop.ogg"),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Loop,
                        ..default()
                    },
                },
                ThrusterSound,
            ));
        }
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        force.force = Vec3 {
            x: 0.0,
            y: thrusters.strength,
            z: 0.0,
        };
    } else if keyboard_input.just_released(KeyCode::Space) {
        if let Ok(sound) = sound_query.get_single() {
            sound.pause();
        }
        force.force = Vec3::ZERO;
    }
}

fn apply_gravity(query: Query<&mut ExternalForce, With<Spaceship>>) {}

// adapted from https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html
fn orbit_camera(
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut query: Query<(&mut OrbitCamera, &mut Transform), Without<Spaceship>>,
    spaceship_query: Query<&Transform, With<Spaceship>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    let rotation_move: Vec2 = ev_motion.iter().map(|ev| ev.delta).sum();
    let scroll: f32 = ev_scroll.iter().map(|ev| ev.y).sum();

    for (mut orbit, mut transform) in &mut query {
        if rotation_move.length_squared() > 0.0 {
            let window = Vec2::new(window.width(), window.height());
            let delta_x = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation *= yaw;
            transform.rotation *= pitch;
        }
        if scroll.abs() > 0.0 {
            orbit.radius -= scroll * orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            orbit.radius = f32::max(orbit.radius, 0.05);
        }

        let rot_matrix = Mat3::from_quat(transform.rotation);
        transform.translation = spaceship_query.single().translation
            + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, orbit.radius));
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rapier = RapierConfiguration::default();
    // We ain't a normal game, we do our own gravity.
    rapier.gravity = Vec3::new(0.0, -0.5, 0.0);
    commands.insert_resource(rapier);

    commands.spawn(PlanetBundle::new(
        &mut meshes,
        &mut materials,
        Transform::from_xyz(0.0, -100.0, 0.0),
        100.0,
        0.0,
        Color::rgb(0.2, 0.8, 0.5),
    ));

    commands.spawn(SpaceshipBundle::new(&mut meshes, &mut materials));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera

    let camera_translation = Vec3::new(-2.0, 2.5, 5.0);
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(camera_translation)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitCamera {
            radius: camera_translation.length(),
        },
    ));
}

impl SpaceshipBundle {
    fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let height = 4.0;
        let width = 0.5;

        Self {
            ship_marker: Spaceship,
            model: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(width, height, width))),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 3.0, 0.0).with_scale(Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
                ..default()
            },
            vel: Velocity {
                linvel: Vec3::ZERO,
                angvel: Vec3::ZERO,
            },
            mass: Mass(1000.0),
            body: RigidBody::Dynamic,
            collider: Collider::cuboid(width / 2.0, height / 2.0, width / 2.0),
            restitution: Restitution::coefficient(0.1),
            thrusters: Thrusters { strength: 1.0 },
            thruster_force: ExternalForce {
                force: Vec3::new(0.0, -0.5, 0.0), // gravity
                torque: Vec3::ZERO,
            },
        }
    }
}

#[derive(Bundle)]
struct PlanetBundle {
    mesh: PbrBundle,
    coll: Collider,
    gravity: HasGravity,
}

impl PlanetBundle {
    fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        position: Transform,
        radius: f32,
        mass: f32,
        color: Color,
    ) -> Self {
        PlanetBundle {
            mesh: PbrBundle {
                mesh: meshes.add(
                    shape::UVSphere {
                        radius,
                        sectors: 100,
                        stacks: 100,
                    }
                    .into(),
                ),
                material: materials.add(color.into()),
                transform: position,
                ..default()
            },
            coll: Collider::ball(radius),
            gravity: HasGravity { mass },
        }
    }
}
