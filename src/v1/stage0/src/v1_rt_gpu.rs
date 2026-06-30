//! Hand-Rust scaffold: wgpu host realization for accelerator-demo GPU epilogue.
//! dissolve-on: feature:dag-gpu-realization-handler

const RELU_MUL_ADD_OP_CODES: [i64; 3] = [1, 2, 3];

const WGSL_INTEGER_KERNEL: &str = r#"@group(0) @binding(0) var<storage, read> a: array<i32>;
@group(0) @binding(1) var<storage, read> b: array<i32>;
@group(0) @binding(2) var<storage, read> c: array<i32>;
@group(0) @binding(3) var<storage, read_write> out: array<i32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let i = gid.x;
  if i >= arrayLength(&out) { return; }
  let tmp = a[i] * b[i] + c[i];
  out[i] = max(tmp, 0);
}
"#;

const WGSL_FLOAT_KERNEL: &str = r#"@group(0) @binding(0) var<storage, read> a: array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read> c: array<f32>;
@group(0) @binding(3) var<storage, read_write> out: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let i = gid.x;
  if i >= arrayLength(&out) { return; }
  let tmp = a[i] * b[i] + c[i];
  out[i] = max(tmp, 0.0);
}
"#;

fn assert_relu_mul_add_op_codes(op_codes: &[i64]) {
    if op_codes != RELU_MUL_ADD_OP_CODES {
        panic!("unsupported op_codes in wgpu accelerator demo kernel");
    }
}

fn wgpu_backends_from_env() -> wgpu::Backends {
    match std::env::var("GUNBC_WGPU_BACKENDS").as_deref() {
        Ok("metal") => wgpu::Backends::METAL,
        Ok("dx12") => wgpu::Backends::DX12,
        Ok("vulkan") => wgpu::Backends::VULKAN,
        Ok("gl") => wgpu::Backends::GL,
        _ => wgpu::Backends::VULKAN | wgpu::Backends::GL | wgpu::Backends::METAL | wgpu::Backends::DX12,
    }
}

fn force_fallback_adapter() -> bool {
    !matches!(
        std::env::var("GUNBC_WGPU_FORCE_HARDWARE").as_deref(),
        Ok("1") | Ok("true")
    )
}

struct GpuDispatchContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl GpuDispatchContext {
    fn new() -> Self {
        pollster::block_on(async {
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu_backends_from_env(),
                ..Default::default()
            });
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    force_fallback_adapter: force_fallback_adapter(),
                    compatible_surface: None,
                })
                .await
                .unwrap_or_else(|| panic!("wgpu: no adapter"));
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("accelerator_demo_gpu"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::downlevel_defaults(),
                        memory_hints: Default::default(),
                    },
                    None,
                )
                .await
                .expect("wgpu: request_device failed");
            Self { device, queue }
        })
    }

    fn dispatch_bytes(&self, shader: &str, a: &[u8], b: &[u8], c: &[u8], elem_size: usize) -> Vec<u8> {
        let len = a.len() / elem_size;
        assert_eq!(a.len(), b.len());
        assert_eq!(b.len(), c.len());
        pollster::block_on(async {
            let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("relu_mul_add"),
                source: wgpu::ShaderSource::Wgsl(shader.into()),
            });
            let a_buf = self.make_storage_buffer("a", a);
            let b_buf = self.make_storage_buffer("b", b);
            let c_buf = self.make_storage_buffer("c", c);
            let out_bytes = len * elem_size;
            let out_buf = self.make_storage_buffer_rw("out", out_bytes as u64);
            let entries = [
                storage_entry(0, false),
                storage_entry(1, false),
                storage_entry(2, false),
                storage_entry(3, true),
            ];
            let bind_group_layout = self.device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("layout"),
                    entries: &entries,
                },
            );
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bind"),
                layout: &bind_group_layout,
                entries: &[
                    bind_entry(0, &a_buf),
                    bind_entry(1, &b_buf),
                    bind_entry(2, &c_buf),
                    bind_entry(3, &out_buf),
                ],
            });
            let pipeline_layout = self.device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("pipeline_layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                },
            );
            let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("pipeline"),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(((len as u32) + 63) / 64, 1, 1);
            }
            self.queue.submit(std::iter::once(encoder.finish()));
            self.read_buffer(&out_buf, out_bytes as u64).await
        })
    }

    fn make_storage_buffer(&self, label: &str, data: &[u8]) -> wgpu::Buffer {
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: data.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, data);
        buf
    }

    fn make_storage_buffer_rw(&self, label: &str, size: u64) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    async fn read_buffer(&self, src: &wgpu::Buffer, bytes: u64) -> Vec<u8> {
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging"),
            size: bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("readback"),
        });
        encoder.copy_buffer_to_buffer(src, 0, &staging, 0, bytes);
        self.queue.submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            tx.send(r).ok();
        });
        self.device.poll(wgpu::Maintain::wait()).panic_on_timeout();
        rx.recv().expect("map callback").expect("map failed");
        slice.get_mapped_range().to_vec()
    }
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn bind_entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

