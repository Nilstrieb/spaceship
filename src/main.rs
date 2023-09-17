mod forces;
mod orbit;

use bevy::{
    audio::PlaybackMode,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_rapier3d::prelude::*;
use forces::ExternalForceSet;
use glam::DVec2;

use crate::forces::update_external_forces;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_external_forces,
                fire_thrusters,
                orbit_camera,
                apply_gravity.before(update_external_forces),
                debug_spaceship_orbit,
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
    thrusters: Thrusters,
    thruster_force: ExternalForce,
    forces: ExternalForceSet,
    light: PointLight,
}

#[derive(Component)]
struct Spaceship;

#[derive(Component)]
struct Thrusters {
    /// Strength in some units
    strength: f32,
}

#[derive(Component)]
struct GravityAttractor {
    mass: f64,
}

#[derive(Component)]
struct OrbitCamera {
    radius: f32,
}

#[derive(Component)]
struct ThrusterSound;

const AMOUNT_OF_FUNNY_ORBIT_SPHERES: u32 = 1000;

fn fire_thrusters(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut ExternalForceSet, &Transform, &Thrusters)>,
    sound_query: Query<&AudioSink, With<ThrusterSound>>,
    asset_server: Res<AssetServer>,
) {
    struct ThrusterForce;

    let (mut force_set, transform, thrusters) = query.single_mut();

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
    } else if keyboard_input.just_released(KeyCode::Space) {
        if let Ok(sound) = sound_query.get_single() {
            sound.pause();
        }
    }

    let rotation = Mat3::from_quat(transform.rotation);

    let mut force = force_set.get::<ThrusterForce>();

    if keyboard_input.pressed(KeyCode::Space) {
        force.force = rotation.mul_vec3(Vec3::new(0.0, thrusters.strength, 0.0));
    } else {
        force.force = Vec3::ZERO;
    }

    let torque = 0.2;
    let keybinds = [
        (KeyCode::W, Vec3::new(torque, 0.0, 0.0)),
        (KeyCode::S, Vec3::new(-torque, -0.0, 0.0)),
        (KeyCode::Q, Vec3::new(0.0, torque, 0.0)),
        (KeyCode::E, Vec3::new(0.0, -torque, 0.0)),
        (KeyCode::A, Vec3::new(0.0, 0.0, torque)),
        (KeyCode::D, Vec3::new(0.0, -0.0, -torque)),
    ];

    let mut any_pressed = false;
    for (bind, vec) in keybinds {
        if keyboard_input.pressed(bind) {
            any_pressed = true;
            force.torque = rotation.mul_vec3(vec);
        }
    }
    if !any_pressed {
        force.torque = Vec3::ZERO;
    }

    force_set.set::<ThrusterForce>(force);
}

fn apply_gravity(
    mut query: Query<(&mut ExternalForceSet, &Transform), With<Spaceship>>,
    body_query: Query<(&GravityAttractor, &Transform), Without<Spaceship>>,
) {
    struct GravityForce;

    let (mut ship_forces, ship_transform) = query.single_mut();

    for (gravity, body_transform) in &body_query {
        let distance = ship_transform
            .translation
            .distance(body_transform.translation) as f64;

        let fg = (orbit::G * gravity.mass) / (distance * distance);
        let direction = (body_transform.translation - ship_transform.translation).normalize();

        let fg = ExternalForce {
            force: direction * (fg as f32),
            torque: Vec3::ZERO,
        };

        ship_forces.set::<GravityForce>(fg);
    }
}

#[derive(Component)]
struct OrbitText;

fn debug_spaceship_orbit(
    query: Query<(&Transform, &Velocity), With<Spaceship>>,
    body_query: Query<(&Transform, &GravityAttractor), Without<Spaceship>>,
    mut text_query: Query<&mut Text, With<OrbitText>>,
    mut gizmos: Gizmos,
    mut query_sphere: Query<
        &mut Transform,
        (
            With<FunnyOrbitalSphere>,
            Without<OrbitText>,
            Without<Spaceship>,
            Without<GravityAttractor>,
        ),
    >,
) {
    let mut text = text_query.single_mut();
    let (ship_transform, &v) = query.single();

    let ship_pos = ship_transform.translation;
    let (body_transform, body_gravity) = body_query.single();
    let body_pos = body_transform.translation;

    let body_rotation = body_transform.rotation;
    let body_axis = body_rotation * Vec3::Y;

    gizmos.ray(body_pos, body_axis * 150.0, Color::GOLD);
    gizmos.ray(body_pos, -body_axis * 150.0, Color::GOLD);

    let velocity = v.linvel;
    let translation = ship_pos - body_pos;

    let orbital_plane_normal = velocity.cross(translation).normalize_or_zero() * 10.0;
    gizmos.ray(ship_pos, orbital_plane_normal, Color::PINK);

    let orbital_plane_rot = Quat::from_rotation_arc(
        orbital_plane_normal.try_normalize().unwrap_or(Vec3::X),
        Vec3::Y,
    );

    let rotated_vel = orbital_plane_rot * velocity;
    let rotated_pos = orbital_plane_rot * translation;

    gizmos.ray(body_pos, rotated_pos, Color::FUCHSIA);
    gizmos.ray(
        body_pos,
        rotated_vel.normalize_or_zero() * 12000.0,
        Color::OLIVE,
    );

    let orbit = orbit::Orbit::from_pos_dir(
        body_gravity.mass,
        DVec2::new(rotated_pos.x.into(), rotated_pos.z.into()),
        DVec2::new(rotated_vel.x.into(), rotated_vel.z.into()),
    );
    text.sections[1].value = format!("{:.2}", orbit.semi_major_axis);
    text.sections[3].value = format!("{:.2}", orbit.apoapsis());
    text.sections[5].value = format!("{:.2}", orbit.periapsis());

    gizmos.ray_gradient(ship_pos, velocity, Color::RED, Color::GREEN);
    gizmos.ray_gradient(ship_pos, translation, Color::BLUE, Color::GREEN);

    gizmos.line(body_transform.translation, ship_pos, Color::WHITE);

    let base_pos = body_pos;
    let distance = (orbit.semi_major_axis as f32) * 2.0;
    for (i, mut sphere) in query_sphere.iter_mut().enumerate() {
        let angle = std::f32::consts::TAU / (AMOUNT_OF_FUNNY_ORBIT_SPHERES as f32) * (i as f32);

        let pos = Vec3::new(angle.cos(), 0.0, angle.sin()) * distance;
        let rotated = base_pos + -orbital_plane_rot * (pos - base_pos);

        sphere.translation = base_pos + rotated;
    }
}

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

