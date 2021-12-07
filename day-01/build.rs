use kaze::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let file = File::create(&dest_path).unwrap();

    let c = Context::new();

    sim::generate(Sweeper::new("sweeper", 32, &c).m, sim::GenerationOptions::default(), file)
}

#[allow(dead_code)]
struct Sweeper<'a> {
    pub m: &'a Module<'a>,

    pub depth: &'a Input<'a>,
    pub depth_valid: &'a Input<'a>,

    pub larger_measurements: &'a Output<'a>,
    pub larger_measurements_valid: &'a Output<'a>,
}

impl<'a> Sweeper<'a> {
    pub fn new(instance_name: impl Into<String>, depth_bit_width: u32, p: &'a impl ModuleParent<'a>) -> Sweeper<'a> {
        let m = p.module(instance_name, "Sweeper");

        let depth = m.input("depth", depth_bit_width);
        let depth_valid = m.input("depth_valid", 1);

        let state_valid = m.reg("state_valid", 1);
        state_valid.default_value(false);
        state_valid.drive_next(state_valid | depth_valid);

        let prev_depth = m.reg("prev_depth", depth_bit_width);
        prev_depth.drive_next(if_(depth_valid, {
            depth
        }).else_({
            prev_depth
        }));

        let larger_measurement = state_valid & depth_valid & depth.gt(prev_depth);

        let larger_measurements_bit_width = 32;
        let larger_measurements_reg = m.reg("larger_measurements_reg", larger_measurements_bit_width);
        larger_measurements_reg.default_value(0u32);
        larger_measurements_reg.drive_next(if_(larger_measurement, {
            larger_measurements_reg + m.lit(1u32, larger_measurements_bit_width)
        }).else_({
            larger_measurements_reg
        }));

        Sweeper {
            m,

            depth,
            depth_valid,

            larger_measurements: m.output("larger_measurements", larger_measurements_reg),
            larger_measurements_valid: m.output("larger_measurements_valid", state_valid),
        }
    }
}
