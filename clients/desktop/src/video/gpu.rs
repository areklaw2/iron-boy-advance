use getset::Getters;

#[derive(Getters)]
pub struct GpuContext {
    #[getset(get = "pub")]
    instance: wgpu::Instance,
    #[getset(get = "pub")]
    adapter: wgpu::Adapter,
    #[getset(get = "pub")]
    device: wgpu::Device,
    #[getset(get = "pub")]
    queue: wgpu::Queue,
}

impl GpuContext {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("failed to find suitable wgpu adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("ironboyadvance-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("failed to acquire wgpu device");

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}
