use std::ops::Mul;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::render::{
    mesh::{Indices, VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};
use bevy_rapier2d::rapier::geometry::ColliderBuilder;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setupv3)
        .add_systems(Update, input_handler)
        .add_systems(Update, move_speeder)
        .add_systems(Update, kill_debris)
        .add_systems(Update, check_collisions)
        .add_systems(Update, warp_space)
        .run();
}

#[derive(Component)]
struct Debris;

#[derive(Component)]
struct Ship {
    player: u8
}

#[derive(Component)]
struct Thruster {
    thruster_time: f32
}

#[derive(Component)]
struct Gun {
    time: f32
}

#[derive(Component)]
struct Speed {
    speed: Vec2,
}

#[derive(Component)]
struct Lifetime {
    death: f32
}

fn create_ship() -> Vec<Vec3>  {
    vec![
        Vec3::new(0.0, 1.0, 0.0), 
        Vec3::new(0.6, -0.3, 0.0), 
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(-0.6,-0.3,0.)
    ]
}

fn create_figter() -> Vec<Vec3>  {
    vec![
        Vec3::new(0.1, 0.1, 0.0), 
        Vec3::new(-0.1, -0.2, 0.0),
        Vec3::new(-0.2, 0.2, 0.0),
        Vec3::new(-0.25, -0.3, 0.0),
        Vec3::new(0., -0.2, 0.0),

        Vec3::new(0.25, -0.3, 0.0),
        Vec3::new(0.2, 0.2, 0.0),
        Vec3::new(0.1, -0.2, 0.0),
        Vec3::new(-0.1, 0.1, 0.0), 
    ]
}

fn create_debris() -> Vec<Vec3> {
    vec![
        Vec3::new(0.0, 0.3, 0.0), 
        Vec3::new(0.3, 0., 0.0), 
        Vec3::new(0.0, -0.3, 0.0),
        Vec3::new(-0.3,0.,0.)
    ]
}

fn create_shot() -> Vec<Vec3> {
    vec![
        Vec3::new(0.0, 0.4, 0.0), 
        Vec3::new(0.1, 0., 0.0), 
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(-0.1,0.,0.)
    ]
}

fn create_mesh(lines: Vec<Vec3>, scale:f32) -> Mesh {
    let lines2:Vec<Vec3> = lines.iter().map(|v|v.mul(scale)).collect();
    let len = lines.len();
    let mut indexes:Vec<u32> = (0..(len as u32)).collect();
    indexes.push(0);
    /* //![0, 1, 2, 3, 0])) */
    Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            lines2
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR, vec![[1.0, 1.0, 1.0, 1.0]; len])
        .with_inserted_indices(
            Indices::U32(indexes))
}

#[derive(Resource)]
struct MeshHandles {
    ship: Handle<Mesh>,
    fighter: Handle<Mesh>,
    debris: Handle<Mesh>,
    shot: Handle<Mesh>,
    material: Handle<ColorMaterial>

}

fn setupv3(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let mesh_handles=MeshHandles {
        ship : meshes.add(create_mesh(create_ship(), 16.)),
        fighter : meshes.add(create_mesh(create_figter(), 32.)),
        debris : meshes.add(create_mesh(create_debris(), 16.)),
        shot : meshes.add(create_mesh(create_shot(), 16.)),
        material: materials.add(ColorMaterial::from(Color::BLUE)),
    };

    commands.spawn(Camera2dBundle::default());

    for (i, pos) in vec![Vec3::new(0.,0.,0.), Vec3::new(100.,100.,0.)].iter().enumerate() {
        commands.spawn((MaterialMesh2dBundle {
            mesh: mesh_handles.ship.clone().into(),
            transform: Transform::default().with_translation(*pos),
            material: mesh_handles.material.clone(),
            ..Default::default()
        },
        Collider::ball(16.0),
        ActiveEvents::CONTACT_FORCE_EVENTS,
        RigidBody::Dynamic,
        GravityScale(0.0),
        Velocity{ linvel:Vec2::new(0.0,0.0), angvel:0.0},
        ExternalImpulse{ impulse:Vec2::new(0., 0.), torque_impulse: 0. },
        Restitution::coefficient(0.7),
        Ship { player:i as u8 },
        Thruster{thruster_time:0.},
        Gun{time:0.},
        ));
    }
    commands.insert_resource(mesh_handles);
}

fn check_collisions(
    mut reader: EventReader<CollisionEvent>,
    mut reader2: EventReader<ContactForceEvent>
) {
    for event in reader.read() {
        dbg!("event {}", event);
    }
    for event in reader2.read() {
        if event.total_force_magnitude>5000000. {
            dbg!("BOOM");
        }
        dbg!("event {}", event);
    }
}

// FIXME: add time
fn move_speeder(
    mut query: Query<(&Speed, &mut Transform)>,
    time: Res<Time>,
) {
    for (speed, mut transform) in &mut query {
        transform.translation+=speed.speed.extend(0.) * time.delta_seconds();
    }
}

