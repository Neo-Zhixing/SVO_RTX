#![feature(array_map)]

use crate::raytracer::{OctreeRayTracerPlugin};
use bevy::render::draw::DrawContext;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::{
    BlendDescriptor, ColorStateDescriptor, ColorWrite, IndexFormat, PrimitiveTopology,
};
use bevy::render::texture::TextureFormat;
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::shape,
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
    },
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use svo::octree::Octree;
use crate::raytracer::chunk::{Chunk, Voxel, ChunkBundle};
use bevy::render::camera::PerspectiveProjection;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin};
use crate::lights::SunLight;

mod raytracer;
mod lights;

/// This example illustrates how to load shaders such that they can be
/// edited while the example is still running.
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup.system())
        .add_plugin(OctreeRayTracerPlugin::default())
        .add_plugin(PrintDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(my_system.system())
        .run();
}

fn setup(
    commands: &mut Commands,
    asset_server: ResMut<AssetServer>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut chunks: ResMut<Assets<Chunk>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    // Watch for changes

    let lod = 4;

    let mut octree2: Octree<Voxel> = Octree::new();
    let monument = dot_vox::load("assets/monu8-without-water.vox").unwrap();
    let model = &monument.models[0];


    for voxel in &model.voxels {
        octree2.set(voxel.x as u32, voxel.z as u32, voxel.y as u32, 256, Voxel(voxel.i as u16));
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


fn my_system(
    mut sun_light_resource: ResMut<SunLight>,
    time: Res<Time>
) {
    sun_light_resource.direction.x = (time.seconds_since_startup()).cos() as f32;
    sun_light_resource.direction.z = (time.seconds_since_startup()).sin() as f32;
}