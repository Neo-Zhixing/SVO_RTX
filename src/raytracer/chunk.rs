use svo::octree::Octree;
use bevy::reflect::TypeUuid;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;
use crate::raytracer::{RayPass, RAY_PIPELINE_HANDLE, RAY_PIPELINE_CUBE_HANDLE};
use bevy::prelude::*;
use bevy::render::pipeline::{RenderPipeline, PipelineSpecialization, IndexFormat, PrimitiveTopology, VertexBufferDescriptor, InputStepMode, VertexAttributeDescriptor, VertexFormat};

#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct Voxel(pub u16);

impl svo::Voxel for Voxel {
    fn avg(voxels: [Self; 8]) -> Self {
        unimplemented!()
    }
}

impl fmt::Debug for Voxel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(TypeUuid)]
#[uuid = "a036bb0e-f7c5-4d94-a2a8-5d7f61aace31"]
pub struct Chunk {
    pub octree: Octree<Voxel>
}

#[derive(Bundle)]
pub struct ChunkBundle {
    pub draw: Draw,
    pub visible: Visible,
    pub ray_pass: RayPass,
    pub chunk: Handle<Chunk>,
    pub bounding_box: Vec4,
    pub render_pipelines: RenderPipelines,
    pub mesh: Handle<Mesh>,
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
            bounding_box: Vec4::new(0.0, 0.0, 0.0, 16.0),
            render_pipelines: RenderPipelines::from_handles(&[RAY_PIPELINE_HANDLE.typed()]),
            mesh: RAY_PIPELINE_CUBE_HANDLE.typed()
        }
    }
}