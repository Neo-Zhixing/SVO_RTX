use crate::material::MaterialPalette;

use bevy::prelude::*;

use bevy::render::render_graph::Node;
use bevy::render::render_graph::{CommandQueue, ResourceSlots, SystemNode};
use bevy::render::renderer::{
    BufferInfo, BufferUsage, RenderContext, RenderResourceBinding, RenderResourceContext,
};

#[derive(Debug)]
pub struct MaterialNode {
    command_queue: CommandQueue,
}

impl MaterialNode {
    pub fn new() -> Self {
        MaterialNode {
            command_queue: Default::default(),
        }
    }
}

impl Node for MaterialNode {
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

impl SystemNode for MaterialNode {
    fn get_system(&self, commands: &mut Commands) -> Box<dyn System<In = (), Out = ()>> {
        let system = material_node_system.system();
        commands.insert_local_resource(
            system.id(),
            MaterialNodeState {
                command_queue: self.command_queue.clone(),
                event_reader: Default::default()
            },
        );
        Box::new(system)
    }
}

#[derive(Default)]
pub struct MaterialNodeState {
    command_queue: CommandQueue,
    pub event_reader: EventReader<AssetEvent<MaterialPalette>>,
}

pub fn material_node_system(
    mut state: Local<MaterialNodeState>,
    render_resource_context: Res<Box<dyn RenderResourceContext>>,
    mut palettes: ResMut<Assets<MaterialPalette>>,
    texture_events: Res<Events<AssetEvent<MaterialPalette>>>,
    mut query: Query<(&Handle<MaterialPalette>, &mut RenderPipelines)>,
) {

    let render_resource_context = &**render_resource_context;
    for event in state.event_reader.iter(&texture_events) {
        match event {
            AssetEvent::Created { handle } => {
                println!("created material palette");
                let mut palette = palettes.get_mut(handle).unwrap();
                if let Some(staging_buffer) = palette.staging_buffer {
                    println!("removed staging buffer");
                    render_resource_context.remove_buffer(staging_buffer);
                    palette.staging_buffer = None;
                }

                let colored_materials_size = palette.colored_materials_size();
                let colored_materials_size_aligned =
                    align_to(colored_materials_size, wgpu::BIND_BUFFER_ALIGNMENT as usize);
                let total_size = colored_materials_size_aligned + palette.materials_size();
                let total_size_aligned = align_to(total_size, wgpu::COPY_BUFFER_ALIGNMENT as usize);

                if palette.buffer.is_none() {
                    let staging_buffer = render_resource_context.create_buffer(BufferInfo {
                        size: total_size_aligned,
                        buffer_usage: BufferUsage::COPY_SRC | BufferUsage::MAP_WRITE,
                        mapped_at_creation: true,
                    });
                    render_resource_context.write_mapped_buffer(
                        staging_buffer,
                        0..(total_size as u64),
                        &mut |data: &mut [u8], _renderer| {
                            // Color palette
                            palette.colored_materials_write_bytes(&mut data[0..colored_materials_size]);
                            palette.materials_write_bytes(
                                &mut data[colored_materials_size_aligned..total_size],
                            );
                        },
                    );
                    render_resource_context.unmap_buffer(staging_buffer);

                    let buffer = render_resource_context.create_buffer(BufferInfo {
                        size: total_size_aligned,
                        buffer_usage: BufferUsage::COPY_DST | BufferUsage::STORAGE,
                        mapped_at_creation: false,
                    });
                    state.command_queue.copy_buffer_to_buffer(
                        staging_buffer,
                        0,
                        buffer,
                        0,
                        total_size_aligned as u64,
                    );
                    palette.staging_buffer = Some(staging_buffer);
                    palette.buffer = Some(buffer);
                }
            }
            AssetEvent::Modified { handle } => {
                println!("palette modified");
            }
            AssetEvent::Removed { handle } => {
                println!("palette removed")
            }
        }
    }

    fn align_to(num: usize, alignment: usize) -> usize {
        ((num + alignment - 1) / alignment) * alignment
    }

    for (palette_handle, mut render_pipelines) in query.iter_mut() {
        let palette = palettes.get(palette_handle).unwrap();
        let colored_materials_size = palette.colored_materials_size();
        let colored_materials_size_aligned =
            align_to(colored_materials_size, wgpu::BIND_BUFFER_ALIGNMENT as usize);
        let total_size = colored_materials_size_aligned + palette.materials_size();
        let total_size_aligned = align_to(total_size, wgpu::COPY_BUFFER_ALIGNMENT as usize);
        render_pipelines.bindings.set(
            "ColoredMaterials",
            RenderResourceBinding::Buffer {
                buffer: palette.buffer.unwrap(),
                range: 0..colored_materials_size_aligned as u64,
                dynamic_index: None,
            },
        );
        render_pipelines.bindings.set(
            "Materials",
            RenderResourceBinding::Buffer {
                buffer: palette.buffer.unwrap(),
                range: colored_materials_size_aligned as u64..total_size_aligned as u64,
                dynamic_index: None,
            },
        );
    }
}
