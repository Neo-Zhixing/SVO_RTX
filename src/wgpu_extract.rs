use std::sync::Arc;
use wgpu::Features;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Context(pub wgpu_core::hub::Global<wgpu_core::hub::IdentityManagerFactory>);

#[derive(Debug)]
pub struct DeviceId {
    id: wgpu_core::id::DeviceId,
    error_sink: u64,
    pub features: Features,
}


#[derive(Debug)]
pub struct Device {
    pub context: Arc<Context>,
    pub id: DeviceId,
}

pub unsafe trait WgpuExtract<T> {
    unsafe fn extract(&self) -> &T {
        &*(self as *const Self as *const T)
    }
    unsafe fn extract_mut(&mut self) -> &mut T {
        &mut *(self as *mut Self as *mut T)
    }
}

unsafe impl WgpuExtract<Device> for wgpu::Device {}

#[derive(Debug)]
pub(crate) struct CommandEncoderId {
    id: wgpu_core::id::CommandEncoderId,
    error_sink: u64,
}

#[derive(Debug)]
pub struct CommandEncoder {
    context: Arc<Context>,
    id: CommandEncoderId,
    /// This type should be !Send !Sync, because it represents an allocation on this thread's
    /// command buffer.
    _p: PhantomData<*const u8>,
}


pub struct RenderPass<'a> {
    pub id: wgpu_core::command::RenderPass,
    pub parent: &'a mut CommandEncoder,
}

unsafe impl<'a> WgpuExtract<RenderPass<'a>> for wgpu::RenderPass<'a> {}