fn warp_space(mut query: Query<&mut Transform>) {
    for (mut transform) in &mut query {
        if transform.translation.x < -SPACE_SIZE {
            transform.translation.x += 2. * SPACE_SIZE;
        }
        if transform.translation.y < -SPACE_SIZE {
            transform.translation.y += 2. * SPACE_SIZE;
        }
        if transform.translation.x > SPACE_SIZE {
            transform.translation.x -= 2. * SPACE_SIZE;
        }
        if transform.translation.y > SPACE_SIZE {
            transform.translation.y -= 2. * SPACE_SIZE;
        }
    }
}

fn kill_debris(
    query: Query<(Entity, &Lifetime)>,
    time: Res<Time>,
    mut commands: Commands
) {
    for (entity, lifetime) in &query {
        if lifetime.death< time.elapsed_seconds() {
            commands.entity(entity).despawn();
        }
    }
}

const THRUSTER_TIME:f32 = 0.05;
const THRUSTER_LIFETIME:f32 = 0.5;
const THRUSTER_SPEED:f32 = 200.;
const GUN_TIME:f32 = 0.05;
const GUN_LIFETIME:f32 = 0.5;
const SHOT_SPEED:f32 = 400.;
const SPACE_SIZE:f32 = 400.;

#[derive(Clone)]
struct KeyConfig {
    thrust: KeyCode,
    left: KeyCode,
    right: KeyCode,
    shoot: KeyCode,
    player: u8
}

fn steering_config() -> Vec<KeyConfig> {
    vec![
        KeyConfig{ 
            player:0, 
            thrust: KeyCode::ArrowUp,
            left: KeyCode::ArrowLeft,
            right: KeyCode::ArrowRight,
            shoot: KeyCode::Space,
        },
        KeyConfig{ 
            player:1, 
            thrust: KeyCode::KeyW,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            shoot: KeyCode::KeyS,
        },
    ]
}

fn get_key_config_for(player: u8) -> Option<KeyConfig> {
    steering_config().iter().find(|&x| x.player == player).map(|c|c.clone()).into()
}

fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut ExternalImpulse, &mut Transform, &mut Thruster, &mut Gun, &Ship)>,
    time: Res<Time>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut commands: Commands,
    mesh_handles: Res<MeshHandles>
) {
    for (mut speed, mut impulse, mut transform, mut thruster, mut gun, ship) in &mut query {

        if let Some(keys) = get_key_config_for(ship.player) {

            if keyboard_input.pressed(keys.shoot) {
                let r = transform.rotation.to_euler(EulerRot::XYZ);

                let rnd = rand::random::<f32>()*0.3-0.15;
                let v = Vec2::from_angle(rnd + r.2+3.1415/2.);

                // speed_up
                gun.time+=time.delta_seconds();
                if gun.time > GUN_TIME {

                    gun.time -= GUN_TIME;

                    commands.spawn((MaterialMesh2dBundle {
                        mesh: mesh_handles.shot.clone().into(),
                        transform: Transform::default()
                            .with_rotation(transform.rotation)
                            .with_translation(transform.translation),
                            material: mesh_handles.material.clone(),
                            ..Default::default()
                    },
                    Debris{},
                    Speed{speed:speed.linvel+v* SHOT_SPEED },
                    Lifetime{ death: time.elapsed_seconds() + GUN_LIFETIME}
                    ));

                }

            }
            if keyboard_input.pressed(keys.thrust) {
                let r = transform.rotation.to_euler(EulerRot::XYZ);

                let rnd = rand::random::<f32>()*0.3-0.15;
                let v = Vec2::from_angle(rnd + r.2+3.1415/2.);
                //speed.linvel+=v*100. * time.delta_seconds();
                impulse.impulse =v*100000. * time.delta_seconds(); 

                // speed_up
                thruster.thruster_time+=time.delta_seconds();
                if thruster.thruster_time > THRUSTER_TIME {

                    thruster.thruster_time -= THRUSTER_TIME;

                    commands.spawn((MaterialMesh2dBundle {
                        mesh: mesh_handles.debris.clone().into(),
                        transform: Transform::default()
                            .with_translation(transform.translation),
                            material: mesh_handles.material.clone(),
                            ..Default::default()
                    },
                    Debris{},
                    Speed{speed:speed.linvel-v* THRUSTER_SPEED },
                    Lifetime{ death: time.elapsed_seconds() + THRUSTER_LIFETIME }
                    ));

                }

            }
            if keyboard_input.pressed(keys.left) {
                speed.angvel=5.;
//                transform.rotate_z(time.delta_seconds() *5.);
            } else if keyboard_input.pressed(keys.right) {
                speed.angvel=-5.;
            //    transform.rotate_z(-time.delta_seconds() *5.);
            } else {
                speed.angvel=0.;
            }

        }
    }
    if keyboard_input.pressed(KeyCode::KeyQ) {
        app_exit_events.send(bevy::app::AppExit);
    }
}

