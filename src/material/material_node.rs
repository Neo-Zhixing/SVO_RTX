use crate::material::texture_repo::TextureRepoHandle;
use crate::material::{Material, MaterialPalette};

use bevy::core::AsBytes;
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
            },
        );
        Box::new(system)
    }
}

#[derive(Debug, Default)]
pub struct MaterialNodeState {
    command_queue: CommandQueue,
}

pub fn material_node_system(
    mut state: Local<MaterialNodeState>,
    render_resource_context: Res<Box<dyn RenderResourceContext>>,
    mut palettes: ResMut<Assets<MaterialPalette>>,
    materials: Res<Assets<Material>>,
    mut query: Query<(&Handle<MaterialPalette>, &mut RenderPipelines)>,
) {
    let render_resource_context = &**render_resource_context;

    for (palette_handle, mut render_pipelines) in query.iter_mut() {
        let mut palette = palettes.get_mut(palette_handle).unwrap();
        if let Some(staging_buffer) = palette.staging_buffer {
            render_resource_context.remove_buffer(staging_buffer);
            palette.staging_buffer = None;
        }

        if palette.buffer.is_none() {
            let texture_repo_handle_size = std::mem::size_of::<TextureRepoHandle>();
            let material_size = texture_repo_handle_size * 1;
            let colored_material_size = texture_repo_handle_size * 1;

            let palette_section_size = std::mem::size_of::<Color>() * 256;
            let colored_material_section_size = colored_material_size * 256;
            let material_section_size = material_size * palette.materials.len();
            let total_size =
                palette_section_size + colored_material_section_size + material_section_size;
            let alignment: usize = wgpu::COPY_BUFFER_ALIGNMENT as usize;
            let total_size_aligned = ((total_size + alignment - 1) / alignment) * alignment;
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
                    data[0..palette_section_size].copy_from_slice(palette.color_palette.as_bytes());
                    let data = &mut data[palette_section_size..];

                    // Colored materials
                    for i in 0..256 {
                        if let Some(material_handle) = &palette.colored_materials[i] {
                            let material = materials.get(material_handle).unwrap();
                            unsafe {
                                let ptr = &mut *(data.as_mut_ptr() as *mut u16).offset(i as isize);
                                *ptr = material.diffuse.0;
                            }
                        }
                    }
                    let data = &mut data[colored_material_section_size..];
                    // other materials
                    for (i, handle) in palette.materials.iter().enumerate() {
                        let material = materials.get(handle.clone()).unwrap();
                        unsafe {
                            let ptr = &mut *(data.as_mut_ptr() as *mut u16).offset(i as isize);
                            *ptr = material.diffuse.0;
                        }
                    }
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

            render_pipelines.bindings.set(
                "Materials",
                RenderResourceBinding::Buffer {
                    buffer: palette.buffer.unwrap(),
                    range: 0..total_size_aligned as u64,
                    dynamic_index: None,
                },
            )
        }
    }
}
