use bevy::prelude::*;

use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::render::{
    mesh::{Indices, VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setupv3)
        .add_systems(Update, input_handler)
        .add_systems(Update, move_speeder)
        .add_systems(Update, kill_debris)
        .run();
}

#[derive(Component)]
struct Debris;

#[derive(Component)]
struct Ship;

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

fn create_mesh(lines: Vec<Vec3>) -> Mesh {
    Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            lines
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR, vec![[1.0, 1.0, 1.0, 1.0]; 4])
        .with_inserted_indices(
            Indices::U32(vec![0, 1, 2, 3, 0]))
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
        ship : meshes.add(create_mesh(create_ship())),
        fighter : meshes.add(create_mesh(create_figter())),
        debris : meshes.add(create_mesh(create_debris())),
        shot : meshes.add(create_mesh(create_shot())),
        material: materials.add(ColorMaterial::from(Color::BLUE)),
    };

    commands.spawn(Camera2dBundle::default());
    commands.spawn((MaterialMesh2dBundle {
        mesh: mesh_handles.ship.clone().into(),
        transform: Transform::default().with_scale(Vec3::splat(16.)).with_translation(Vec3::new(100., 100. , 0.)),
        material: mesh_handles.material.clone(),
        ..Default::default()
    },
    Ship,
    Thruster{thruster_time:0.},
    Gun{time:0.},
    Speed{speed:Vec2::new(0., 0.) }
    ));
    commands.spawn((MaterialMesh2dBundle {
        mesh: mesh_handles.fighter.clone().into(),
        transform: Transform::default().with_scale(Vec3::splat(16.)),
        material: mesh_handles.material.clone(),
        ..Default::default()
    },
    Ship,
    Thruster{thruster_time:0.},
    Gun{time:0.},
    Speed{speed:Vec2::new(0., 0.) }
    ));
    commands.insert_resource(mesh_handles);
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

fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Speed, &mut Transform, &mut Thruster, &mut Gun), With<Ship>>,
    time: Res<Time>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut commands: Commands,
    mesh_handles: Res<MeshHandles>
) {
    if keyboard_input.pressed(KeyCode::Space) {
        for (speed, transform, _, mut gun) in &mut query {
            let r = transform.rotation.to_euler(EulerRot::XYZ);
           
            let rnd = rand::random::<f32>()*0.3-0.15;
            let v = Vec2::from_angle(rnd + r.2+3.1415/2.);

            // speed_up
            gun.time+=time.delta_seconds();
            if gun.time > GUN_TIME {

                gun.time -= GUN_TIME;

                commands.spawn((MaterialMesh2dBundle {
                    mesh: mesh_handles.shot.clone().into(),
                    transform: Transform::default().with_scale(Vec3::splat(16.))
                        .with_rotation(transform.rotation)
                        .with_translation(transform.translation),
                        material: mesh_handles.material.clone(),
                        ..Default::default()
                },
                Debris{},
                Speed{speed:speed.speed+v* SHOT_SPEED },
                Lifetime{ death: time.elapsed_seconds() + GUN_LIFETIME}
                ));

            }

        }
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        for (mut speed, transform, mut thruster, _) in &mut query {
            let r = transform.rotation.to_euler(EulerRot::XYZ);
           
            let rnd = rand::random::<f32>()*0.3-0.15;
            let v = Vec2::from_angle(rnd + r.2+3.1415/2.);
            speed.speed+=v*100. * time.delta_seconds();

            // speed_up
            thruster.thruster_time+=time.delta_seconds();
            if thruster.thruster_time > THRUSTER_TIME {

                thruster.thruster_time -= THRUSTER_TIME;

                commands.spawn((MaterialMesh2dBundle {
                    mesh: mesh_handles.debris.clone().into(),
                    transform: Transform::default().with_scale(Vec3::splat(16.))
                        .with_translation(transform.translation),
                        material: mesh_handles.material.clone(),
                        ..Default::default()
                },
                Debris{},
                Speed{speed:speed.speed-v* THRUSTER_SPEED },
                Lifetime{ death: time.elapsed_seconds() + THRUSTER_LIFETIME }
                ));

            }

        }
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        for (_speed, mut transform, _, _) in &mut query {
            transform.rotate_z(time.delta_seconds() *5.);
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        for (_speed, mut transform, _, _) in &mut query {
            transform.rotate_z(-time.delta_seconds() *5.);
        }
    }

    if keyboard_input.pressed(KeyCode::KeyQ) {
        app_exit_events.send(bevy::app::AppExit);
    }
}

