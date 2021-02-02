use crate::lights::node::LightsNode;
use crate::lights::{AmbientLight, SunLight};
use crate::material::material_node::MaterialNode;

use crate::material::texture_repo_node::TextureRepoNode;
use crate::material::{MaterialPalette, DEFAULT_MATERIAL_PALETTE_HANDLE};
use crate::raytracer::chunk::Chunk;
use crate::raytracer::chunk_node::ChunkNode;

use bevy::prelude::*;
use bevy::reflect::TypeUuid;

use bevy::render::mesh::Indices;
use bevy::render::pass::{
    LoadOp, Operations, PassDescriptor, RenderPassColorAttachmentDescriptor,
    RenderPassDepthStencilAttachmentDescriptor, TextureAttachment,
};
use bevy::render::pipeline::{
    BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrite, CompareFunction,
    CullMode, DepthBiasState, DepthStencilState, FrontFace, IndexFormat, MultisampleState,
    PipelineDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, StencilFaceState,
    StencilState,
};
use bevy::render::render_graph::base as base_render_graph;
use bevy::render::render_graph::{PassNode, RenderGraph, WindowSwapChainNode, WindowTextureNode};

use bevy::render::shader::{ShaderStage, ShaderStages};
use bevy::render::texture::TextureFormat;

pub mod chunk;
pub mod chunk_node;

pub const RAY_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 0x786f7ab62875ebbc);
pub const RAY_PIPELINE_CUBE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Mesh::TYPE_UUID, 0x786f7ab62875ebbd);

#[derive(Default)]
pub struct OctreeRayTracerPlugin;

/// A component that indicates that an entity should be drawn in the "main pass"
#[derive(Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct RayPass;

pub mod node {
    pub const RAY_PASS: &str = "ray_pass";
    pub const OCTREE_CHUNK_NODE: &str = "octree_chunk_node";
    pub const LIGHT_NODE: &str = "light_node";
    pub const TEXTURE_REPO: &str = "texture_repo_node";
    pub const MATERIAL_REPO: &str = "material_repo_node";
}

