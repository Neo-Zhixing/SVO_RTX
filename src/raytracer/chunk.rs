use crate::material::MaterialPalette;
use crate::material::DEFAULT_MATERIAL_PALETTE_HANDLE;
use crate::raytracer::{RayPass, RAY_PIPELINE_CUBE_HANDLE, RAY_PIPELINE_HANDLE};
use crate::Voxel;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::pipeline::{
    IndexFormat, InputStepMode, PipelineSpecialization, PrimitiveTopology, RenderPipeline,
    VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat,
};
use bevy::render::renderer::BufferId;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;
use svo::octree::Octree;

#[derive(TypeUuid)]
#[uuid = "a036bb0e-f7c5-4d94-a2a8-5d7f61aace31"]
pub struct Chunk {
    pub bounding_box: Vec4,
    pub octree: Octree<Voxel>,
}

impl Chunk {
    pub fn new(octree: Octree<Voxel>, bounding_box: Vec4) -> Self {
        Chunk {
            bounding_box,
            octree,
        }
    }
}

pub struct ChunkState {
    pub(crate) octree_buffer: Option<BufferId>,
    pub(crate) staging_buffer: Option<BufferId>,
}

#[derive(Bundle)]
pub struct ChunkBundle {
    pub draw: Draw,
    pub visible: Visible,
    pub ray_pass: RayPass,
    pub chunk: Handle<Chunk>,
    pub render_pipelines: RenderPipelines,
    pub mesh: Handle<Mesh>,
    pub state: ChunkState,
    pub palette: Handle<MaterialPalette>,
}

impl ChunkBundle {
    pub fn new(chunk: Handle<Chunk>) -> Self {
        ChunkBundle {
            chunk,
            draw: Default::default(),
            visible: Visible {
                is_visible: true,
                is_transparent: false,
            },
            ray_pass: RayPass,
            render_pipelines: RenderPipelines::from_handles(&[RAY_PIPELINE_HANDLE.typed()]),
            mesh: RAY_PIPELINE_CUBE_HANDLE.typed(),
            state: ChunkState {
                octree_buffer: None,
                staging_buffer: None,
            },
            palette: DEFAULT_MATERIAL_PALETTE_HANDLE.typed(),
        }
    }
}
