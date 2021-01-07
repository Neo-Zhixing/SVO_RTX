use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::draw::{DrawContext, RenderCommand};
use bevy::render::pass::{
    LoadOp, Operations, PassDescriptor, RenderPassColorAttachmentDescriptor,
    RenderPassDepthStencilAttachmentDescriptor, TextureAttachment,
};
use bevy::render::pipeline::{BlendDescriptor, ColorStateDescriptor, ColorWrite, CompareFunction, DepthStencilStateDescriptor, IndexFormat, InputStepMode, PipelineDescriptor, PipelineSpecialization, PrimitiveTopology, RenderPipeline, StencilStateDescriptor, StencilStateFaceDescriptor, VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat, BlendFactor, BlendOperation, RasterizationStateDescriptor, FrontFace, CullMode};
use bevy::render::render_graph::base as base_render_graph;
use bevy::render::render_graph::{PassNode, RenderGraph, WindowSwapChainNode, WindowTextureNode};
use bevy::render::renderer::{
    BufferId, BufferInfo, BufferUsage, RenderResourceBindings,
    RenderResourceContext,
};
use bevy::render::shader::ShaderStages;
use bevy::render::texture::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage,
};
use bevy::window::WindowId;
use crate::raytracer::projection_node::CameraProjectionNode;
use crate::raytracer::chunk::Chunk;
use crate::raytracer::octree_node::OctreeNode;
use bevy::prelude::shape::Cube;
use bevy::core::AsBytes;

pub mod projection_node;
pub mod chunk;
pub mod octree_node;

pub const RAY_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 0x786f7ab62875ebbc);

#[derive(Default)]
pub struct OctreeRayTracerPlugin;

pub struct RayTracerSharedResources {
    cube_vertex_buffer: BufferId,
    cube_index_buffer: BufferId,
}

/// A component that indicates that an entity should be drawn in the "main pass"
#[derive(Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct RayPass;

pub mod node {
    pub const RAY_PASS: &str = "ray_pass";
    pub const PROJECTION_NODE: &str = "ray_projection_node";
    pub const OCTREE_CHUNK_NODE: &str = "octree_chunk_node";
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

            ray_pass_node.use_default_clear_color(0);
            ray_pass_node.add_camera(base_render_graph::camera::CAMERA_3D);
            render_graph.add_node(node::RAY_PASS, ray_pass_node);
            render_graph.add_system_node(node::PROJECTION_NODE, CameraProjectionNode::new(base_render_graph::camera::CAMERA_3D));
            render_graph.add_node_edge(node::PROJECTION_NODE, node::RAY_PASS).unwrap();
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
            render_graph
                .add_system_node(node::OCTREE_CHUNK_NODE, OctreeNode::new());
            render_graph
                .add_node_edge(node::OCTREE_CHUNK_NODE, node::RAY_PASS);


