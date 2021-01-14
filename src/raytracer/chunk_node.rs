use crate::raytracer::chunk::{Chunk, ChunkState};
use crate::Voxel;
use bevy::core::AsBytes;
use bevy::prelude::*;
use bevy::render::camera::{ActiveCameras, Camera, PerspectiveProjection};
use bevy::render::render_graph::Node;
use bevy::render::render_graph::{CommandQueue, ResourceSlots, SystemNode};
use bevy::render::renderer::{
    BufferId, BufferInfo, BufferUsage, RenderContext, RenderResourceBinding,
    RenderResourceBindings, RenderResourceContext,
};
use std::borrow::Cow;
use svo::octree::Octree;

#[derive(Debug)]
pub struct ChunkNode {
    command_queue: CommandQueue,
}

impl ChunkNode {
    pub fn new() -> Self {
        ChunkNode {
            command_queue: Default::default(),
        }
    }
}

impl Node for ChunkNode {
    fn update(
        &mut self,
        _world: &World,
        _resources: &Resources,
        render_context: &mut dyn RenderContext,
        _input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        self.command_queue.execute(render_context);
    }
}

impl SystemNode for ChunkNode {
    fn get_system(&self, commands: &mut Commands) -> Box<dyn System<In = (), Out = ()>> {
        let system = chunk_node_system.system();
        commands.insert_local_resource(
            system.id(),
            ChunkNodeState {
                command_queue: self.command_queue.clone(),
            },
        );
        Box::new(system)
    }
}

#[derive(Debug, Default)]
pub struct ChunkNodeState {
    command_queue: CommandQueue,
}

pub fn chunk_node_system(
    mut state: Local<ChunkNodeState>,
    render_resource_context: Res<Box<dyn RenderResourceContext>>,
    chunks: Res<Assets<Chunk>>,
    mut render_resource_bindings: ResMut<RenderResourceBindings>,
    mut query: Query<(&Handle<Chunk>, &mut ChunkState, &mut RenderPipelines)>,
) {
    let render_resource_context = &**render_resource_context;

    for (chunk_handle, mut chunk_state, mut render_pipelines) in query.iter_mut() {
        if let Some(staging_buffer) = chunk_state.staging_buffer {
        } else {
            let chunk = chunks.get(chunk_handle).unwrap();
            let octree = &chunk.octree;
            let bbox_size = std::mem::size_of::<Vec4>();
            let data_size = octree.total_data_size() + bbox_size;
            // Temp code for writing buffer
            let octree_buffer = render_resource_context.create_buffer(BufferInfo {
                size: data_size,
                buffer_usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
                mapped_at_creation: false,
            });

            let staging_buffer = render_resource_context.create_buffer(BufferInfo {
                size: data_size,
                buffer_usage: BufferUsage::MAP_WRITE | BufferUsage::COPY_SRC,
                mapped_at_creation: true,
            });
            render_pipelines.bindings.set(
                "Chunk",
                RenderResourceBinding::Buffer {
                    buffer: octree_buffer,
                    range: 0..data_size as u64,
                    dynamic_index: None,
                },
            );

            chunk_state.octree_buffer = Some(octree_buffer);
            chunk_state.staging_buffer = Some(staging_buffer);

            render_resource_context.write_mapped_buffer(
                staging_buffer,
                0..data_size as u64,
                &mut |data: &mut [u8], _renderer| {
                    data[0..bbox_size].copy_from_slice(chunk.bounding_box.as_bytes());
                    octree.copy_into_slice(&mut data[bbox_size..data_size])
                },
            );
            render_resource_context.unmap_buffer(staging_buffer);

            state.command_queue.copy_buffer_to_buffer(
                staging_buffer,
                0,
                octree_buffer,
                0,
                octree.total_data_size() as u64,
            );
        }
    }
}
