#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    #[test]
    fn part_1() {
        let input = include_str!("input.txt");

        let mut m = Sweeper::new();

        m.reset();

        for line in input.lines() {
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
}