static GPU_CONTEXT: std::sync::LazyLock<std::sync::Mutex<Option<GpuDispatchContext>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(None));

fn gpu_ctx() -> std::sync::MutexGuard<'static, Option<GpuDispatchContext>> {
    let mut guard = GPU_CONTEXT.lock().expect("gpu context mutex poisoned");
    if guard.is_none() {
        *guard = Some(GpuDispatchContext::new());
    }
    guard
}

pub fn wgpu_probe_adapter() -> Result<String, String> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu_backends_from_env(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: force_fallback_adapter(),
                compatible_surface: None,
            })
            .await
            .ok_or_else(|| "no wgpu adapter".to_string())?;
        let info = adapter.get_info();
        Ok(format!("backend={:?} name={}", info.backend, info.name))
    })
}

pub fn wgpu_elementwise_kernel(op_codes: &[i64], a: &[i64], b: &[i64], c: &[i64]) -> Vec<i64> {
    assert_relu_mul_add_op_codes(op_codes);
    let ai: Vec<i32> = a.iter().map(|&x| x as i32).collect();
    let bi: Vec<i32> = b.iter().map(|&x| x as i32).collect();
    let ci: Vec<i32> = c.iter().map(|&x| x as i32).collect();
    let ctx = gpu_ctx();
    let raw = ctx.as_ref().expect("gpu context").dispatch_bytes(
        WGSL_INTEGER_KERNEL,
        bytemuck::cast_slice(&ai),
        bytemuck::cast_slice(&bi),
        bytemuck::cast_slice(&ci),
        std::mem::size_of::<i32>(),
    );
    bytemuck::cast_slice::<u8, i32>(&raw)
        .iter()
        .map(|&x| i64::from(x))
        .collect()
}

pub fn wgpu_elementwise_float_kernel(
    op_codes: &[i64],
    a: &[f64],
    b: &[f64],
    c: &[f64],
) -> Vec<f64> {
    assert_relu_mul_add_op_codes(op_codes);
    let af: Vec<f32> = a.iter().map(|&x| x as f32).collect();
    let bf: Vec<f32> = b.iter().map(|&x| x as f32).collect();
    let cf: Vec<f32> = c.iter().map(|&x| x as f32).collect();
    let ctx = gpu_ctx();
    let raw = ctx.as_ref().expect("gpu context").dispatch_bytes(
        WGSL_FLOAT_KERNEL,
        bytemuck::cast_slice(&af),
        bytemuck::cast_slice(&bf),
        bytemuck::cast_slice(&cf),
        std::mem::size_of::<f32>(),
    );
    bytemuck::cast_slice::<u8, f32>(&raw)
        .iter()
        .map(|&x| f64::from(x))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wgpu_adapter_initializes_with_fallback() {
        let info = wgpu_probe_adapter().expect("wgpu adapter must initialize");
        assert!(!info.is_empty());
    }

    #[test]
    fn wgpu_integer_relu_mul_add_matches_fixture() {
        let a = [2_i64, -1, 0, 5];
        let b = [3_i64, 4, 7, -2];
        let c = [1_i64, 10, -3, 8];
        let out = wgpu_elementwise_kernel(&RELU_MUL_ADD_OP_CODES, &a, &b, &c);
        assert_eq!(out, vec![7, 6, 0, 0]);
    }
}
