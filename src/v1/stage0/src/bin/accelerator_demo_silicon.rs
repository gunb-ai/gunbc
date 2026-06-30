//! Operator-runnable real-silicon path for accelerator-demo GPU epilogue.
//! DORMANT until the operator runs on M5 (Metal) or RTX 5090 (DX12/Vulkan).
//!
//! Usage:
//!   GUNBC_WGPU_FORCE_HARDWARE=1 GUNBC_WGPU_BACKENDS=metal cargo run --bin accelerator_demo_silicon -- --device apple-m5-metal
//!   GUNBC_WGPU_FORCE_HARDWARE=1 GUNBC_WGPU_BACKENDS=dx12 cargo run --bin accelerator_demo_silicon -- --device nvidia-rtx5090-dx12-vulkan

use clap::Parser;
use serde::Serialize;
use v1_compiler::v1_interpreter::v1_rt_gpu;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "apple-m5-metal")]
    device: String,
}

#[derive(Serialize)]
struct FidelityReceipt {
    device_label: String,
    backend_label: String,
    adapter_info: String,
    integer_bit_exact: bool,
    integer_output: Vec<i64>,
    float_declared_fidelity: &'static str,
    float_declaration_truthful: bool,
    float_oracle: Vec<f64>,
    float_gpu: Vec<f64>,
    wgsl_integer_storage: &'static str,
    wgsl_float_storage: &'static str,
    oracle_float_storage: &'static str,
    status: &'static str,
}

fn main() {
    let args = Args::parse();
    let adapter = v1_rt_gpu::wgpu_probe_adapter().expect("wgpu adapter required on real silicon");
    let op_codes = [1_i64, 2, 3];
    let a = [2_i64, -1, 0, 5];
    let b = [3_i64, 4, 7, -2];
    let c = [1_i64, 10, -3, 8];
    let int_gpu = v1_rt_gpu::wgpu_elementwise_kernel(&op_codes, &a, &b, &c);
    let int_expected = [7_i64, 6, 0, 0];
    let integer_bit_exact = int_gpu == int_expected;

    let fa = [1.0000000596046446_f64];
    let fb = [1.0000000596046446_f64];
    let fc = [-1.0_f64];
    let float_gpu = v1_rt_gpu::wgpu_elementwise_float_kernel(&op_codes, &fa, &fb, &fc);
    let float_oracle = {
        let tmp = fa[0] * fb[0] + fc[0];
        vec![if tmp > 0.0 { tmp } else { 0.0 }]
    };
    let float_differs = float_gpu != float_oracle;
    let receipt = FidelityReceipt {
        device_label: args.device,
        backend_label: std::env::var("GUNBC_WGPU_BACKENDS").unwrap_or_else(|_| "auto".into()),
        adapter_info: adapter,
        integer_bit_exact,
        integer_output: int_gpu,
        float_declared_fidelity: "Lossy",
        float_declaration_truthful: float_differs,
        float_oracle,
        float_gpu,
        wgsl_integer_storage: "i32",
        wgsl_float_storage: "f32",
        oracle_float_storage: "Float64",
        status: "executed",
    };
    println!("{}", serde_json::to_string_pretty(&receipt).expect("receipt json"));
}
