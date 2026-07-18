use cribler::{
    BirthdaySuite, ChiSqSuite, KsSuite, McPiSuite, NistSuite, ParanoidConfig, ParanoidSuite,
    RunsSuite, SerialSuite, p_value_from_z,
};

struct Xorshift32(u32);
impl Xorshift32 {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    fn next_u64(&mut self) -> u64 {
        (self.next_u32() as u64) << 32 | self.next_u32() as u64
    }
    fn unit_f64(&mut self) -> f64 {
        self.next_u32() as f64 * (1.0 / 4294967296.0)
    }
}

#[test]
fn f64_suites_run_over_named_cases() {
    let mut rng1 = Xorshift32(0x2545F491);
    let mut rng2 = Xorshift32(0x2545F492);
    let mut rng3 = Xorshift32(0x2545F493);
    let mut rng4 = Xorshift32(0x2545F494);
    let mut rng5 = Xorshift32(0x2545F495);

    let mut chi = ChiSqSuite::new(0);
    chi.add_sampler("xorshift32", || rng1.unit_f64()).unwrap();
    assert_eq!(chi.run().unwrap().len(), 1);

    let mut mcpi = McPiSuite::new(0);
    mcpi.add_sampler("xorshift32", || rng2.unit_f64()).unwrap();
    assert_eq!(mcpi.run().unwrap().len(), 1);

    let mut serial = SerialSuite::new(0);
    serial
        .add_sampler("xorshift32", || rng3.unit_f64())
        .unwrap();
    assert_eq!(serial.run().unwrap().len(), 1);

    let mut runs = RunsSuite::new(0);
    runs.add_sampler("xorshift32", || rng4.unit_f64()).unwrap();
    assert_eq!(runs.run().unwrap().len(), 1);

    let mut ks = KsSuite::new(0);
    ks.add_sampler("xorshift32", || rng5.unit_f64()).unwrap();
    assert_eq!(ks.run().unwrap().len(), 1);
}

#[test]
fn word_suites_run_over_named_cases() {
    let mut rng1 = Xorshift32(12345);
    let mut rng2 = Xorshift32(54321);

    let mut birthday = BirthdaySuite::new(0);
    birthday
        .add_sampler("xorshift32", || rng1.next_u64())
        .unwrap();
    assert_eq!(birthday.run().unwrap().len(), 1);

    let mut nist = NistSuite::new(0);
    nist.add_sampler("xorshift32", 32, || rng2.next_u64())
        .unwrap();
    assert_eq!(nist.run().unwrap().len(), 1);
}

#[test]
fn duplicate_case_name_is_rejected() {
    let mut rng1 = Xorshift32(7);
    let mut rng2 = Xorshift32(8);
    let mut chi = ChiSqSuite::new(0);
    chi.add_sampler("dup", || rng1.unit_f64()).unwrap();
    chi.add_sampler("dup", || rng2.unit_f64()).unwrap();
    assert!(chi.run().is_err());
}

#[test]
fn paranoid_suite_wraps_chisq_trials() {
    let config = ParanoidConfig {
        trials: 20,
        ..ParanoidConfig::default()
    };
    let mut suite = ParanoidSuite::with_config(config).unwrap();
    suite
        .add_trial("xorshift32.paranoid", |i| {
            let mut rng = Xorshift32((i as u32 + 1).wrapping_mul(2654435761) | 1);
            let mut sampler = || rng.unit_f64();
            let z = cribler::run_chisq(
                format!("trial-{i}"),
                &mut sampler,
                cribler::ChiSqConfig::default(),
            )
            .unwrap()
            .z_score;
            p_value_from_z(z)
        })
        .unwrap();
    let results = suite.run().unwrap();
    assert_eq!(results.len(), 1);
}
