use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use rand::Rng;
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::input::ElementState;
use bevy::render::camera::{ActiveCameras, PerspectiveProjection};
use std::ops::Mul;
use bevy::input::keyboard::KeyboardInput;

use bevy::reflect::TypeUuid;

pub const BULLET_MESH_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Mesh::TYPE_UUID, 13148362314412771389);
pub const BULLET_MATERIAL_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(StandardMaterial::TYPE_UUID, 13148362314412771390);

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "CS434 lab1".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup.system())
        .add_system(fan_rotation_system.system())
        .add_system(mouse_fin_bullet_system.system())
        .add_system(bullet_windmill_destruction_system.system())
        .run();
}

struct Windmill {
    state: usize,
    fins: [Option<Entity>; 3]
}
struct WindmillFin {
    index: usize,
}
struct Bullet {
    dir: Vec3
}
struct Scores {
    score: usize,
}

/// set up a simple 3D scene
fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world

    let mut rng = rand::thread_rng();
    let windmill_rod_mesh = meshes.add(Mesh::from(shape::Capsule {
        radius: 0.1,
        rings: 5,
        depth: 5.0,
        latitudes: 16,
        longitudes: 32,
        uv_profile: Default::default()
    }));
    let windmill_fan_mesh = meshes.add(Mesh::from(shape::Capsule {
        radius: 0.05,
        rings: 5,
        depth: 1.0,
        latitudes: 16,
        longitudes: 32,
        uv_profile: Default::default()
    }));

    meshes.set_untracked(BULLET_MESH_HANDLE, Mesh::from(shape::Icosphere { radius: 0.05, subdivisions: 12 }));
    materials.set_untracked(BULLET_MATERIAL_HANDLE, Color::rgb(1.0, 0.0, 0.0).into());

    let windmill_material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    for i in 0..10 {
        let x = rng.gen_range(-15.0..15.0);
        let z = rng.gen_range(-15.0..15.0);
        let mut fins: [Option<Entity>; 3] = [None; 3];
        for i in 0..3 {
            commands.spawn(PbrBundle {
                mesh: windmill_fan_mesh.clone(),
                material: windmill_material.clone(),
                transform: Transform::from_xyz(x, 2.0, z),
                ..Default::default()
            })
                .with(WindmillFin {
                    index: i,
                });;
            fins[i] = commands.current_entity();
        }
        commands.spawn(PbrBundle {
            mesh: windmill_rod_mesh.clone(),
            material: windmill_material.clone(),
            transform: Transform::from_xyz(x, 2.0, z),
            ..Default::default()
        })
            .with(Windmill {
                state: 0,
                fins
            });
    }
    commands
        // plane
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        // light
        .spawn(LightBundle {
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..Default::default()
        })
        .insert_resource(Scores {
            score: 0
        })
        // camera
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0)
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        })
        .with(FlyCamera::default());
}

fn fan_rotation_system(
    time: Res<Time>,
    windmill_query: Query<(&Windmill, &Transform)>,
    mut windmill_fins_query: Query<(&WindmillFin, &mut Transform)>
) {
    for (windmill, windmill_transform) in windmill_query.iter() {
       for fin_entity in windmill.fins.iter() {
           if let Some(entity) = fin_entity {
               let (fin, mut fin_transform) = windmill_fins_query.get_mut(*entity).unwrap();
               let angle = time.seconds_since_startup() as f32 + (fin.index as f32 * std::f32::consts::FRAC_PI_3 * 2.0);
               fin_transform.rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), angle);
               fin_transform.translation = windmill_transform.translation;
               fin_transform.translation.y += 2.5;
               fin_transform.translation.x -= angle.sin() * 0.5;
               fin_transform.translation.y += angle.cos() * 0.5;
           }
       }
    }
}

fn mouse_fin_bullet_system(
    mut commands: &mut Commands,
    mut windows: ResMut<Windows>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    active_cameras: Res<ActiveCameras>,
    camera_query: Query<(&Transform), With<PerspectiveProjection>>,
) {
    let camera = if let Some(camera) = active_cameras.get("Camera3d") {
        camera
    } else {
        return;
    };
    let mut spawn_bullet = || {
        let camera_transform = camera_query.get(camera).unwrap();
        let ray = camera_transform.rotation.mul(Vec3::new(0.0, 0.0, -1.0));

        commands.spawn(PbrBundle {
            mesh: BULLET_MESH_HANDLE.typed(),
            material: BULLET_MATERIAL_HANDLE.typed(),
            transform: Transform {
                translation: camera_transform.translation,
                ..Transform::default()
            },
            ..Default::default()
        })
            .with(Bullet {
                dir: ray,
            });
    };
    let window = windows.get_primary_mut().unwrap();
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ElementState::Pressed && key_code == KeyCode::Escape {
                window.set_cursor_lock_mode(false);
                window.set_cursor_visibility(true);
            }
            else if event.state == ElementState::Pressed && key_code == KeyCode::Return {
                spawn_bullet();
                return;
            }
        }
    }



    for event in mouse_button_input_events.iter() {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
        match event {
            MouseButtonInput {
                button: MouseButton::Left,
                state: ElementState::Pressed,
            } => {
                spawn_bullet();
                return;
            },
            _ => (),
        }
    }
}

fn bullet_windmill_destruction_system(
    mut commands: &mut Commands,
    time: Res<Time>,
    mut bullet_query: Query<(Entity, &mut Bullet, &mut Transform)>,
    mut windmill_query: Query<(Entity, &mut Windmill, &Transform)>,
    mut scores: ResMut<Scores>
) {
    for (bullet_entity, mut bullet, mut transform) in bullet_query.iter_mut() {
        transform.translation += bullet.dir * time.delta_seconds() * 5.0;
        if transform.translation.y < 0.0 || transform.translation.z.abs() > 25.0 || transform.translation.x.abs() > 25.0 {
            commands.despawn(bullet_entity);
            continue;
        }
        for (windmill_entity, mut windmill, windmill_transform) in windmill_query.iter_mut() {
            if (windmill_transform.translation.x - transform.translation.x).abs() < 1.0 &&
                (windmill_transform.translation.y - transform.translation.y).abs() < 3.0 &&
                (windmill_transform.translation.z - transform.translation.z).abs() < 1.0 {


                let fin_to_destroy_index = windmill.state;
                if fin_to_destroy_index == 3 {
                    commands.despawn(windmill_entity);
                    commands.despawn(bullet_entity);
                    scores.score += 1;
                    println!("Current score: {}", scores.score);
                    if scores.score == 10 {
                        println!("You win!");
                    }
                    break;
                }
                bullet.dir = -bullet.dir;
                let fin_to_destroy = windmill.fins[fin_to_destroy_index].take().unwrap();
                windmill.state += 1;
                commands.despawn(fin_to_destroy);
                break;
            }
        }
    }
}