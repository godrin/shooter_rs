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
        .run();
}

#[derive(Component)]
struct Ship;

#[derive(Component)]
struct Speed {
    speed: Vec2
}

fn create_ship() -> Vec<Vec3>  {
    vec![
        Vec3::new(0.0, 1.0, 0.0), 
        Vec3::new(0.6, -0.3, 0.0), 
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(-0.6,-0.3,0.)
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

fn setupv3(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((MaterialMesh2dBundle {
        mesh: meshes.add(create_mesh(create_debris())).into(),
        transform: Transform::default().with_scale(Vec3::splat(16.)),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        ..Default::default()
    },
    Ship{},
    Speed{speed:Vec2::new(0., 0.) }
    ));
}

// FIXME: add time
fn move_speeder(
    mut query: Query<(&Speed, &mut Transform), With<Ship>>,
    time: Res<Time>,
) {
  for (speed, mut transform) in &mut query {
      transform.translation+=speed.speed.extend(0.) * time.delta_seconds();
  }
}

fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Speed, &mut Transform), With<Ship>>,
    time: Res<Time>,
     mut app_exit_events: ResMut<Events<bevy::app::AppExit>>
) {
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        for (mut speed, transform) in &mut query {
            let r = transform.rotation.to_euler(EulerRot::XYZ);
            let v = Vec2::from_angle(r.2+3.1415/2.);
            speed.speed+=v*100. * time.delta_seconds();
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        for (_speed, mut transform) in &mut query {
            transform.rotate_z(time.delta_seconds() *5.);
        }
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        for (_speed, mut transform) in &mut query {
            transform.rotate_z(-time.delta_seconds() *5.);
        }
    }

    if keyboard_input.pressed(KeyCode::KeyQ) {
        app_exit_events.send(bevy::app::AppExit);
    }
}

