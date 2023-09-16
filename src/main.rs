//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                fire_thrusters.after(apply_velocity),
            ),
        )
        .run();
}

#[derive(Bundle)]
struct SpaceshipBundle {
    ship_marker: Spaceship,
    model: PbrBundle,
    vel: Velocity,
    gravity: Gravity,
    body: RigidBody,
    collider: Collider,
    restitution: Restitution,
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec3);

#[derive(Component)]
struct Spaceship;

#[derive(Component)]
struct Gravity;

#[derive(Component)]
struct Ground;

fn fire_thrusters(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Spaceship>>,
    time_step: Res<FixedTime>,
) {
    let mut transform = query.single_mut();

    if keyboard_input.pressed(KeyCode::Space) {
        transform.y += 10.0 * time_step.period.as_secs_f32();
    }
}

// TODO: DELETE
fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time_step: Res<FixedTime>) {
    for (mut trans, vel) in &mut query {
        trans.translation.x += vel.x * time_step.period.as_secs_f32();
        trans.translation.y += vel.y * time_step.period.as_secs_f32();
        trans.translation.z += vel.z * time_step.period.as_secs_f32();
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(5.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_scale(Vec3 {
                x: 5.0,
                y: 1.0,
                z: 5.0,
            }),
            ..default()
        },
        Ground,
        Collider::cuboid(5.0, 0.1, 5.0),
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
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

impl SpaceshipBundle {
    fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            ship_marker: Spaceship,
            model: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 0.5, 0.0).with_scale(Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                }),
                ..default()
            },
            vel: Velocity(Vec3::default()),
            gravity: Gravity,
            body: RigidBody::Dynamic,
            collider: Collider::ball(0.5),
            restitution: Restitution::coefficient(0.1),
        }
    }
}

fn collides(a_pos: Vec3, a_size: Vec3, b_pos: Vec3, b_size: Vec3) -> bool {
    let a_min = a_pos - a_size / 2.0;
    let a_max = a_pos + a_size * 2.0;
    let b_min = b_pos - b_size / 2.0;
    let b_max = b_pos + b_size / 2.0;

    let axis_collides =
        |axis: fn(Vec3) -> f32| axis(a_min) < axis(b_max) && axis(a_max) > axis(b_min);

    axis_collides(|v| v.x) && axis_collides(|v| v.y) && axis_collides(|v| v.z)
}
