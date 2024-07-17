 use bevy::prelude::*;

#[derive(Component)]
pub struct Debris;

#[derive(Component)]
pub struct EnergyDisplay {
    pub ship: Entity
}

#[derive(Component)]
pub struct Asteroid;

#[derive(Component)]
pub struct Ship {
    pub player: u8
}

#[derive(Component)]
pub struct Moon {
}

#[derive(Component)]
pub struct Shield {
    pub energy:f32
}

#[derive(Component)]
pub struct Thruster {
    pub thruster_time: f32
}

#[derive(Component)]
pub struct Gun {
    pub time: f32
}

#[derive(Component)]
pub struct Lifetime {
    pub death: f32
}

