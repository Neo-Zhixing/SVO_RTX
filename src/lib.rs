pub mod lights;
pub mod material;
pub mod raytracer;
pub mod wgpu_extract;

pub use raytracer::chunk_node::ChunkNode;
pub use raytracer::OctreeRayTracerPlugin;
pub use raytracer::RayPass;

use std::fmt::{Debug, Formatter, Result};

#[derive(Copy, Clone, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Voxel(u16);

// 0:  Air
// 0x01 - 0x80: 2^15 - 1 = 32767 standarlone blocks
// 0x80 - 0xFF:
#[derive(Debug)]
pub enum VoxelData {
    Regular(u16),
    Colored(u8, u8),
}
impl Voxel {
    pub fn new_colored(id: u8, color: u8) -> Self {
        assert_eq!(id & 0x80, 0, "Colored voxel has index 0 - 127");
        let id = id | 0x80;
        Voxel(((id as u16) << 8) | (color as u16))
    }
    pub fn new(id: u16) -> Self {
        assert_eq!(id & 0x8000, 0, "Colored voxel has index 0 - 0x8f");
        Voxel(id)
    }
    pub fn get(&self) -> VoxelData {
        if self.0 & 0x8000 == 0 {
            VoxelData::Regular(self.0)
        } else {
            let voxel_id = (self.0 >> 8) as u8 & 0x7f;
            let color = (self.0 & 0xff) as u8;
            VoxelData::Colored(voxel_id, color)
        }
    }
    pub fn with_color(&self, color: u8) -> Voxel {
        match self.get() {
            VoxelData::Regular(_) => self.clone(),
            VoxelData::Colored(id, _) => Voxel::new_colored(id, color),
        }
    }
}

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

impl Debug for Voxel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.get().fmt(f)
    }
}