            // ensure ray pass runs after main pass
            // So that pixels covered by UI / Mesh rendered objects will not be traced
            render_graph.add_node_edge(base_render_graph::node::MAIN_PASS, node::RAY_PASS).unwrap();
        }

        app.add_system_to_stage(
            bevy::render::stage::DRAW,
            draw_raytracing_pipelines_system.system(),
        );

        let (cube_vertex_buffer, cube_index_buffer) = {
            // Adding the quad mesh
            let render_resource_context = app
                .resources()
                .get::<Box<dyn RenderResourceContext>>()
                .unwrap();
            let render_resource_context = &**render_resource_context;

            let vertices: [f32; 24] = [
                0.0, 0.0, 0.0, // 0
                0.0, 0.0, 1.0, // 1
                0.0,1.0, 0.0, // 2
                0.0,1.0, 1.0, // 3
                1.0, 0.0, 0.0, // 4
                1.0, 0.0, 1.0, // 5
                1.0, 1.0, 0.0, // 6
                1.0, 1.0, 1.0 // 7
            ];
            let vertex_buffer = render_resource_context.create_buffer_with_data(
                BufferInfo {
                    buffer_usage: BufferUsage::VERTEX,
                    ..Default::default()
                },
                &vertices.as_bytes(),
            );
            let indices: [u16; 14] = [
                3, 7, 1, 5,
                4, 7, 6, 3,
                2, 1, 0, 4,
                2, 6
            ];
            let index_buffer = render_resource_context.create_buffer_with_data(
                BufferInfo {
                    buffer_usage: BufferUsage::INDEX,
                    ..Default::default()
                },
                &indices.as_bytes()
            );
            (vertex_buffer, index_buffer)
        };
        app
            .add_asset::<Chunk>();
        app.add_resource(RayTracerSharedResources {
            cube_vertex_buffer,
            cube_index_buffer
        });

        let resources = app.resources();
        let asset_server = resources.get_mut::<AssetServer>().unwrap();

        asset_server.watch_for_changes().unwrap();
        let mut pipelines = resources.get_mut::<Assets<PipelineDescriptor>>().unwrap();
        pipelines.set_untracked(
            RAY_PIPELINE_HANDLE,
            PipelineDescriptor {
                name: Some("octree_raytracing_pipeline".into()),
                primitive_topology: PrimitiveTopology::TriangleStrip,
                layout: None,
                index_format: IndexFormat::Uint16,
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
                rasterization_state: Some(RasterizationStateDescriptor {
                    front_face: FrontFace::Cw,
                    cull_mode: CullMode::Front,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                    clamp_depth: false
                }),
                depth_stencil_state: Some(DepthStencilStateDescriptor {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilStateDescriptor {
                        front: StencilStateFaceDescriptor::IGNORE,
                        back: StencilStateFaceDescriptor::IGNORE,
                        read_mask: 0,
                        write_mask: 0,
                    },
                }),
                color_states: vec![ColorStateDescriptor {
                    format: TextureFormat::default(),
                    color_blend: BlendDescriptor {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                    alpha_blend: BlendDescriptor {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    write_mask: ColorWrite::ALL,
                }],
                shader_stages: ShaderStages {
                    vertex: asset_server.load::<Shader, _>("shaders/ray.vert"),
                    fragment: Some(asset_server.load::<Shader, _>("shaders/ray.frag")),
                },
            },
        );
    }
}

#[derive(Default, Reflect)]
#[reflect(Component)]
pub struct RayTracer {
    render_pipeline: RenderPipeline,

    #[reflect(ignore)]
    bindings: RenderResourceBindings,
}

#[derive(Bundle)]
pub struct OctreeRaytracerBundle {
    pub draw: Draw,
    pub ray_tracer: RayTracer,
    pub visible: Visible,
    pub ray_pass: RayPass,
}

impl Default for OctreeRaytracerBundle {
    fn default() -> Self {
        OctreeRaytracerBundle {
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
        }
    }
}

pub fn draw_raytracing_pipelines_system(
    mut draw_context: DrawContext,
    mut render_resource_bindings: ResMut<RenderResourceBindings>,
    shared_resources: Res<RayTracerSharedResources>,
    mut query: Query<(&mut Draw, &mut RayTracer)>,
) {
    for (mut draw, mut ray_tracer) in query.iter_mut() {
        ray_tracer.bindings.vertex_attribute_buffer = Some(shared_resources.cube_vertex_buffer);
        ray_tracer.bindings.index_buffer = Some(shared_resources.cube_index_buffer);

        draw_context
            .set_pipeline(
                &mut draw,
                &ray_tracer.render_pipeline.pipeline,
                &ray_tracer.render_pipeline.specialization,
            )
            .unwrap();
        let render_resource_bindings: &mut [&mut RenderResourceBindings] =
            &mut [&mut ray_tracer.bindings, &mut render_resource_bindings];
        draw_context
            .set_bind_groups_from_bindings(&mut draw, render_resource_bindings)
            .unwrap();
        draw_context
            .set_vertex_buffers_from_bindings(&mut draw, &[&ray_tracer.bindings])
            .unwrap();
        draw.draw_indexed(0..14, 0, 0..1);
    }
}
