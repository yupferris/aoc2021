use kaze::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let mut file = File::create(&dest_path).unwrap();

    let c = Context::new();

    sim::generate(Sweeper::new("sweeper", 32, &c).m, sim::GenerationOptions::default(), &mut file)?;
    sim::generate(Slider::new("slider", 32, &c).m, sim::GenerationOptions::default(), &mut file)?;

    Ok(())
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

#[allow(dead_code)]
struct Slider<'a> {
    pub m: &'a Module<'a>,

    pub depth: &'a Input<'a>,
    pub depth_valid: &'a Input<'a>,

    pub larger_measurements: &'a Output<'a>,
    pub larger_measurements_valid: &'a Output<'a>,
}

impl<'a> Slider<'a> {
    pub fn new(instance_name: impl Into<String>, depth_bit_width: u32, p: &'a impl ModuleParent<'a>) -> Slider<'a> {
        let m = p.module(instance_name, "Slider");

        let depth = m.input("depth", depth_bit_width);
        let depth_valid = m.input("depth_valid", 1);

        let mut prev_depths: Vec<&dyn Signal<'_>> = Vec::new();
        for i in 0..2 {
            let prev_depth = m.reg(format!("prev_depth_{}", i), depth_bit_width);
            prev_depth.drive_next(if_(depth_valid, {
                if i == 0 {
                    depth
                } else {
                    prev_depths[i - 1]
                }
            }).else_({
                prev_depth
            }));
            prev_depths.push(prev_depth);
        }

        let mut state_valids = Vec::new();
        for i in 0..3 {
            let state_valid = m.reg(format!("state_valid_{}", i), 1);
            state_valid.default_value(false);
            state_valid.drive_next(if_(depth_valid, {
                if i == 0 {
                    m.lit(true, 1)
                } else {
                    state_valids[i - 1]
                }
            }).else_({
                state_valid
            }));
            state_valids.push(state_valid);
        }

        let state_valid = *state_valids.last().unwrap();

        let mut sum: &dyn Signal<'_> = depth;
        for &prev_depth in &prev_depths {
            sum = sum + prev_depth;
        }

        let prev_sum = m.reg("prev_sum", depth_bit_width);
        prev_sum.default_value(0u32);
        prev_sum.drive_next(if_(depth_valid, {
            sum
        }).else_({
            prev_sum
        }));

        let larger_measurement = state_valid & depth_valid & sum.gt(prev_sum);

        let larger_measurements_bit_width = 32;
        let larger_measurements_reg = m.reg("larger_measurements_reg", larger_measurements_bit_width);
        larger_measurements_reg.default_value(0u32);
        larger_measurements_reg.drive_next(if_(larger_measurement, {
            larger_measurements_reg + m.lit(1u32, larger_measurements_bit_width)
        }).else_({
            larger_measurements_reg
        }));

        Slider {
            m,

            depth,
            depth_valid,

            larger_measurements: m.output("larger_measurements", larger_measurements_reg),
            larger_measurements_valid: m.output("larger_measurements_valid", state_valid),
        }
    }
}
