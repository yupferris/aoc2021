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

    sim::generate(Parser::new("number_line_parser", 32, &c).m, sim::GenerationOptions {
        tracing: true,
        ..Default::default()
    }, &mut file)?;
    sim::generate(Sweeper::new("sweeper", 32, &c).m, sim::GenerationOptions::default(), &mut file)?;
    sim::generate(Slider::new("slider", 32, &c).m, sim::GenerationOptions::default(), &mut file)?;

    Ok(())
}

#[allow(dead_code)]
struct Parser<'a> {
    pub m: &'a Module<'a>,

    pub ingress_ready: &'a Output<'a>,
    pub ingress_valid: &'a Input<'a>,
    pub ingress_data: &'a Input<'a>,

    pub egress_ready: &'a Input<'a>,
    pub egress_valid: &'a Output<'a>,
    pub egress_data: &'a Output<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(instance_name: impl Into<String>, number_bit_width: u32, p: &'a impl ModuleParent<'a>) -> Parser<'a> {
        let m = p.module(instance_name, "Parser");

        let state_bit_width = 2;
        let state_parsing = 0u32;
        let state_output_eof = 1u32;
        let state_finished = 2u32;
        let state = m.reg("state", state_bit_width);
        state.default_value(state_parsing);

        let egress_valid_reg = m.reg("egress_valid_reg", 1);
        egress_valid_reg.default_value(false);

        let egress_ready = m.input("egress_ready", 1);
        let backpressure = egress_valid_reg & !egress_ready;

        let ingress_ready = state.eq(m.lit(state_parsing, state_bit_width)) & !backpressure;

        let ingress_valid = m.input("ingress_valid", 1);
        let ingress_data = m.input("ingress_data", 8);

        let ingress_handshake = ingress_ready & ingress_valid;

        let ingress_eof = ingress_handshake & ingress_data.eq(m.lit(0u8, 8)); // TODO: Proper constant
        let ingress_eol = ingress_handshake & ingress_data.eq(m.lit('\n' as u8, 8));

        let acc = m.reg("acc", number_bit_width);
        acc.default_value(0u32);
        acc.drive_next(if_(ingress_handshake, {
            if_(ingress_eol, {
                m.lit(0u32, number_bit_width)
            }).else_({
                let digit = ingress_data - m.lit('0' as u8, 8);
                (acc * m.lit(10u32, number_bit_width)).bits(number_bit_width - 1, 0) + m.lit(0u32, number_bit_width - 8).concat(digit)
            })
        }).else_({
            acc
        }));

        state.drive_next(if_(state.eq(m.lit(state_parsing, state_bit_width)) & ingress_eof, {
            m.lit(state_output_eof, state_bit_width)
        }).else_if(state.eq(m.lit(state_output_eof, state_bit_width)) & egress_ready, {
            m.lit(state_finished, state_bit_width)
        }).else_({
            state
        }));

        egress_valid_reg.drive_next(if_(ingress_eof | ingress_eol, {
            m.lit(true, 1)
        }).else_if(egress_valid_reg & egress_ready, {
            m.lit(false, 1)
        }).else_({
            egress_valid_reg
        }));
        let egress_data_reg = m.reg("egress_data_reg", number_bit_width);
        egress_data_reg.drive_next(if_(ingress_eof, {
            m.lit(true, 1).repeat(number_bit_width) // TODO: Proper constant
        }).else_if(ingress_eol, {
            acc
        }).else_({
            egress_data_reg
        }));

        let egress_valid = m.output("egress_valid", egress_valid_reg);
        let egress_data = m.output("egress_data", egress_data_reg);

        Parser {
            m,

            ingress_ready: m.output("ingress_ready", ingress_ready),
            ingress_valid,
            ingress_data,

            egress_ready,
            egress_valid,
            egress_data,
        }
    }
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
