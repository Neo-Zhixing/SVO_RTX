use bevy::prelude::*;
use std::borrow::Cow;
use bevy::render::render_graph::{CommandQueue, ResourceSlots, SystemNode};
use bevy::render::renderer::{RenderContext, BufferId, RenderResourceContext, BufferUsage, RenderResourceBinding, RenderResourceBindings, BufferInfo};
use bevy::render::camera::{ActiveCameras, Camera, PerspectiveProjection};
use bevy::core::AsBytes;
use bevy::render::render_graph::Node;


#[derive(Debug)]
pub struct CameraProjectionNode {
    command_queue: CommandQueue,
    camera_name: Cow<'static, str>,
}

impl CameraProjectionNode {
    pub fn new<T>(camera_name: T) -> Self
        where
            T: Into<Cow<'static, str>>,
    {
        CameraProjectionNode {
            command_queue: Default::default(),
            camera_name: camera_name.into(),
        }
    }
}

impl Node for CameraProjectionNode {
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

impl SystemNode for CameraProjectionNode {
    fn get_system(&self, commands: &mut Commands) -> Box<dyn System<In = (), Out = ()>> {
        let system = projection_node_system.system();
        commands.insert_local_resource(
            system.id(),
            ProjectionNodeState {
                camera_name: self.camera_name.clone(),
                command_queue: self.command_queue.clone(),
                projection_buffer: None,
                staging_buffer: None,
            },
        );
        Box::new(system)
    }
}

#[derive(Debug, Default)]
pub struct ProjectionNodeState {
    command_queue: CommandQueue,
    camera_name: Cow<'static, str>,
    projection_buffer: Option<BufferId>,
    staging_buffer: Option<BufferId>,
}

pub fn projection_node_system(
    mut state: Local<ProjectionNodeState>,
    active_cameras: Res<ActiveCameras>,
    render_resource_context: Res<Box<dyn RenderResourceContext>>,
    // PERF: this write on RenderResourceAssignments will prevent this system from running in parallel
    // with other systems that do the same
    mut render_resource_bindings: ResMut<RenderResourceBindings>,
    query: Query<(&GlobalTransform, &PerspectiveProjection)>,
) {
    let render_resource_context = &**render_resource_context;

    let (global_transform, perspective_projection) = if let Some(entity) = active_cameras.get(&state.camera_name) {
        query.get(entity).unwrap()
    } else {
        return;
    };
    let data_size = std::mem::size_of::<[f32; 20]>();

    let staging_buffer = if let Some(staging_buffer) = state.staging_buffer {
        render_resource_context.map_buffer(staging_buffer);
        staging_buffer
    } else {
        let buffer = render_resource_context.create_buffer(BufferInfo {
            size: data_size,
            buffer_usage: BufferUsage::COPY_DST | BufferUsage::UNIFORM,
            mapped_at_creation: false,
        });
        render_resource_bindings.set(
            &(state.camera_name.to_owned() + "Projection"),
            RenderResourceBinding::Buffer {
                buffer,
                range: 0..data_size as u64,
                dynamic_index: None,
            },
        );
        state.projection_buffer = Some(buffer);

        let staging_buffer = render_resource_context.create_buffer(BufferInfo {
            size: data_size,
            buffer_usage: BufferUsage::COPY_SRC | BufferUsage::MAP_WRITE,
            mapped_at_creation: true,
        });

        state.staging_buffer = Some(staging_buffer);
        staging_buffer
    };

    let projection_data: [f32; 4] = [
        perspective_projection.fov,
        perspective_projection.aspect_ratio,
        perspective_projection.near,
        perspective_projection.far,
    ];
    let transform_data: [f32; 16] = global_transform.compute_matrix().to_cols_array();

    render_resource_context.write_mapped_buffer(
        staging_buffer,
        0..data_size as u64,
        &mut |data: &mut [u8], _renderer| {
            let slice_pos = std::mem::size_of::<[f32; 16]>();
            data[0..slice_pos].copy_from_slice(transform_data.as_bytes());
            data[slice_pos..data_size].copy_from_slice(projection_data.as_bytes());
        },
    );
    render_resource_context.unmap_buffer(staging_buffer);

    let projection_buffer = state.projection_buffer.unwrap();
    state.command_queue.copy_buffer_to_buffer(
        staging_buffer,
        0,
        projection_buffer,
        0,
        data_size as u64,
    );
}
