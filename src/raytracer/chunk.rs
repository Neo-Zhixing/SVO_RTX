use svo::octree::Octree;
use bevy::reflect::TypeUuid;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;
use crate::raytracer::{RayPass, RayTracer, RAY_PIPELINE_HANDLE};
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
    pub ray_tracer: RayTracer,
    pub visible: Visible,
    pub ray_pass: RayPass,
    pub chunk: Handle<Chunk>,
    pub bounding_box: Vec4,
}

impl ChunkBundle {
    pub fn new(chunk: Handle<Chunk>) -> Self {
        ChunkBundle {
            chunk,
            ray_tracer: RayTracer {
                render_pipeline: RenderPipeline {
                    pipeline: RAY_PIPELINE_HANDLE.typed(),
                    specialization: PipelineSpecialization {
                        sample_count: 1,
                        index_format: IndexFormat::Uint16,
                        shader_specialization: Default::default(),
                        primitive_topology: PrimitiveTopology::TriangleStrip,
                        dynamic_bindings: Default::default(),
                        vertex_buffer_descriptor: VertexBufferDescriptor {
                            name: "ray_blit_vertex_buffer".into(),
                            stride: (std::mem::size_of::<f32>() * 3) as u64,
                            step_mode: InputStepMode::Vertex,
                            attributes: vec![VertexAttributeDescriptor {
                                name: "position".into(),
                                offset: 0,
                                format: VertexFormat::Float3,
                                shader_location: 0,
                            }],
                        },
                    },
                    dynamic_bindings_generation: std::usize::MAX,
                },
                bindings: Default::default(),
            },
            draw: Default::default(),
            visible: Visible {
                is_visible: true,
                is_transparent: false,
            },
            ray_pass: RayPass,
            bounding_box: Vec4::new(0.0, 0.0, 0.0, 16.0)
        }
    }
}