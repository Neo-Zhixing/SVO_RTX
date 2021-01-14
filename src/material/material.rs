use std::borrow::Cow;
use crate::material::texture_repo::TextureRepoHandle;
use std::collections::HashMap;
use crate::Voxel;
use bevy::asset::{Handle, HandleUntyped};
use bevy::render::color::Color;
use bevy::reflect::TypeUuid;
use bevy::render::renderer::{BufferInfo, BufferId};

pub const DEFAULT_MATERIAL_PALETTE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(MaterialPalette::TYPE_UUID, 0x786f4ab62875ebbd);

#[derive(TypeUuid)]
#[uuid = "5329ec9b-019a-4337-8b23-730919e90b9d"]
pub struct Material {
    pub name: Cow<'static, str>,
    pub diffuse: TextureRepoHandle,
}

#[derive(TypeUuid)]
#[uuid = "fb384976-d212-44c6-b3eb-f51aa7cf782f"]
pub struct ColoredMaterial {
    pub name: Cow<'static, str>,
    pub diffuse: TextureRepoHandle,
}

#[derive(TypeUuid, Debug)]
#[uuid = "6ac654c6-607f-426f-98b5-2e7f6d810056"]
pub struct MaterialPalette {
    pub color_palette: [Color; 256],
    pub colored_materials: [Option<Handle<ColoredMaterial>>; 256],
    pub materials: Vec<Handle<Material>>,
    pub buffer: Option<BufferId>,
    pub staging_buffer: Option<BufferId>,
}

impl MaterialPalette {
    pub fn new() -> Self {
        MaterialPalette {
            color_palette: [Color::rgb_linear(0.0, 0.0, 0.0); 256],
            colored_materials: [None; 256],
            materials: vec![],
            buffer: None,
            staging_buffer: None
        }
    }
    pub fn add_material(&mut self, material: Handle<Material>) -> Voxel {
        self.materials.push(material);
        // 0 was reserved for air
        Voxel::new(self.materials.len() as u16)
    }
}
