use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::render::camera::PerspectiveProjection;

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_sky::SkyPlugin;
use ray_tracing::lights::SunLight;
use ray_tracing::material::texture_repo::TextureRepo;
use ray_tracing::material::{
    ColoredMaterial, Material, MaterialPalette, DEFAULT_MATERIAL_PALETTE_HANDLE,
};
use ray_tracing::raytracer::chunk::{Chunk, ChunkBundle};
use ray_tracing::OctreeRayTracerPlugin;
use ray_tracing::Voxel;
use svo::octree::Octree;

/// This example illustrates how to load shaders such that they can be
/// edited while the example is still running.
fn main() {
    App::build()
        // Bevy plugins
        .add_plugin(LogPlugin::default())
        .add_plugin(bevy::reflect::ReflectPlugin::default())
        .add_plugin(bevy::core::CorePlugin::default())
        .add_plugin(bevy::transform::TransformPlugin::default())
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin::default())
        .add_plugin(bevy::input::InputPlugin::default())
        .add_plugin(bevy::window::WindowPlugin::default())
        .add_plugin(bevy::asset::AssetPlugin::default())
        .add_plugin(bevy::render::RenderPlugin::default())
        .add_plugin(bevy::winit::WinitPlugin::default())
        .add_plugin(bevy::wgpu::WgpuPlugin::default())
        .add_plugin(DiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        // Custom plugins
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup.system())
        .insert_resource(TextureRepo::new(512, 512))
        .add_plugin(OctreeRayTracerPlugin::default())
        .add_plugin(SkyPlugin)
        .add_system(my_system.system())
        .run();
}

