use bevy::prelude::*;
use std::borrow::Cow;
use bevy::render::render_graph::{CommandQueue, ResourceSlots, SystemNode};
use bevy::render::renderer::{RenderContext, BufferId, RenderResourceContext, BufferUsage, RenderResourceBinding, RenderResourceBindings, BufferInfo};
use bevy::render::camera::{ActiveCameras, Camera, PerspectiveProjection};
use bevy::core::AsBytes;
use bevy::render::render_graph::Node;
use svo::octree::Octree;
use crate::Voxel;
use crate::raytracer::chunk::Chunk;


#[derive(Debug)]
pub struct OctreeNode {
    command_queue: CommandQueue,
}

impl OctreeNode {
    pub fn new() -> Self {
        OctreeNode {
            command_queue: Default::default(),
        }
    }
}

impl Node for OctreeNode {
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

impl SystemNode for OctreeNode {
    fn get_system(&self, commands: &mut Commands) -> Box<dyn System<In = (), Out = ()>> {
        let system = octree_node_system.system();
        commands.insert_local_resource(
            system.id(),
            OctreeNodeState {
                command_queue: self.command_queue.clone(),
                octree_buffer: None,
                staging_buffer: None,
            },
        );
        Box::new(system)
    }
}

#[derive(Debug, Default)]
pub struct OctreeNodeState {
    command_queue: CommandQueue,
    octree_buffer: Option<BufferId>,
    staging_buffer: Option<BufferId>,
}

pub fn octree_node_system(
    mut state: Local<OctreeNodeState>,
    active_cameras: Res<ActiveCameras>,
    render_resource_context: Res<Box<dyn RenderResourceContext>>,
    // PERF: this write on RenderResourceAssignments will prevent this system from running in parallel
    // with other systems that do the same
    chunks: Res<Assets<Chunk>>,
    mut render_resource_bindings: ResMut<RenderResourceBindings>,
    query: Query<(&Handle<Chunk>, )>,
) {
    let render_resource_context = &**render_resource_context;

    for (chunk_handle, ) in query.iter() {
        if let Some(staging_buffer) = state.staging_buffer {
        } else {
            println!("Buffer created");
            let chunk = chunks.get(chunk_handle).unwrap();
            let octree = &chunk.octree;
            // Temp code for writing buffer
            let octree_buffer = render_resource_context.create_buffer(BufferInfo {
                size: octree.total_data_size(),
                buffer_usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
                mapped_at_creation: false
            });

            let staging_buffer = render_resource_context.create_buffer(BufferInfo {
                size: octree.total_data_size(),
                buffer_usage: BufferUsage::MAP_WRITE | BufferUsage::COPY_SRC,
                mapped_at_creation: true
            });
            render_resource_bindings.set(
                "Chunk",
                RenderResourceBinding::Buffer {
                    buffer: octree_buffer,
                    range: 0..octree.total_data_size() as u64,
                    dynamic_index: None
                }
            );

            state.octree_buffer = Some(octree_buffer);
            state.staging_buffer = Some(staging_buffer);


            render_resource_context.write_mapped_buffer(
                staging_buffer,
                0..octree.total_data_size() as u64,
                &mut |data: &mut [u8], _renderer| {
                    octree.copy_into_slice(data)
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
