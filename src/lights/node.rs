use crate::lights::{AmbientLight, PointLight, SunLight};
use bevy::core::{AsBytes, Byteable};
use bevy::prelude::*;
use bevy::render::render_graph::{CommandQueue, Node, ResourceSlots, SystemNode};
use bevy::render::renderer::{
    BufferId, BufferInfo, BufferUsage, RenderContext, RenderResourceBinding,
    RenderResourceBindings, RenderResourceContext,
};

const LIGHTS: &str = "Lights";
/// A Render Graph [Node] that write light data from the ECS to GPU buffers
#[derive(Debug, Default)]
pub struct LightsNode {
    command_queue: CommandQueue,
    max_lights: usize,
}

impl LightsNode {
    pub fn new(max_lights: usize) -> Self {
        LightsNode {
            max_lights,
            command_queue: CommandQueue::default(),
        }
    }
}

impl Node for LightsNode {
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

impl SystemNode for LightsNode {
    fn get_system(&self, commands: &mut Commands) -> Box<dyn System<In = (), Out = ()>> {
        let system = lights_node_system.system();
        commands.insert_local_resource(
            system.id(),
            LightsNodeSystemState {
                command_queue: self.command_queue.clone(),
                max_lights: self.max_lights,
                light_buffer: None,
                staging_buffer: None,
            },
        );
        Box::new(system)
    }
}

/// Local "lights node system" state
#[derive(Debug, Default)]
pub struct LightsNodeSystemState {
    light_buffer: Option<BufferId>,
    staging_buffer: Option<BufferId>,
    command_queue: CommandQueue,
    max_lights: usize,
}

pub fn lights_node_system(
    mut state: Local<LightsNodeSystemState>,
    render_resource_context: Res<Box<dyn RenderResourceContext>>,
    ambient_light_resource: Res<AmbientLight>,
    sun_light_resource: Res<SunLight>,
    // TODO: this write on RenderResourceBindings will prevent this system from running in parallel with other systems that do the same
    mut render_resource_bindings: ResMut<RenderResourceBindings>,
    query: Query<(&PointLight, &GlobalTransform)>,
) {
    let state = &mut state;
    let render_resource_context = &**render_resource_context;

    let point_light_count = query.iter().count() as u32;
    let current_light_uniform_size = std::mem::size_of::<[f32; 11]>()
        + std::mem::size_of::<u32>()
        + std::mem::size_of::<PointLight>() * point_light_count as usize;
    let max_light_uniform_size = std::mem::size_of::<[f32; 11]>()
        + std::mem::size_of::<u32>()
        + std::mem::size_of::<PointLight>() * state.max_lights;

    if let Some(staging_buffer) = state.staging_buffer {
        render_resource_context.map_buffer(staging_buffer);
    } else {
        let buffer = render_resource_context.create_buffer(BufferInfo {
            size: max_light_uniform_size,
            buffer_usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            ..Default::default()
        });
        render_resource_bindings.set(
            LIGHTS,
            RenderResourceBinding::Buffer {
                buffer,
                range: 0..max_light_uniform_size as u64,
                dynamic_index: None,
            },
        );
        state.light_buffer = Some(buffer);

        let staging_buffer = render_resource_context.create_buffer(BufferInfo {
            size: max_light_uniform_size,
            buffer_usage: BufferUsage::COPY_SRC | BufferUsage::MAP_WRITE,
            mapped_at_creation: true,
        });
        state.staging_buffer = Some(staging_buffer);
    }

    let staging_buffer = state.staging_buffer.unwrap();
    render_resource_context.write_mapped_buffer(
        staging_buffer,
        0..current_light_uniform_size as u64,
        &mut |data, _renderer| {
            let mut current_size_head: usize = 0;
            let mut current_size_tail = std::mem::size_of::<[f32; 4]>();
            // ambient light
            let ambient_light_data: [f32; 4] = [
                ambient_light_resource.color.r_linear(),
                ambient_light_resource.color.g_linear(),
                ambient_light_resource.color.b_linear(),
                ambient_light_resource.color.a(),
            ];
            data[current_size_head..current_size_tail]
                .copy_from_slice(ambient_light_data.as_bytes());

            // sun_light
            let sun_light_color_data: [f32; 4] = [
                sun_light_resource.color.r_linear(),
                sun_light_resource.color.g_linear(),
                sun_light_resource.color.b_linear(),
                sun_light_resource.color.a(),
            ];
            let size = std::mem::size_of::<[f32; 4]>();
            current_size_head = current_size_tail;
            current_size_tail += size;
            data[current_size_head..current_size_tail]
                .copy_from_slice(sun_light_color_data.as_bytes());

            let sun_light_dir_data: [f32; 3] = sun_light_resource.direction.into();
            let size = std::mem::size_of::<[f32; 3]>();
            current_size_head = current_size_tail;
            current_size_tail += size;
            data[current_size_head..current_size_tail]
                .copy_from_slice(sun_light_dir_data.as_bytes());

            let size = std::mem::size_of::<u32>();
            current_size_head = current_size_tail;
            current_size_tail += size;
            data[current_size_head..current_size_tail]
                .copy_from_slice(point_light_count.as_bytes());

            // light array
            for ((light, global_transform), slot) in query.iter().zip(
                data[current_size_tail..current_light_uniform_size]
                    .chunks_exact_mut(std::mem::size_of::<[f32; 8]>()),
            ) {
                let color: [f32; 4] = [
                    light.color.r_linear(),
                    light.color.g_linear(),
                    light.color.b_linear(),
                    light.color.a(),
                ];
                let color_size = std::mem::size_of::<[f32; 4]>();
                slot[0..color_size].copy_from_slice(color.as_bytes());

                let pos: [f32; 3] = global_transform.translation.into();
                let pos_size = std::mem::size_of::<[f32; 3]>();
                slot[color_size..(color_size + pos_size)].copy_from_slice(pos.as_bytes());
            }
        },
    );
    render_resource_context.unmap_buffer(staging_buffer);
    let light_buffer = state.light_buffer.unwrap();
    state.command_queue.copy_buffer_to_buffer(
        staging_buffer,
        0,
        light_buffer,
        0,
        current_light_uniform_size as u64,
    );
}
