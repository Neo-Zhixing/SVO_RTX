use crate::material::texture_repo::TextureRepo;

use bevy::prelude::*;

use bevy::render::render_graph::Node;
use bevy::render::render_graph::{CommandQueue, ResourceSlots};
use bevy::render::renderer::{
    BufferInfo, BufferUsage, RenderContext, RenderResourceBinding, RenderResourceBindings,
    SamplerId, TextureId,
};
use bevy::render::texture::{
    AddressMode, Extent3d, FilterMode, SamplerDescriptor, TextureDescriptor, TextureDimension,
    TextureUsage,
};

#[derive(Debug)]
pub struct TextureRepoNode {
    command_queue: CommandQueue,
    texture: Option<TextureId>,
    sampler: Option<SamplerId>,
    size: Extent3d,
}

impl TextureRepoNode {
    pub fn new() -> Self {
        TextureRepoNode {
            command_queue: Default::default(),
            texture: None,
            sampler: None,
            size: Extent3d {
                width: 0,
                height: 0,
                depth: 0,
            },
        }
    }
}

impl Node for TextureRepoNode {
    fn update(
        &mut self,
        _world: &World,
        resources: &Resources,
        render_context: &mut dyn RenderContext,
        _input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        let repo = resources.get_mut::<TextureRepo>();
        if repo.is_none() {
            return;
        }
        let mut repo = repo.unwrap();

        if self.sampler.is_none() {
            let sampler = render_context
                .resources()
                .create_sampler(&SamplerDescriptor {
                    address_mode_u: AddressMode::Repeat,
                    address_mode_v: AddressMode::Repeat,
                    address_mode_w: AddressMode::Repeat,
                    mag_filter: FilterMode::Nearest,
                    min_filter: FilterMode::Linear,
                    mipmap_filter: FilterMode::Nearest,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: std::f32::MAX,
                    compare_function: None,
                    anisotropy_clamp: None,
                    border_color: None,
                });
            let mut render_resource_bindings =
                resources.get_mut::<RenderResourceBindings>().unwrap();
            render_resource_bindings.set(
                "TextureRepoSampler",
                RenderResourceBinding::Sampler(sampler),
            );
            self.sampler = Some(sampler);
        }
        if self.texture.is_none() && repo.len() == 0 {
            // Initial state. Create placeholder texture.
            let new_texture = render_context
                .resources()
                .create_texture(TextureDescriptor {
                    size: Extent3d {
                        width: 1,
                        height: 1,
                        depth: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: Default::default(),
                    usage: TextureUsage::SAMPLED,
                });

            self.texture = Some(new_texture);

            let mut render_resource_bindings =
                resources.get_mut::<RenderResourceBindings>().unwrap();
            render_resource_bindings
                .set("TextureRepo", RenderResourceBinding::Texture(new_texture));
        }
        if self.size.depth < repo.len() as u32 {
            // Texture size increased, needs to create a larger texture now.
            let new_size = repo.get_extent();
            let new_texture = render_context
                .resources()
                .create_texture(TextureDescriptor {
                    size: new_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: Default::default(),
                    usage: TextureUsage::COPY_DST | TextureUsage::SAMPLED,
                });
            if let Some(old_texture) = self.texture {
                if self.size.depth > 0 {
                    render_context.copy_texture_to_texture(
                        old_texture,
                        [0, 0, 0],
                        0,
                        new_texture,
                        [0, 0, 0],
                        0,
                        self.size,
                    );
                }
                render_context.resources().remove_texture(old_texture);
            }

            self.texture = Some(new_texture);
            self.size = new_size;

            let mut render_resource_bindings =
                resources.get_mut::<RenderResourceBindings>().unwrap();
            render_resource_bindings
                .set("TextureRepo", RenderResourceBinding::Texture(new_texture));
        }
        // Copy new textures
        let image_size: usize =
            self.size.width as usize * self.size.height as usize * std::mem::size_of::<u32>();
        for (handle, image) in repo.drain() {
            let image = image.into_bgra8();
            let staging_buffer = render_context.resources().create_buffer(BufferInfo {
                size: image_size,
                buffer_usage: BufferUsage::MAP_WRITE | BufferUsage::COPY_SRC,
                mapped_at_creation: true,
            });
            render_context.resources().write_mapped_buffer(
                staging_buffer,
                0..(image_size as u64),
                &mut |data: &mut [u8], _renderer| {
                    data.copy_from_slice(image.as_raw());
                },
            );
            render_context.copy_buffer_to_texture(
                staging_buffer,
                0,
                std::mem::size_of::<u32>() as u32 * self.size.width,
                self.texture.unwrap(),
                [0, 0, handle.get() as u32 - 1],
                0,
                Extent3d {
                    width: self.size.width,
                    height: self.size.height,
                    depth: 1,
                },
            );
            render_context.resources().unmap_buffer(staging_buffer);
            render_context.resources().remove_buffer(staging_buffer);
        }
    }
}
