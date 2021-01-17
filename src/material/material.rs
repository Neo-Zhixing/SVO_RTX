use crate::material::texture_repo::TextureRepoHandle;
use crate::Voxel;
use bevy::asset::{Handle, HandleUntyped};
use bevy::reflect::TypeUuid;
use bevy::render::color::Color;
use bevy::render::renderer::BufferId;
use std::borrow::Cow;
use bevy::core::{Bytes, AsBytes};
use std::fmt::{Debug, Formatter, Result};

pub const DEFAULT_MATERIAL_PALETTE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(MaterialPalette::TYPE_UUID, 0x786f4ab62875ebbd);

#[derive(TypeUuid)]
#[uuid = "5329ec9b-019a-4337-8b23-730919e90b9d"]
pub struct Material {
    pub name: Cow<'static, str>,
    pub scale: f32,
    pub diffuse: Option<TextureRepoHandle>,
    pub normal: Option<TextureRepoHandle>,
}

#[derive(TypeUuid)]
#[uuid = "5329ec9b-019a-4337-8b23-730919e90b9d"]
pub struct ColoredMaterial {
    pub material: Material,
    pub color_palette: [Color; 256],
}

const MATERIAL_DATA_SIZE: usize = 16;
const COLORED_MATERIAL_DATA_SIZE: usize = MATERIAL_DATA_SIZE + std::mem::size_of::<[Color; 256]>();
#[derive(TypeUuid, Debug)]
#[uuid = "6ac654c6-607f-426f-98b5-2e7f6d810056"]
pub struct MaterialPalette {
    pub colored_materials: Vec<ColoredMaterial>,
    pub materials: Vec<Material>,
    pub buffer: Option<BufferId>,
    pub staging_buffer: Option<BufferId>,
}

impl Bytes for Material {
    fn write_bytes(&self, buffer: &mut [u8]) {
        self.scale.write_bytes(&mut buffer[0..4]);
        self.diffuse.write_bytes(&mut buffer[4..6]);
        self.normal.write_bytes(&mut buffer[6..8]);
    }
    fn byte_len(&self) -> usize {
        MATERIAL_DATA_SIZE
    }
}

impl Default for ColoredMaterial {
    // Default colored material is just plain color
    fn default() -> Self {
        ColoredMaterial {
            material: Material {
                name: "PlainColor".into(),
                scale: 0.0,
                diffuse: None,
                normal: None
            },
            color_palette: [Color::BLACK; 256]
        }
    }
}

impl Debug for Material {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!("Material {}", self.name))
    }
}

impl Debug for ColoredMaterial {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!("ColoredMaterial {}", self.material.name))
    }
}

impl Bytes for ColoredMaterial {
    fn write_bytes(&self, buffer: &mut [u8]) {
        self.material.write_bytes(&mut buffer[0..MATERIAL_DATA_SIZE]);
        buffer[MATERIAL_DATA_SIZE..].copy_from_slice(self.color_palette.as_bytes());
    }
    fn byte_len(&self) -> usize {
        COLORED_MATERIAL_DATA_SIZE
    }
}

impl MaterialPalette {
    pub fn new() -> Self {
        MaterialPalette {
            colored_materials: Vec::new(),
            materials: Vec::new(),
            buffer: None,
            staging_buffer: None,
        }
    }
    pub fn add_material(&mut self, material: Material) -> Voxel {
        self.materials.push(material);
        // 0 was reserved for air
        Voxel::new(self.materials.len() as u16)
    }
    pub fn add_colored_material(&mut self, material: ColoredMaterial) -> Voxel {
        let voxel =
            Voxel::new_colored(self.colored_materials.len() as u8, 0);
        self.colored_materials.push(material);
        voxel
    }
    pub fn materials_size(&self) -> usize {
        MATERIAL_DATA_SIZE * self.materials.len()
    }
    pub fn colored_materials_size(&self) -> usize {
        COLORED_MATERIAL_DATA_SIZE * self.colored_materials.len()
    }

    pub fn materials_write_bytes(&self, mut buffer: &mut [u8]) {
        for material in self.materials.iter() {
            let slice = &mut buffer[0..MATERIAL_DATA_SIZE];
            material.write_bytes(slice);
            buffer = &mut buffer[MATERIAL_DATA_SIZE..];
        }
    }

    pub fn colored_materials_write_bytes(&self, mut buffer: &mut [u8]) {
        for material in self.colored_materials.iter() {
            let slice = &mut buffer[0..COLORED_MATERIAL_DATA_SIZE];
            material.write_bytes(slice);
            buffer = &mut buffer[COLORED_MATERIAL_DATA_SIZE..];
        }
    }
}