impl Plugin for OctreeRayTracerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        {
            // Build render graph
            let mut render_graph = app.resources_mut().get_mut::<RenderGraph>().unwrap();
            let mut ray_pass_node = PassNode::<&RayPass>::new(PassDescriptor {
                color_attachments: vec![RenderPassColorAttachmentDescriptor {
                    attachment: TextureAttachment::Input("color_attachment".to_string()),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: TextureAttachment::Input("depth".to_string()),
                    depth_ops: Some(Operations {
                        load: LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
                sample_count: 1,
            });

            ray_pass_node.add_camera(base_render_graph::camera::CAMERA_3D);
            render_graph.add_node(node::RAY_PASS, ray_pass_node);
            render_graph
                .add_node_edge(base_render_graph::node::TEXTURE_COPY, node::RAY_PASS)
                .unwrap();
            render_graph
                .add_node_edge(base_render_graph::node::SHARED_BUFFERS, node::RAY_PASS)
                .unwrap();
            render_graph
                .add_node_edge(base_render_graph::node::CAMERA_3D, node::RAY_PASS)
                .unwrap();
            render_graph
                .add_slot_edge(
                    base_render_graph::node::PRIMARY_SWAP_CHAIN,
                    WindowSwapChainNode::OUT_TEXTURE,
                    node::RAY_PASS,
                    "color_attachment",
                )
                .unwrap();
            render_graph
                .add_slot_edge(
                    base_render_graph::node::MAIN_DEPTH_TEXTURE,
                    WindowTextureNode::OUT_TEXTURE,
                    node::RAY_PASS,
                    "depth",
                )
                .unwrap();

            // Octree chunks
            render_graph.add_system_node(node::OCTREE_CHUNK_NODE, ChunkNode::new());
            render_graph
                .add_node_edge(node::OCTREE_CHUNK_NODE, node::RAY_PASS)
                .unwrap();

            // Materials
            render_graph.add_system_node(node::MATERIAL_REPO, MaterialNode::new());
            render_graph
                .add_node_edge(node::MATERIAL_REPO, node::RAY_PASS)
                .unwrap();
            // Textures
            render_graph.add_node(node::TEXTURE_REPO, TextureRepoNode::new());
            render_graph
                .add_node_edge(node::TEXTURE_REPO, node::RAY_PASS)
                .unwrap();

            // Adding lights
            render_graph.add_system_node(node::LIGHT_NODE, LightsNode::new(16));
            render_graph
                .add_node_edge(node::LIGHT_NODE, node::RAY_PASS)
                .unwrap();

            // ensure ray pass runs after main pass
            // So that pixels covered by UI / Mesh rendered objects will not be traced
            render_graph
                .add_node_edge(base_render_graph::node::MAIN_PASS, node::RAY_PASS)
                .unwrap();
        }

        {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
            mesh.set_indices(Some(Indices::U16(vec![
                3, 7, 1, 5, 4, 7, 6, 3, 2, 1, 0, 4, 2, 6,
            ])));
            mesh.set_attribute(
                Mesh::ATTRIBUTE_POSITION,
                vec![
                    [0.0, 0.0, 0.0], // 0
                    [0.0, 0.0, 1.0], // 1
                    [0.0, 1.0, 0.0], // 2
                    [0.0, 1.0, 1.0], // 3
                    [1.0, 0.0, 0.0], // 4
                    [1.0, 0.0, 1.0], // 5
                    [1.0, 1.0, 0.0], // 6
                    [1.0, 1.0, 1.0], // 7
                ],
            );
            // Adding the quad mesh
            app.resources()
                .get_mut::<Assets<Mesh>>()
                .unwrap()
                .set_untracked(RAY_PIPELINE_CUBE_HANDLE, mesh);
        };
        app.add_asset::<Chunk>()
            .add_asset::<MaterialPalette>()
            .insert_resource(AmbientLight {
                color: Color::rgb_linear(0.2, 0.2, 0.2),
            })
            .insert_resource(SunLight {
                color: Color::rgb_linear(0.8, 0.8, 0.8),
                direction: Vec3::new(0.5, 0.5, 0.5).normalize(),
            });

        let resources = app.resources();
        {
            let mut palettes = resources.get_mut::<Assets<MaterialPalette>>().unwrap();
            palettes.set_untracked(DEFAULT_MATERIAL_PALETTE_HANDLE, MaterialPalette::new())
        }
        let mut shaders = resources.get_mut::<Assets<Shader>>().unwrap();

        let mut pipelines = resources.get_mut::<Assets<PipelineDescriptor>>().unwrap();
        pipelines.set_untracked(
            RAY_PIPELINE_HANDLE,
            PipelineDescriptor {
                name: Some("octree_raytracing_pipeline".into()),
                layout: None,
                color_target_states: vec![ColorTargetState {
                    format: TextureFormat::default(),
                    color_blend: BlendState {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                    alpha_blend: BlendState {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    write_mask: ColorWrite::ALL,
                }],
                shader_stages: ShaderStages {
                    vertex: shaders.add(Shader::from_glsl(
                        ShaderStage::Vertex,
                        include_str!("../../assets/shaders/ray.vert"),
                    )),
                    fragment: Some(shaders.add(Shader::from_glsl(
                        ShaderStage::Fragment,
                        include_str!("../../assets/shaders/ray.frag"),
                    ))),
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleStrip,
                    strip_index_format: Some(IndexFormat::Uint16),
                    front_face: FrontFace::Cw,
                    cull_mode: CullMode::Front,
                    polygon_mode: PolygonMode::Fill,
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState {
                        front: StencilFaceState::IGNORE,
                        back: StencilFaceState::IGNORE,
                        read_mask: 0,
                        write_mask: 0,
                    },
                    bias: DepthBiasState {
                        constant: 0,
                        slope_scale: 0.0,
                        clamp: 0.0,
                    },
                    clamp_depth: false,
                }),
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            },
        );
    }
}
