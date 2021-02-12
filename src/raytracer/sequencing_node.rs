
use std::borrow::Cow;
use bevy::render::renderer::{RenderResourceId, RenderResourceType, RenderContext};
use bevy::render::render_graph::{Node, ResourceSlotInfo, ResourceSlots};
use bevy::prelude::*;

/// For each frame, the sequencing node sends one of the input textures as the output
pub struct SequencingNode {
    state: u32,
    inputs: Vec<ResourceSlotInfo>,
}

impl SequencingNode {
    pub const OUT_TEXTURE: &'static str = "texture";

    pub fn new(size: u32) -> Self {
        let mut inputs: Vec<ResourceSlotInfo> = Vec::with_capacity(size as usize);
        for _ in 0..size {
            inputs.push(ResourceSlotInfo {
                name: Default::default(),
                resource_type: RenderResourceType::Texture
            })
        }
        SequencingNode {
            state: 0,
            inputs
        }
    }
}

impl Node for SequencingNode {
    fn output(&self) -> &[ResourceSlotInfo] {
        static OUTPUT: &[ResourceSlotInfo] = &[ResourceSlotInfo {
            name: Cow::Borrowed(SequencingNode::OUT_TEXTURE),
            resource_type: RenderResourceType::Texture,
        }];
        OUTPUT
    }

    fn input(&self) -> &[ResourceSlotInfo] {
        self.inputs.as_slice()
    }
    fn update(
        &mut self,
        _world: &World,
        _resources: &Resources,
        _render_context: &mut dyn RenderContext,
        input: &ResourceSlots,
        output: &mut ResourceSlots,
    ) {
        self.state = self.state % input.len() as u32;
        let resource = input.get(self.state as usize).unwrap();
        output.set(0, resource);

        self.state += 1;
    }
}
