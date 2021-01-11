use svo::octree::Octree;
use bevy::reflect::TypeUuid;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;
use crate::raytracer::{RayPass, RAY_PIPELINE_HANDLE, RAY_PIPELINE_CUBE_HANDLE};
use bevy::prelude::*;
use bevy::render::pipeline::{RenderPipeline, PipelineSpecialization, IndexFormat, PrimitiveTopology, VertexBufferDescriptor, InputStepMode, VertexAttributeDescriptor, VertexFormat};
use bevy::render::renderer::BufferId;

#[derive(Copy, Clone, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Voxel(pub u16);

impl svo::Voxel for Voxel {
    fn avg(arr: &[Self; 8]) -> Self {
        // find most frequent element
        let mut arr = arr.clone();
        arr.sort();

        let mut count: u8 = 0;
        let mut max_count: u8 = 0;
        let mut max_element: u16 = 0;
        let mut last_element: u16 = 0;
        for i in &arr {
            if i.0 != last_element {
                if count > max_count {
                    max_count = count;
                    max_element = i.0;
                }
                count = 0;
            }
            count += 1;
            last_element = i.0;
        }
        Voxel(max_element)
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
            }
        }
    }
}