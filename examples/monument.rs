use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::render::camera::PerspectiveProjection;

use bevy::{
    prelude::*,
    render::{pipeline::PipelineDescriptor, render_graph::RenderGraph},
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use ray_tracing::lights::SunLight;
use ray_tracing::material::texture_repo::TextureRepo;
use ray_tracing::material::{Material, MaterialPalette, DEFAULT_MATERIAL_PALETTE_HANDLE, ColoredMaterial};
use ray_tracing::raytracer::chunk::{Chunk, ChunkBundle};
use ray_tracing::OctreeRayTracerPlugin;
use ray_tracing::Voxel;
use svo::octree::Octree;
use bevy::window::WindowMode;

/// This example illustrates how to load shaders such that they can be
/// edited while the example is still running.
fn main() {
    App::build()
        // Bevy plugins
        .add_plugin(bevy::reflect::ReflectPlugin::default())
        .add_plugin(bevy::core::CorePlugin::default())
        .add_plugin(bevy::transform::TransformPlugin::default())
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin::default())
        .add_plugin(bevy::input::InputPlugin::default())
        .add_plugin(bevy::window::WindowPlugin::default())
        .add_plugin(bevy::asset::AssetPlugin::default())
        .add_plugin(bevy::render::RenderPlugin::default())
        .add_plugin(bevy::winit::WinitPlugin::default())
        .add_plugin(bevy::wgpu::WgpuPlugin::default())
        .add_plugin(DiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Custom plugins
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup.system())
        .add_resource(TextureRepo::new(512, 512))
        .add_plugin(OctreeRayTracerPlugin::default())
        .add_system(my_system.system())
        .run();
}

fn setup(
    commands: &mut Commands,
    _asset_server: ResMut<AssetServer>,
    _pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut chunks: ResMut<Assets<Chunk>>,
    _render_graph: ResMut<RenderGraph>,
    mut texture_repo: ResMut<TextureRepo>,
    mut material_palettes: ResMut<Assets<MaterialPalette>>,
) {
    let mut grass_material = ColoredMaterial {
        material: Material {
            name: "Grass".into(),
            diffuse: Some(texture_repo.load("assets/textures/grass.png")),
            scale: 1.0,
            normal: None
        },
        color_palette: [Color::BLACK; 256]
    };
    let rock_material = Material {
        name: "Stone".into(),
        diffuse: Some(texture_repo.load("assets/textures/stone.png")),
        scale: 1.0,
        normal: None
    };
    let wool_material = Material {
        name: "Wool".into(),
        diffuse: Some(texture_repo.load("assets/textures/wool.png")),
        scale: 1.0,
        normal: None
    };
    let mut colored_material = ColoredMaterial::default();

    let mut palette = material_palettes
        .get_mut(DEFAULT_MATERIAL_PALETTE_HANDLE)
        .unwrap();

    let _lod = 4;

    let mut octree2: Octree<Voxel> = Octree::new();
    let monument = dot_vox::load("assets/monu9.vox").unwrap();
    let model = &monument.models[0];

    for (i, color) in monument.palette.iter().enumerate() {
        let r = color >> 24;
        let g = (color >> 16) & 0xFF;
        let b = (color >> 8) & 0xFF;
        let a = color & 0xFF;
        let color = Color::rgba_u8(r as u8, g as u8, b as u8, a as u8);
        colored_material.color_palette[i] = color;
        grass_material.color_palette[i] = color;
    }
    println!("Added material");

    let colored_voxel = palette.add_colored_material(colored_material);
    let grass_voxel = palette.add_colored_material(grass_material);
    let rock_voxel = palette.add_material(rock_material);
    let wool_voxel = palette.add_material(wool_material);
    for voxel in &model.voxels {
        octree2.set(
            voxel.x as u32,
            voxel.z as u32,
            voxel.y as u32,
            256,
            colored_voxel.with_color(voxel.i)
        );
    }

    let chunk2 = Chunk::new(octree2, Vec4::new(0.0, 0.0, 0.0, 16.0));

    let chunk_handle2 = chunks.add(chunk2);
    commands
        .spawn(ChunkBundle::new(chunk_handle2))
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            perspective_projection: PerspectiveProjection {
                near: 0.1,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(FlyCamera::default());
}

fn my_system(mut sun_light_resource: ResMut<SunLight>, time: Res<Time>) {
    sun_light_resource.direction.x = (time.seconds_since_startup()).cos() as f32;
    sun_light_resource.direction.z = (time.seconds_since_startup()).sin() as f32;
}
