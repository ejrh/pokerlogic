use std::ops::Range;

use bevy::asset::Assets;
use bevy::camera::visibility::RenderLayers;
use bevy::color::Color;
use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, In, Query, Res, ResMut, SystemInput},
};
use bevy::math::{primitives::Circle, Vec3};
use bevy::mesh::{Mesh, Mesh2d};
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use bevy::time::Time;
use bevy::transform::components::Transform;
use rand::RngExt;

#[derive(Component)]
pub struct Firework {
    fuse: f32,
    level: usize,
}

#[derive(Component)]
pub struct Velocity(Vec3);

const NUM_PARTICLES: usize = 20;
const SPAWN_RADIUS: f32 = 1.0;
const SPAWN_VELOCITY: f32 = 1000.0;
const SCALE_RANGE: Range<f32> = 1.0..4.0;
const FUSE_RANGE: Range<f32> = 0.5..1.0;
const GRAVITY: Vec3 = Vec3::new(0.0, -1000.0, 0.0);
const DRAG: f32 = 1.5;

pub fn launch_fireworks(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    let initial_position = Vec3::new(0.0, 0.0, 0.0);
    let initial_velocity = Vec3::new(0.0, 500.0, 0.0);
    let input = In::wrap((2, initial_position, initial_velocity));
    spawn_fireworks(input, commands, meshes, materials);
}

pub fn spawn_fireworks(
    params: In<(usize, Vec3, Vec3)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (level, initial_position, initial_velocity) = *params;

    let fireworks_mesh = meshes.add(Circle::new(1.0));
    let fireworks_materials = [
        materials.add(ColorMaterial::from(Color::srgb(1.0, 1.0, 1.0))),
        materials.add(ColorMaterial::from(Color::srgb(1.0, 1.0, 0.2))),
        materials.add(ColorMaterial::from(Color::srgb(1.0, 0.2, 0.2))),
        materials.add(ColorMaterial::from(Color::srgb(0.0, 0.0, 0.0))),
    ];

    for _ in 0..NUM_PARTICLES {
        let position = rand_in_circle(SPAWN_RADIUS);
        let velocity = rand_in_circle(SPAWN_VELOCITY);
        let scale = Vec3::new(rand::rng().random_range(SCALE_RANGE), rand::rng().random_range(SCALE_RANGE), 1.0);
        let fuse = rand::rng().random_range(FUSE_RANGE);

        let material = &fireworks_materials[rand::rng().random_range(0..fireworks_materials.len())];

        let position = initial_position + position;
        let velocity = initial_velocity + velocity;

        commands.spawn((
            Firework { fuse, level },
            Mesh2d(fireworks_mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_translation(position).with_scale(scale),
            Velocity(velocity),
            RenderLayers::layer(1),
        ));
    }
}

pub fn animate_fireworks(
    mut fireworks: Query<(&mut Velocity, &mut Transform), With<Firework>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut v, mut t) in fireworks.iter_mut() {
        let a = GRAVITY - DRAG * v.0;
        v.0 += a * dt;

        t.translation += v.0 * dt;
    }
}

pub fn expire_fireworks(
    mut fireworks: Query<(Entity, &mut Firework, &Velocity, &Transform), With<Mesh2d>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    for (id, mut f, v, t) in fireworks.iter_mut() {
        // If the fuse expires, spawn the next level
        f.fuse -= dt;
        if f.fuse < 0.0 {
            if f.level > 0 {
                commands.run_system_cached_with(spawn_fireworks, (f.level - 1, t.translation, v.0));
            }
            commands.entity(id).despawn();
            continue;
        }

        // If we are below the visible screen and heading down, just disappear
        if t.translation.y < -400.0 && v.0.y < 0.0 {
            commands.entity(id).despawn();
        }
    }
}

fn rand_in_circle(radius: f32) -> Vec3 {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let distance = rng.random_range(0.0f32..1.0).sqrt() * radius;
    let x = distance * angle.cos();
    let y = distance * angle.sin();

    Vec3::new(x, y, 0.0)
}
