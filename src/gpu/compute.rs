
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
) -> Vec<Particle> {


    // Step 1: Convert to GPU format
    let gpu_particles: Vec<GpuParticle> = input_particles.iter().map(|p| GpuParticle::from(*p)).collect();

    // Step 2: Create GPU buffer
    let size = (gpu_particles.len() * std::mem::size_of::<GpuParticle>()) as wgpu::BufferAddress;

    let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("StagingBuffer"),
        contents: bytemuck::cast_slice(&gpu_particles),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
    });

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("OutputBuffer"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    // Step 3: Create bind group layout & pipeline
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("BindGroupLayout"),
    entries: &[
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

    let params = SimParams {
        dt: 0.016,
        width: 300,
        height: 300,
    };

    let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ParamsBuffer"),
        contents: bytemuck::cast_slice(&[params]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let occupancy_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("OccupancyBuffer"),
        size: (params.width * params.height * std::mem::size_of::<u32>() as u32) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("BindGroup"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: staging_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: occupancy_buffer.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("PipelineLayout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    label: Some("ComputePipeline"),
    layout: Some(&pipeline_layout),
    module: shader,
    entry_point: "main",
    compilation_options: wgpu::PipelineCompilationOptions::default(),
});

    // Step 4: Dispatch compute shader
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Compute Encoder"),
    });

    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups((gpu_particles.len() as u32 + 63) / 64, 1, 1);
    }

    // Step 5: Copy result to output buffer
    encoder.copy_buffer_to_buffer(&staging_buffer, 0, &output_buffer, 0, size);
    let zero_occupancy = vec![0u32; (params.width * params.height) as usize];
    queue.write_buffer(&occupancy_buffer, 0, bytemuck::cast_slice(&zero_occupancy));
    queue.submit(Some(encoder.finish()));

    // Step 6: Read back results
    let buffer_slice = output_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::Maintain::Wait);

    let data = buffer_slice.get_mapped_range();
    let result: &[GpuParticle] = bytemuck::cast_slice(&data);
    let updated_particles: Vec<Particle> = result.iter().map(|gp| Particle::from(*gp)).collect();

    drop(data);
    output_buffer.unmap();

    updated_particles
}