#[derive(Component)]
struct FunnyOrbitalSphere;

/// set up a simple 3D scene
fn setup(
    // mut windows: Query<&mut Window>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // We ain't a normal game, we do our own gravity.
    commands.insert_resource(RapierConfiguration {
        gravity: Vec3::ZERO,
        ..default()
    });

    commands.spawn(PlanetBundle::new(
        &mut meshes,
        &mut materials,
        &asset_server,
        Transform::from_xyz(0.0, -100.0, 0.0),
        10000.0,
    ));

    commands.spawn(SpaceshipBundle::new(
        &mut meshes,
        &mut materials,
        Vec3::new(0.0, 100.0, 0.0),
    ));

    for _ in 0..AMOUNT_OF_FUNNY_ORBIT_SPHERES {
        commands.spawn((
            FunnyOrbitalSphere,
            PbrBundle {
                mesh: meshes.add(
                    shape::UVSphere {
                        radius: 100.0,
                        ..default()
                    }
                    .into(),
                ),
                ..default()
            },
        ));
    }

    // light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
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

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Semi Major Axis: ",
                TextStyle {
                    font_size: 20.0,
                    color: Color::GRAY,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: 20.0,
                color: Color::GRAY,
                ..default()
            }),
            TextSection::new(
                "\nApoapsis: ",
                TextStyle {
                    font_size: 20.0,
                    color: Color::GRAY,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: 20.0,
                color: Color::GRAY,
                ..default()
            }),
            TextSection::new(
                "\nPeriapsis: ",
                TextStyle {
                    font_size: 20.0,
                    color: Color::GRAY,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: 20.0,
                color: Color::GRAY,
                ..default()
            }),
        ]),
        OrbitText,
    ));

    // let mut window = windows.single_mut();
    // window.cursor.visible = false;
    // window.cursor.grab_mode = CursorGrabMode::Locked;
}

impl SpaceshipBundle {
    fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>, pos: Vec3) -> Self {
        let height = 4.0;
        let width = 0.5;

        SpaceshipBundle {
            ship_marker: Spaceship,
            model: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(width, height, width))),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_translation(pos),
                ..default()
            },
            vel: Velocity {
                linvel: Vec3::ZERO,
                angvel: Vec3::ZERO,
            },
            body: RigidBody::Dynamic,
            collider: Collider::cuboid(width / 2.0, height / 2.0, width / 2.0),
            restitution: Restitution::coefficient(0.1),
            thrusters: Thrusters { strength: 1.0 },
            thruster_force: ExternalForce {
                force: Vec3::new(0.0, -0.5, 0.0), // gravity
                torque: Vec3::ZERO,
            },
            forces: ExternalForceSet::default(),
            light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
struct PlanetBundle {
    mesh: PbrBundle,
    coll: Collider,
    gravity: GravityAttractor,
}

impl PlanetBundle {
    fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        assert_server: &AssetServer,
        position: Transform,
        radius: f64,
    ) -> Self {
        use std::f64::consts::PI;

        let density = 2000.0 /* kg*m^3 */;

        let mass = (4.0 / 3.0) * PI * radius * radius * radius * density;

        let texture_handle = assert_server.load("2k_moon.png");
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            ..default()
        });

        PlanetBundle {
            mesh: PbrBundle {
                mesh: meshes.add(
                    shape::UVSphere {
                        radius: radius as f32,
                        sectors: 100,
                        stacks: 100,
                    }
                    .into(),
                ),
                material,
                transform: position,
                ..default()
            },
            coll: Collider::ball(radius as f32),
            gravity: GravityAttractor { mass },
        }
    }
}
