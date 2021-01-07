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

mod raytracer;

/// This example illustrates how to load shaders such that they can be
/// edited while the example is still running.
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup.system())
        .add_plugin(OctreeRayTracerPlugin::default())
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
    let octree: Octree<Voxel> = Octree::from_signed_distance_field(|l: glam::Vec3| {
        0.4 - l.distance(Vec3::new(0.5, 0.5, 0.5))
    }, Voxel(1), lod);
    let chunk = Chunk { octree };

    let chunk_handle = chunks.add(chunk);

    commands
        .spawn(ChunkBundle::new(chunk_handle))
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            perspective_projection: PerspectiveProjection {
                near: 0.1,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(FlyCamera::default());;
}
