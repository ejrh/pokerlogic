use std::ops::Range;

use bevy::asset::Assets;
use bevy::camera::visibility::RenderLayers;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::{Circle, ColorMaterial, Commands, Component, Entity, MeshMaterial2d, Query, Res, ResMut, Time, Transform, With};
use rand::RngExt;

#[derive(Component)]
pub struct Velocity(Vec3);

const NUM_PARTICLES: usize = 1000;
const SPAWN_RADIUS: f32 = 10.0;
const SPAWN_VELOCITY: f32 = 2000.0;
const SCALE_RANGE: Range<f32> = 1.0..4.0;
const GRAVITY: Vec3 = Vec3::new(0.0, -1000.0, 0.0);
const DRAG: f32 = 1.5;

pub fn spawn_fireworks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let fireworks_mesh = meshes.add(Circle::new(1.0));
    let fireworks_materials = [
        materials.add(ColorMaterial::from(Color::srgb(1.0, 1.0, 0.2))),
        materials.add(ColorMaterial::from(Color::srgb(1.0, 0.2, 0.2))),
        materials.add(ColorMaterial::from(Color::srgb(0.0, 0.0, 0.0))),
    ];

    for _ in 0..NUM_PARTICLES {
        let (x,y) = rand_in_circle(SPAWN_RADIUS);
        let (vx, vy) = rand_in_circle(SPAWN_VELOCITY);
        let scale = rand::rng().random_range(SCALE_RANGE);
        let material = &fireworks_materials[rand::rng().random_range(0..fireworks_materials.len())];

        commands.spawn((
            Mesh2d(fireworks_mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(scale)),
            Velocity(Vec3::new(vx, vy, 0.0)),
            RenderLayers::layer(1),
        ));
    }
}

pub fn animate_fireworks(
    mut fireworks: Query<(Entity, &mut Velocity, &mut Transform), With<Mesh2d>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    for (id, mut v, mut t) in fireworks.iter_mut() {
        let a = GRAVITY - DRAG * v.0;
        v.0 += a * dt;

        t.translation += v.0 * dt;

        // If we are below the visible screen and heading down, just disappear
        if t.translation.y < -400.0 && v.0.y < 0.0 {
            commands.entity(id).despawn();
        }
    }
}

fn rand_in_circle(radius: f32) -> (f32, f32) {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let distance = rng.random_range(0.0f32..1.0).sqrt() * radius;
    let x = distance * angle.cos();
    let y = distance * angle.sin();

    (x, y)
}
