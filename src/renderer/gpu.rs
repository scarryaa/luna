use anyhow::anyhow;
use wgpu::{Adapter, Device, Instance, Queue};

pub struct GpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl GpuContext {
    pub async fn new() -> crate::Result<Self> {
        let instance = Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok_or_else(|| anyhow!("Failed to find adapter"))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }
}
