mod components;

use std::f32::consts::PI;
use std::ops::Mul;

use bevy::prelude::*;

use bevy::sprite::MaterialMesh2dBundle;

use bevy::render::{
    mesh::Indices,
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};
use bevy_rapier2d::prelude::*;

use crate::components::*;

const HEAL_SPEED: f32=0.2;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
//                .add_plugins(RapierDebugRenderPlugin::default())
        .add_event::<Boom>()
        .add_systems(Startup, setupv3)
        .add_systems(Update, input_handler)
        .add_systems(Update, kill_debris)
        .add_systems(Update, (apply_gravity, check_collisions, kill))
        .add_systems(Update, warp_space)
        .add_systems(Update, load_shield)
        .add_systems(Update, copy_shield_value)
        .add_systems(Update, init_energy_display)
        .add_systems(Update, arrange_energy_display)
        .run();
}
#[derive(Event)]
struct Boom {
    entity: Entity
}

fn create_ship() -> Vec<Vec3>  {
    vec![
        Vec3::new(0.0, 0.7, 0.0), 
        Vec3::new(0.6, -0.5, 0.0), 
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(-0.6,-0.5,0.)
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

fn create_asteroid() -> Vec<Vec3> {
    vec![
        Vec3::new(0.0, 0.8, 0.0), 
        Vec3::new(0.3, 0.7, 0.0), 
        Vec3::new(0.3, 0.3, 0.0), 
        Vec3::new(0.8, -0.1, 0.0), 
        Vec3::new(0.7, -0.8, 0.0), 
        Vec3::new(-0.5, -0.9, 0.0), 
        Vec3::new(-0.5, -0.7, 0.0), 
        Vec3::new(-0.8, 0.2, 0.0), 
        Vec3::new(-0.6, 0.7, 0.0), 
    ]
}
fn create_asteroid_old() -> Vec<Vec3> {
    vec![
        Vec3::new(0.0, 0.3, 0.0), 
        Vec3::new(0.3, 0., 0.0), 
        Vec3::new(0.0, -0.3, 0.0),
        Vec3::new(-0.3,0.,0.)
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

const MOON_TILES:usize = 15;
fn create_moon() -> Vec<Vec3> {
   (0..MOON_TILES).map(|i|Vec2::from_angle(i as f32*PI*2.0/MOON_TILES as f32).extend(0.)).collect()
}

fn create_shield() -> Vec<Vec3> {
    let segments = 8;
    (0..segments).into_iter().map(|i|
        Vec2::from_angle((i as f32)*2.*PI/(segments as f32)).extend(0.)
    ).collect()
}

fn create_mesh(geometry: fn()->Vec<Vec3>, scale:f32) -> Mesh {
    let lines:Vec<Vec3> = geometry().iter().map(|v|
        v.mul(scale)
    ).collect();
    let len = lines.len();
    let mut indexes:Vec<u32> = (0..(len as u32)).collect();
    indexes.push(0);
    /* //![0, 1, 2, 3, 0])) */
    Mesh::new(PrimitiveTopology::LineStrip,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            lines
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR, vec![[1.0, 1.0, 1.0, 1.0]; len])
        .with_inserted_indices(
            Indices::U32(indexes))
}

#[derive(Bundle)]
struct Celestial {
        events: ActiveEvents,
        body: RigidBody,
        gravity_scale: GravityScale,
        mass_property: ReadMassProperties,
        force: ExternalForce,
        velocity: Velocity,
        restitution: Restitution,
        sleeping:Sleeping,
}

impl Default for Celestial {
    fn default() -> Self {
       Celestial {
        events: ActiveEvents::CONTACT_FORCE_EVENTS,
        body: RigidBody::Dynamic,
        gravity_scale: GravityScale(0.0),
        mass_property: ReadMassProperties::default(),
        force: ExternalForce::default(),
        velocity: Velocity{ linvel:Vec2::new(0.0,0.0), angvel:0.0},
        restitution: Restitution::coefficient(0.7),
        sleeping: Sleeping::disabled(),
       }
    }
}

#[derive(Resource)]
struct MeshHandles {
    ship: Handle<Mesh>,
    fighter: Handle<Mesh>,
    debris: Handle<Mesh>,
    shot: Handle<Mesh>,
    asteroid: Handle<Mesh>,
    shield: Handle<Mesh>,
    moon: Handle<Mesh>,

    material: Handle<ColorMaterial>,
    shot_material: Handle<ColorMaterial>,
    debris_material: Handle<ColorMaterial>,
}

fn setupv3(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh_handles=MeshHandles {
        ship : meshes.add(create_mesh(create_ship, 16.)),
        fighter : meshes.add(create_mesh(create_figter, 32.)),
        debris : meshes.add(create_mesh(create_debris, 16.)),
        shot : meshes.add(create_mesh(create_shot, 16.)),
        asteroid : meshes.add(create_mesh(create_asteroid, 8.)),
        shield : meshes.add(create_mesh(create_shield, 16.)),
        moon : meshes.add(create_mesh(create_moon, 16.)),

        material: materials.add(ColorMaterial::from(Color::BLUE)),
        shot_material: materials.add(ColorMaterial::from(Color::RED)),
        debris_material: materials.add(ColorMaterial::from(Color::GRAY)),
    };

    commands.spawn(Camera2dBundle::default());

    for (i, pos) in vec![Vec3::new(-100.,0.,0.), Vec3::new(100.,0.,0.)].iter().enumerate() {
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
        ReadMassProperties::default(),
        ExternalForce::default(),
        Velocity{ linvel:Vec2::new(0.0,0.0), angvel:0.0},
        ExternalImpulse{ impulse:Vec2::new(0., 0.), torque_impulse: 0. },
        Restitution::coefficient(0.7),
        Sleeping::disabled(),
        Ship { player:i as u8 },
        Shield { energy: 1.0 },
        Thruster{thruster_time:0.},
        Gun{time:0.},
        )).with_children(|p| {
            p.spawn((
                    MaterialMesh2dBundle {
                        mesh: mesh_handles.shield.clone().into(),
                        transform: Transform::default(),
                        material: materials.add(ColorMaterial::from(Color::RED)),
                        ..Default::default()
                    },
                    Shield{energy:1.0}
            ));
        }
        );

    }

    for _ in 0..4 {
        let pos = Vec3::new(
            rand::random::<f32>()*100.-50.,
            rand::random::<f32>()*100.-50.,
            0.);
        spawn_asteroid(&mut commands, pos, &mesh_handles, 4., Velocity{linvel:Vec2::ZERO, angvel:0.});
    }
    commands.insert_resource(mesh_handles);
}


        
fn spawn_asteroid(
    commands: &mut Commands,
    pos: Vec3,
    mesh_handles: &MeshHandles,
    size: f32,
    velocity: Velocity
) {
    let vertices:Vec<Vec2> = create_asteroid().iter().map(|v|v.xy()*8.).collect();

    commands.spawn((MaterialMesh2dBundle {
        mesh: mesh_handles.asteroid.clone().into(),
        transform: Transform::default().with_translation(pos).with_scale(Vec3::splat(size)),
        material: mesh_handles.material.clone(),
        ..Default::default()
    },
    Collider::convex_hull(vertices.as_slice()).unwrap(),
    ActiveEvents::CONTACT_FORCE_EVENTS,
    RigidBody::Dynamic,
    ReadMassProperties::default(),
    ExternalForce::default(),
    GravityScale(0.0),
    Sleeping::disabled(),
    Velocity{ linvel:velocity.linvel, angvel:0.0},
    ExternalImpulse{ impulse:Vec2::new(0., 0.), torque_impulse: 0. },
    Restitution::coefficient(0.7),
    Shield { energy: 0.1 },
    Asteroid,
    ));
}

fn init_energy_display(
    ships: Query<(Entity, &Transform), Changed<Ship>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 10.0,
        color: Color::WHITE,
    };
    for (ship, &transform) in &ships {
        commands
            .spawn((
                    Text2dBundle {
                        text: Text::from_section("asdsd", text_style.clone()),
                        transform,
                            ..default()
                    },
                    EnergyDisplay {ship},
            ));
    }
}

fn load_shield(
    mut ships: Query<&mut Shield, With<Ship>>,
    timer: Res<Time>
) {
    for mut ship in &mut ships {
        if ship.energy>=0. {
            ship.energy+=timer.delta_seconds() * HEAL_SPEED;
            if ship.energy>1. {
                ship.energy = 1.;
            }
        }
    }
}

fn arrange_energy_display(
    ships: Query<(&Transform, &Shield), With<Ship>>,
    mut displays: Query<(Entity, &mut Transform, &EnergyDisplay, &mut Text), Without<Ship>>,
    mut commands: Commands,
) {
    for (display_entity, mut transform, display, mut text) in &mut displays {
        if let Ok((ship_transform, shield)) = ships.get(display.ship) {
            if shield.energy<0. {
                commands.entity(display_entity).despawn();
            }
            transform.translation = ship_transform.translation+ Vec3::new(-20., 30., 0.);
            text.sections = vec![TextSection::from(format!("{:0} %",(100.*shield.energy) as i32))];
        }
    }
}


fn copy_shield_value(
    ship_shield: Query<&Shield, (With<Ship>, Changed<Shield>)>,
    mut shield: Query<(&mut Shield, &Parent, &mut Handle<ColorMaterial>), Without<Ship>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    for (mut shield, parent, mut material) in &mut shield {
        if let Ok(p) = ship_shield.get(parent.get()) {
            shield.energy = p.energy;
            let x = materials.add(ColorMaterial::from(Color::rgba(1., 1., 1., p.energy)));
            *material = x;
        }
    }
}

const GRAVITY_SCALE:f32 = 0.3;

fn apply_gravity(
    all_masses: Query<(Entity, &Transform, &ReadMassProperties)>,
    mut forced_masses: Query<(Entity, &Transform, &ReadMassProperties, &mut ExternalForce)>,
) {
    for (forced_entity, forced_pos, forced_mass, mut force) in &mut forced_masses {
        let mut force_sum = Vec3::ZERO;
        for (other_entity, other_pos, other_mass) in &all_masses {
            if forced_entity != other_entity {
                let direction = other_pos.translation-forced_pos.translation;
                force_sum += GRAVITY_SCALE*direction*(forced_mass.mass*other_mass.mass)/direction.length_squared();
            }
        }
        force.force= force_sum.xy();
    }
}


fn kill(mut reader: EventReader<Boom>,
    asteroids: Query<(&Transform, &Velocity), With<Asteroid>>,
    positions: Query<(&Transform, &Velocity)>,
    mut commands: Commands,
    mesh_handles: Res<MeshHandles>,
    time: Res<Time>,
) {
    for event in reader.read() {
        if let Ok((asteroid_transform, velocity)) = asteroids.get(event.entity) {
            if asteroid_transform.scale.x>1. {
                for _ in 0..4 {
                    spawn_asteroid(&mut commands, asteroid_transform.translation+ Vec3::new(rand::random::<f32>()*20.,rand::random::<f32>()*20.,0.), &mesh_handles, asteroid_transform.scale.x/2., velocity.clone());
                }
            }
        }
        if let Some(mut e) = commands.get_entity(event.entity) {
            e.despawn();
        }
        if let Ok((pos, speed)) = positions.get(event.entity) {    
            for _ in 0..4 {
                let v = Vec2::from_angle(rand::random::<f32>()*3.1415*2.);
                spawn_debris(&mut commands, &mesh_handles, pos.translation, speed.linvel+ v * DEBRIS_SPEED, &time);
            }
        }
    }
}

fn check_collisions(
    mut reader2: EventReader<ContactForceEvent>,
    mut objects: Query<&mut Shield>,
    asteroids: Query<&Asteroid>,
    mut writer: EventWriter<'_, Boom>
) {
    for event in reader2.read() {
        if asteroids.get(event.collider1).is_ok() && asteroids.get(event.collider2).is_ok() {
            trace!("Both asteroids!");
        } else {
            for entity in vec![event.collider1, event.collider2] {
                if let Ok(mut ship) = objects.get_mut(entity) {
                    (*ship).energy-=0.2;
                    if ship.energy <0. {
                        writer.send(Boom { entity });
                    }
                }
            }
        }
    }
}

fn warp_space(mut query: Query<&mut Transform>) {
    for mut transform in &mut query {
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
const DEBRIS_SPEED:f32 = 50.;
const GUN_TIME:f32 = 0.2;
const GUN_LIFETIME:f32 = 1.0;
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
    for (mut speed, mut impulse, transform, mut thruster, mut gun, ship) in &mut query {

        if let Some(keys) = get_key_config_for(ship.player) {

            gun.time-=time.delta_seconds();
            if keyboard_input.pressed(keys.shoot) {
                let r = transform.rotation.to_euler(EulerRot::XYZ);

                let rnd = rand::random::<f32>()*0.1-0.05;
                let v = Vec2::from_angle(rnd + r.2+3.1415/2.);

                if gun.time < 0. {
                    gun.time = GUN_TIME;

                    commands.spawn((MaterialMesh2dBundle {
                        mesh: mesh_handles.shot.clone().into(),
                        transform: Transform::default()
                            .with_rotation(transform.rotation)
                            .with_translation(transform.translation+v.extend(0.)),
                            material: mesh_handles.shot_material.clone(),
                            ..Default::default()
                    },
                    Debris{},
                    RigidBody::Dynamic,
                    GravityScale(0.0),
                    Collider::ball(2.0),
                    Velocity{linvel:speed.linvel+v* SHOT_SPEED, angvel:0. },
                    Lifetime{ death: time.elapsed_seconds() + GUN_LIFETIME}
                    ));
                }
            }
            if keyboard_input.pressed(keys.thrust) {
                let r = transform.rotation.to_euler(EulerRot::XYZ);

                let rnd = rand::random::<f32>()*0.3-0.15;
                let v = Vec2::from_angle(rnd + r.2+3.1415/2.);
                impulse.impulse =v*100000. * time.delta_seconds(); 

                // speed_up
                thruster.thruster_time+=time.delta_seconds();
                if thruster.thruster_time > THRUSTER_TIME {
                    thruster.thruster_time -= THRUSTER_TIME;

                    spawn_debris(&mut commands, &mesh_handles, transform.translation, speed.linvel-v* THRUSTER_SPEED, &time);
                }

            }
            if keyboard_input.pressed(keys.left) {
                // instead - maybe add a force?
                speed.angvel=5.;
            } else if keyboard_input.pressed(keys.right) {
                speed.angvel=-5.;
            } else {
                speed.angvel=0.;
            }

        }
    }
    if keyboard_input.pressed(KeyCode::KeyQ) {
        app_exit_events.send(bevy::app::AppExit);
    }
}

fn spawn_debris(commands: &mut Commands, mesh_handles: &Res<MeshHandles>, pos: Vec3, speed: Vec2, time: &Res<Time>) {
    commands.spawn((MaterialMesh2dBundle {
        mesh: mesh_handles.debris.clone().into(),
        transform: Transform::default()
            .with_translation(pos),
            material: mesh_handles.debris_material.clone(),
            ..Default::default()
    },
    Debris{},
    RigidBody::Dynamic,
    Velocity{ linvel: speed, angvel:0. },
    Lifetime{ death: time.elapsed_seconds() + THRUSTER_LIFETIME }
    ));
}

