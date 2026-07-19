# cribler

Batteries-included statistical randomness-test toolkit for Rust.

`cribler` re-exports every engine from
[`cribler_core`](https://crates.io/crates/cribler_core) (the
dependency-minimal, closure-based foundation) and adds ergonomic named-case
batching (`XxxSuite`), a whole-battery aggregator (`Suite`), and optional
`rand_core`/`urng` integration. `Suite::run`/`XxxSuite::run` return
[`hashbrown::HashMap`](https://crates.io/crates/hashbrown) rather than
`std::collections::HashMap`.

No RNG library is a hard dependency by default: every engine takes a plain
`FnMut() -> f64` / `FnMut() -> u64` sampler closure, so any generator — a
hand-rolled PRNG, [`urng`](https://crates.io/crates/urng), or anything
exposing `rand_core::Rng` — plugs in directly without enabling any feature.
Enable the `rand` or `urng` feature for pre-built typed convenience on top of
that.

## Engines

| Engine                        | What it detects                                                                           |
| ----------------------------- | ----------------------------------------------------------------------------------------- |
| Chi-squared (`ChiSq`)         | Uniformity of the output distribution                                                     |
| Monte Carlo pi (`McPi`)       | 2D uniformity sanity check via pi estimation                                              |
| Serial (`Serial`)             | Correlation between consecutive outputs (multi-dimensional chi-squared)                   |
| Runs (`Runs`)                 | Sequential correlation via monotonic run lengths                                          |
| Kolmogorov-Smirnov (`Ks`)     | Deviation of the empirical CDF from the ideal uniform CDF                                 |
| Birthday spacing (`Birthday`) | Short periods / lattice structure (Marsaglia's DIEHARD test)                              |
| NIST SP 800-22 (`Nist`)       | Frequency, Block Frequency, Runs, Longest-Run-of-Ones, Cumulative Sums (bit-level subset) |
| Paranoid (`Paranoid`)         | Elevates any single test into a battery-level test (NIST SP 800-22 §4.2)                  |

## Quick start

```rust
use cribler::{ChiSqConfig, run_chisq};

// Any FnMut() -> f64 works; this one is a tiny inline xorshift32.
let mut state: u32 = 1;
let mut sampler = || {
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    state as f64 / 4294967296.0
};

let result = run_chisq("xorshift32", &mut sampler, ChiSqConfig::default()).unwrap();
println!("{result:?}");
```

## Running multiple named cases (`XxxSuite`)

```rust
use cribler::ChiSqSuite;

let mut state1: u32 = 1;
let mut state2: u32 = 2;
let mut next = |state: &mut u32| -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    *state as f64 / 4294967296.0
};

let mut suite = ChiSqSuite::new(0);
suite.add_sampler("generator-a", || next(&mut state1)).unwrap();
suite.add_sampler("generator-b", || next(&mut state2)).unwrap();

for (name, result) in suite.run().unwrap() {
    println!("{name}: {:?}", result.verdict);
}
```

## Running the whole battery at once (`Suite`)

[`Suite`] wraps one `ChiSqSuite`/`McPiSuite`/`SerialSuite`/`RunsSuite`/
`KsSuite`/`BirthdaySuite`/`NistSuite` each, all sharing the same seed, so
registering a generator once runs every engine against it. `Suite::run`
returns one [`SuiteResult`] (all seven engines' results) per registered
generator, keyed by its case name (`HashMap<String, SuiteResult>`):

```rust
use cribler::{Suite, WordSource};

#[derive(Clone)]
struct MyRng(u64);

impl<'a> From<MyRng> for WordSource<'a> {
    fn from(mut rng: MyRng) -> Self {
        WordSource::new("MyRng", 64, move || {
            rng.0 = rng.0.wrapping_mul(6364136223846793005).wrapping_add(1);
            rng.0
        })
    }
}

let results = Suite::new(1).from_custom(MyRng(1)).unwrap().run().unwrap();
let result = &results["MyRng"];
println!("{:?}", result.chisq.verdict);
println!("{:?}", result.nist.verdict);
```

`Suite` also has `from_urng32`/`from_urng64` (feature `urng`) and `from_rand`
(feature `rand`), mirroring `XxxSuite` — the only difference is `from_custom`
requires `S: Clone`, since each engine needs its own independent generator
instance.

## Elevating a test to battery level (`Paranoid`)

A single p-value can pass by chance even for a bad generator. `Paranoid`
re-runs a test many times over independent samples and checks both the
pass proportion and the uniformity of the resulting p-values
(NIST SP 800-22 §4.2):

```rust
use cribler::{ChiSqConfig, ParanoidConfig, p_value_from_z, run_chisq, run_paranoid};

let mut trial = |i: usize| {
    let mut state = (i as u32 + 1).wrapping_mul(2654435761) | 1;
    let mut sampler = || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state as f64 / 4294967296.0
    };
    let z = run_chisq(format!("trial-{i}"), &mut sampler, ChiSqConfig::default())
        .unwrap()
        .z_score;
    p_value_from_z(z)
};

let result = run_paranoid("xorshift32.paranoid", ParanoidConfig::default(), &mut trial).unwrap();
println!("{:?}", result.verdict);
```

## `Suite::from_urng32`/`from_urng64`/`from_rand`/`from_custom`

Every `XxxSuite` is created with a seed — `ChiSqSuite::new(seed)` — and has
methods that register a named test case from a generator seeded from it:

- `from_urng32::<R>()` / `from_urng64::<R>()` — a `urng::Rng32`/`Rng64` `R`, constructed from the suite's seed via [`UrngSeed32`]/[`UrngSeed64`] (feature `urng`). No instance to pass in.
- `from_rand::<R>()` — a `rand_core::Rng` `R`, constructed via `SeedableRng::seed_from_u64` from the suite's seed (feature `rand`). No instance to pass in.
- `from_custom(source)` — anything else. Custom generators are always integer-based, never float-based: implement `From<YourType> for WordSource` directly and pass an instance through (the suite's seed isn't involved; see the [`source`] module docs for a full example). On the `[0, 1)`-sampling engines, the raw `u64` draws are converted with `unit_f64_from_u32`/`unit_f64_from_u64` depending on `word_bits`.

The case name is taken from the generator's type name (`R`'s name for
`from_urng32`/`from_urng64`/`from_rand`, or whatever you pass to
`WordSource::new` for `from_custom`).

### `rand` feature

```toml
[dependencies]
cribler = { version = "0.1", features = ["rand"] }
```

```rust,ignore
use cribler::ChiSqSuite;

let results = ChiSqSuite::new(0)
    .from_rand::<rand::rngs::StdRng>()
    .unwrap()
    .run()
    .unwrap();
```

### `urng` feature

```toml
[dependencies]
cribler = { version = "0.1", features = ["urng"] }
```

```rust
use cribler::ChiSqSuite;
use urng::Sfc32;

let results = ChiSqSuite::new(0)
    .from_urng32::<Sfc32>()
    .unwrap()
    .run()
    .unwrap();
println!("{:?}", results["Sfc32"].verdict);
```

Chain multiple generator types onto the same `Suite` before calling `run()`
to batch them under a shared config (each type contributes one named case,
so generators of the same type need telling apart some other way — e.g. by
going through `from_custom` with an explicit name instead):

```rust
use cribler::ChiSqSuite;
use urng::{Sfc32, Xorshift32};

let results = ChiSqSuite::new(0)
    .from_urng32::<Sfc32>().unwrap()
    .from_urng32::<Xorshift32>().unwrap()
    .run()
    .unwrap();
assert_eq!(results.len(), 2);
```

No feature flag is required at all if you'd rather skip the typed bridges:
every engine also takes a plain closure, so
`|| cribler_core::unit_f64_from_u32(rng.nextu())` works without any extra
dependency.

Note: `cribler` depends on `urng` here (not the other way around) — `urng`
itself has no dependency on `cribler`, which is what makes this feature
possible without a cyclic package dependency.

## `serde` feature

Enable `serde` to make every `Config`/`Verdict`/`Result` type (from both
`cribler_core` and `cribler`, including `SuiteResult`) `Serialize`/
`Deserialize`:

```toml
[dependencies]
cribler = { version = "0.1", features = ["serde", "urng"] }
```

```rust
use cribler::ChiSqSuite;
use urng::Sfc32;

let results = ChiSqSuite::new(0).from_urng32::<Sfc32>().unwrap().run().unwrap();
let json = serde_json::to_string(&results).unwrap();
let round_tripped: hashbrown::HashMap<String, cribler::ChiSqResult> =
    serde_json::from_str(&json).unwrap();
assert_eq!(results["Sfc32"].verdict, round_tripped["Sfc32"].verdict);
```

## Crate layout

- [`cribler_core`](https://crates.io/crates/cribler_core): the engines
  themselves, with no dependency on any RNG crate. Use this directly for the
  smallest possible dependency footprint.
- `cribler` (this crate): `cribler_core` plus `Suite` batching and optional
  `rand_core` integration.

## License

MIT
