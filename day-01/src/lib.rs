#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    const INPUT: &'static str = include_str!("input.txt");

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
