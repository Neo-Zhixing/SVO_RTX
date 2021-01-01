use crate::raytracer::{OctreeRayTracerPlugin, OctreeRaytracerBundle};
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

mod raytracer;

/// This example illustrates how to load shaders such that they can be
/// edited while the example is still running.
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_plugin(OctreeRayTracerPlugin::default())
        .run();
}

fn setup(
    commands: &mut Commands,
    asset_server: ResMut<AssetServer>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    // Watch for changes

    commands
        .spawn(OctreeRaytracerBundle::default())
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(-2.0, 2.5, 5.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
}
