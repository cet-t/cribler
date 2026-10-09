#[derive(Clone, Copy)]
struct Tiny64(u64);

impl<'a> From<Tiny64> for cribler::WordSource<'a> {
    fn from(mut rng: Tiny64) -> Self {
        cribler::WordSource::new("Tiny64", 64, move || {
            rng.0 = rng.0.wrapping_mul(6364136223846793005).wrapping_add(1);
            rng.0
        })
    }
}

fn main() -> anyhow::Result<()> {
    let results = cribler::Suite::new(1)
        .from_custom(Tiny64(1))?
        .from_urng32::<urng::Sfc32>()?
        .from_rand::<rand_sfc::Sfc32>()?
        .from_urng64::<urng::Sfc64>()?
        .from_rand::<rand_sfc::Sfc64>()?
        .from_urng32::<urng::Philox32>()?
        .from_urng_core(urng_splitmix::SplitMix32::new)?
        .from_urng_core(urng_splitmix::SplitMix64::new)?
        .run()?;

    for r in &results {
        println!("{}", serde_json::to_string_pretty(&r)?);
    }

    Ok(())
}
