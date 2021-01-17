use bevy::render::texture::Extent3d;
use image::{DynamicImage, GenericImageView};
use std::collections::hash_map;
use std::path::Path;
use bevy::core::Bytes;
use std::num::NonZeroU16;

pub struct TextureRepo {
    width: u32,
    height: u32,
    pub(crate) textures: hash_map::HashMap<TextureRepoHandle, DynamicImage>,
    length: u16,
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct TextureRepoHandle(NonZeroU16);

impl TextureRepoHandle {
    pub fn get(&self) -> u16 {
        self.0.get()
    }
}

impl Bytes for TextureRepoHandle {
    fn write_bytes(&self, buffer: &mut [u8]) {
        self.0.get().write_bytes(buffer)
    }

    fn byte_len(&self) -> usize {
        std::mem::size_of::<NonZeroU16>()
    }
}

impl TextureRepo {
    pub fn new(width: u32, height: u32) -> Self {
        TextureRepo {
            width,
            height,
            textures: hash_map::HashMap::new(),
            length: 0
        }
    }
    pub fn drain(&mut self) -> impl Iterator<Item=(TextureRepoHandle, DynamicImage)> + '_ {
        self.textures.drain()
    }
    pub fn len(&self) -> u16 {
        self.length
    }
    pub fn load<'a, P: AsRef<Path>>(&mut self, path: P) -> TextureRepoHandle {
        let image = image::open(path).unwrap();
        assert_eq!(image.width(), self.width);
        assert_eq!(image.height(), self.height);
        self.length += 1;
        let handle = TextureRepoHandle(
            unsafe {
                NonZeroU16::new_unchecked(self.length)
            }
        );
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
            depth: self.textures.len() as u32,
        }
    }
}
