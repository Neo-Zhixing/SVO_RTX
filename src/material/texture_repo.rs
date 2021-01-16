
use bevy::render::texture::{
    Extent3d,
};
use image::{DynamicImage, GenericImageView};
use std::collections::hash_map;
use std::path::Path;

pub struct TextureRepo {
    width: u32,
    height: u32,
    pub(crate) textures: hash_map::HashMap<TextureRepoHandle, DynamicImage>,
    pub(crate) length: u16,
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct TextureRepoHandle(pub(crate) u16);

impl TextureRepo {
    pub fn new(width: u32, height: u32) -> Self {
        TextureRepo {
            width,
            height,
            textures: hash_map::HashMap::new(),
            length: 0,
        }
    }
    pub fn load<'a, P: AsRef<Path>>(&mut self, path: P) -> TextureRepoHandle {
        let image = image::open(path).unwrap();
        assert_eq!(image.width(), self.width);
        assert_eq!(image.height(), self.height);
        let handle = TextureRepoHandle(self.length);
        self.length += 1;
        self.textures.insert(handle, image);
        handle
    }
    pub fn set<'a, P: AsRef<Path>>(&mut self, handle: TextureRepoHandle, path: P) {
        let image = image::open(path).unwrap();
        assert_eq!(image.width(), self.width);
        assert_eq!(image.height(), self.height);
        self.textures.insert(handle, image);
    }
    pub fn get_extent(&self) -> Extent3d {
        Extent3d {
            width: self.width,
            height: self.height,
            depth: self.length as u32,
        }
    }
}
