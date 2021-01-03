use svo::octree::Octree;
use bevy::reflect::TypeUuid;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;

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

