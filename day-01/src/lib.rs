#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    use kaze::runtime::tracing::*;
    use kaze::runtime::tracing::vcd::*;

    use std::env;
    use std::fs::File;
    use std::io;

    const INPUT: &'static str = include_str!("input.txt");

    fn build_trace(test_name: &'static str) -> io::Result<impl Trace> {
        let mut path = env::temp_dir();
        path.push(format!("{}.vcd", test_name));
        println!("Writing trace to {:?}", path);
        let file = File::create(path)?;
        VcdTrace::new(file, 10, TimeScaleUnit::Ns)
    }

    #[test]
    fn parser() -> io::Result<()> {
        let expected_output = INPUT.lines().map(|line| {
            str::parse::<u32>(line).expect("Couldn't parse depth")
        }).collect::<Vec<_>>();

        let input = {
            let mut ret = INPUT.as_bytes().iter().cloned().collect::<Vec<_>>();
            // Null terminator
            ret.push(0x00); // TODO: Share constant with RTL
            ret
        };
        let mut output = Vec::new();

        let mut input_index = 0;

        let trace = build_trace("parser")?;

        let mut m = Parser::new(trace)?;
        let mut time_stamp = 0;

        m.reset();

        loop {
            m.prop();

            // Input
            if input_index < input.len() {
                m.ingress_data = input[input_index] as _;
                m.ingress_valid = true;

                if m.ingress_valid && m.ingress_ready {
                    input_index += 1;
                }
            } else {
                m.ingress_valid = false;
            };

            // Output
            m.egress_ready = true;

            m.prop();
            m.update_trace(time_stamp)?;

            if m.egress_ready && m.egress_valid {
                // TODO: Share constant with RTL
                if m.egress_data == 0xffffffff {
                    break;
                } else {
                    output.push(m.egress_data);
                }
            }

            m.posedge_clk();
            time_stamp += 1;
        }

        assert_eq!(expected_output, output);

        Ok(())
    }

    #[test]
    fn part_1() {
        let mut m = Sweeper::new();

        m.reset();

        for line in INPUT.lines() {
            let depth = str::parse::<u32>(line).expect("Couldn't parse depth");
            m.depth = depth;
            m.depth_valid = true;
            m.prop();
            m.posedge_clk();
        }

        m.prop();
        assert!(m.larger_measurements_valid);
        assert_eq!(1532, m.larger_measurements);
    }

    #[test]
    fn part_2() {
        let mut m = Slider::new();

        m.reset();

        for line in INPUT.lines() {
            let depth = str::parse::<u32>(line).expect("Couldn't parse depth");
            m.depth = depth;
            m.depth_valid = true;
            m.prop();
            m.posedge_clk();
        }

        m.prop();
        assert!(m.larger_measurements_valid);
        assert_eq!(1571, m.larger_measurements);
    }
}