fn setup(
    commands: &mut Commands,
    mut chunks: ResMut<Assets<Chunk>>,
    mut texture_repo: ResMut<TextureRepo>,
    mut material_palettes: ResMut<Assets<MaterialPalette>>,
) {
    let mut colored_material = ColoredMaterial::default();
    colored_material.color_palette[1] = Color::BLUE;
    colored_material.color_palette[2] = Color::YELLOW;
    colored_material.color_palette[3] = Color::RED;
    colored_material.color_palette[4] = Color::GREEN;
    let scale: f32 = 1.0;
    let stone_material = Material {
        name: "stone".into(),
        scale,
        diffuse: Some(texture_repo.load("assets/textures/stone.png")),
        normal: None,
    };
    let log_material = Material {
        name: "log".into(),
        scale,
        diffuse: Some(texture_repo.load("assets/textures/log_oak.png")),
        normal: None,
    };
    let grass_material = ColoredMaterial {
        material: Material {
            name: "grass".into(),
            scale,
            diffuse: Some(texture_repo.load("assets/textures/grass.png")),
            normal: None,
        },
        color_palette: [Color::GREEN; 256],
    };
    let leaves_material = ColoredMaterial {
        material: Material {
            name: "leaves".into(),
            scale,
            diffuse: Some(texture_repo.load("assets/textures/leaves_oak.png")),
            normal: None,
        },
        color_palette: [Color::GREEN; 256],
    };
    let dirt_material = Material {
        name: "dirt".into(),
        scale,
        diffuse: Some(texture_repo.load("assets/textures/dirt.png")),
        normal: None,
    };
    let sand_material = Material {
        name: "sand".into(),
        scale,
        diffuse: Some(texture_repo.load("assets/textures/sand.png")),
        normal: None,
    };

    let palette = material_palettes
        .get_mut(DEFAULT_MATERIAL_PALETTE_HANDLE)
        .unwrap();

    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 1, "Format: mcanvil <mca filepath>");
    let region_dir = args[0].clone();
    println!("Using region dir {}", region_dir);

    let colored_voxel = palette.add_colored_material(colored_material);
    let grass_voxel = palette.add_colored_material(grass_material);
    let leaves_voxel = palette.add_colored_material(leaves_material);
    let stone_voxel = palette.add_material(stone_material);
    let log_voxel = palette.add_material(log_material);
    let dirt_voxel = palette.add_material(dirt_material);
    let sand_voxel = palette.add_material(sand_material);

    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 1, "Format: mcanvil <mca filepath>");
    let mut octree: Octree<Voxel> = Octree::new();
    let mut load_region = |region_x: usize, region_y: usize| {
        let file =
            std::fs::File::open(format!("{}/r.{}.{}.mca", region_dir, region_x, region_y)).unwrap();
        let mut region = fastanvil::Region::new(file);

        region
            .for_each_chunk(|chunk_x, chunk_z, chunk_data| {
                println!("loading chunk {} {}", chunk_x, chunk_z);
                let chunk: fastanvil::Chunk =
                    fastnbt::de::from_bytes(chunk_data.as_slice()).unwrap();

                if let Some(sections) = chunk.level.sections {
                    for section in sections {
                        if section.palette.is_none() {
                            continue;
                        }
                        let palette = section.palette.unwrap();
                        if let Some(block_states) = section.block_states {
                            let bits_per_item = (block_states.0.len() * 8) / 4096;
                            let mut buff: [u16; 4096] = [0; 4096];
                            block_states.unpack_into(bits_per_item, &mut buff);
                            for (i, indice) in buff.iter().enumerate() {
                                let indice = *indice;
                                let block = &palette[indice as usize];
                                let x = (i & 0xF) as u32;
                                let z = ((i >> 4) & 0xF) as u32;
                                let y = (i >> 8) as u32;

                                let y = y + section.y as u32 * 16;
                                assert_eq!(i >> 12, 0);
                                let voxel = match block.name {
                                    "minecraft:air" => continue,
                                    "minecraft:cave_air" => continue,
                                    "minecraft:stone" => colored_voxel.with_color(3),
                                    "minecraft:granite" => colored_voxel.with_color(3),
                                    "minecraft:gravel" => colored_voxel.with_color(3),
                                    "minecraft:diorite" => colored_voxel.with_color(3),
                                    "minecraft:iron_ore" => colored_voxel.with_color(3),
                                    "minecraft:coal_ore" => colored_voxel.with_color(3),
                                    "minecraft:andesite" => colored_voxel.with_color(3),
                                    "minecraft:bedrock" => colored_voxel.with_color(3),
                                    "minecraft:grass" => continue,
                                    "minecraft:tall_grass" => continue,
                                    "minecraft:grass_block" => colored_voxel.with_color(4),
                                    "minecraft:oak_log" => colored_voxel.with_color(1),
                                    "minecraft:oak_leaves" => colored_voxel.with_color(4),
                                    "minecraft:acacia_leaves" => colored_voxel.with_color(4),
                                    "minecraft:dirt" => colored_voxel.with_color(3),
                                    "minecraft:water" => colored_voxel.with_color(1),
                                    "minecraft:sand" => colored_voxel.with_color(2),
                                    "minecraft:lava" => colored_voxel.with_color(3),
                                    _ => {
                                        //println!("Missing block: w {:?}", block.name);
                                        colored_voxel
                                    }
                                };
                                octree.set(
                                    x + chunk_x as u32 * 16 + region_x as u32 * 512,
                                    y,
                                    z + chunk_z as u32 * 16 + region_y as u32 * 512,
                                    1024,
                                    voxel,
                                );
                            }
                        }
                    }
                }
            })
            .unwrap();

    };

    load_region(1, 0);
    load_region(0, 0);
    load_region(1, 1);
    load_region(0, 1);


    let chunk = Chunk::new(
        octree,
        Vec4::new(0.0, 0.0, 0.0, 1024.0),
    );
    let chunk_handle = chunks.add(chunk);
    commands.spawn(ChunkBundle::new(chunk_handle));


    commands
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            perspective_projection: PerspectiveProjection {
                near: 0.1,
                far: 16384.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(FlyCamera {
            sensitivity: 1.0,
            ..FlyCamera::default()
        });
}

fn my_system(mut sun_light_resource: ResMut<SunLight>, time: Res<Time>) {
    sun_light_resource.direction.x = (time.seconds_since_startup()).cos() as f32;
    sun_light_resource.direction.z = (time.seconds_since_startup()).sin() as f32;
    sun_light_resource.direction.y = -1.0;
}
