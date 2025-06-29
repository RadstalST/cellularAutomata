use crate::domain::particle::{Particle, GpuParticle};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimParams {
    pub dt: f32,
    pub width: u32,
    pub height: u32,
}


pub async fn initialize_gpu() -> (wgpu::Device, wgpu::Queue, wgpu::ShaderModule) {
    use wgpu::Instance;

    let instance = Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        dx12_shader_compiler: Default::default(),
        flags: wgpu::InstanceFlags::default(),
        gles_minor_version: wgpu::Gles3MinorVersion::default(),
    });
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    (device, queue, shader)
}


pub async fn dispatch_particles(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shader: &wgpu::ShaderModule,
    input_particles: &[Particle],
    input_occupancy: &[u32],
) -> (Vec<Particle>, Vec<u32>) {
    // 1) Prepare GPU‐side data
    let gpu_particles: Vec<GpuParticle> = input_particles.iter().map(|p| GpuParticle::from(*p)).collect();
    let particle_size = std::mem::size_of::<GpuParticle>() as wgpu::BufferAddress;
    let particle_buffer_size = particle_size * gpu_particles.len() as wgpu::BufferAddress;

    let occupancy_len = input_occupancy.len();
    let occupancy_size = (occupancy_len * std::mem::size_of::<u32>()) as wgpu::BufferAddress;

    // 2) Create ping-pong buffers (we only actually use "A" in this scheme)
    let particle_buffer_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ParticleBufferA"),
        contents: bytemuck::cast_slice(&gpu_particles),
        usage: wgpu::BufferUsages::STORAGE
             | wgpu::BufferUsages::COPY_SRC
             | wgpu::BufferUsages::COPY_DST,
    });
    let _particle_buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ParticleBufferB"),
        size: particle_buffer_size,
        usage: wgpu::BufferUsages::STORAGE
             | wgpu::BufferUsages::COPY_SRC
             | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let occupancy_buffer_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("OccupancyBufferA"),
        contents: bytemuck::cast_slice(input_occupancy),
        usage: wgpu::BufferUsages::STORAGE
             | wgpu::BufferUsages::COPY_SRC
             | wgpu::BufferUsages::COPY_DST,
    });
    let _occupancy_buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("OccupancyBufferB"),
        size: occupancy_size,
        usage: wgpu::BufferUsages::STORAGE
             | wgpu::BufferUsages::COPY_SRC
             | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // 3) Readback buffers
    let particle_readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ParticleReadback"),
        size: particle_buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let occupancy_readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("OccupancyReadback"),
        size: occupancy_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // 4) Uniforms
    let params = SimParams { dt: 0.016, width: 300, height: 300 };
    let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ParamsBuffer"),
        contents: bytemuck::cast_slice(&[params]),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    // 5) BindGroupLayout
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("BindGroupLayout"),
        entries: &[
            // particles@0
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // params@1
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // occupancy@2
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    // 6) Compute pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("PipelineLayout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("ComputePipeline"),
        layout: Some(&pipeline_layout),
        module: shader,
        entry_point: "main",
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    });

    // 7) Dispatch & copy out
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("CommandEncoder"),
    });

    // Bind A as the only buffer
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("BindGroup"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: particle_buffer_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: occupancy_buffer_a.as_entire_binding(),
            },
        ],
    });

    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        cpass.set_pipeline(&pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        // one workgroup per 64 particles
        cpass.dispatch_workgroups(((gpu_particles.len() as u32) + 63) / 64, 1, 1);
    }

    // Copy to CPU‐readback
    encoder.copy_buffer_to_buffer(&particle_buffer_a, 0, &particle_readback, 0, particle_buffer_size);
    encoder.copy_buffer_to_buffer(&occupancy_buffer_a, 0, &occupancy_readback, 0, occupancy_size);

    queue.submit(Some(encoder.finish()));
    device.poll(wgpu::Maintain::Wait);

    // Clear occupancy on A for next frame
    let zero_occ = vec![0u32; occupancy_len];
    queue.write_buffer(&occupancy_buffer_a, 0, bytemuck::cast_slice(&zero_occ));

    // 8) Read‐back helper
    async fn read_back<T: bytemuck::Pod>(
        buffer: &wgpu::Buffer,
        device: &wgpu::Device,
    ) -> Vec<T> {
        let slice = buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
        device.poll(wgpu::Maintain::Wait);
        receiver.receive().await.unwrap().unwrap();
        let data = slice.get_mapped_range();
        let vec = bytemuck::cast_slice::<u8, T>(&data).to_vec();
        drop(data);
        buffer.unmap();
        vec
    }

    // 9) Convert back to CPU structs
    let raw_particles = read_back::<GpuParticle>(&particle_readback, device).await;
    let updated_particles = raw_particles.into_iter().map(Particle::from).collect();

    let new_occupancy = read_back::<u32>(&occupancy_readback, device).await;

    (updated_particles, new_occupancy)
}
