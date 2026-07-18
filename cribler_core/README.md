# cribler_core

Closure-based, RNG-library-agnostic statistical randomness-test engines.

Every engine is a free function `run_xxx(name, sampler, config)` that takes a
plain `FnMut() -> T` sampler (`T` is `f64` for the `[0, 1)`-float engines,
`u64` for the word-level engines) and a `Config`, and returns a `Result`
carrying a `Verdict`. There is **no dependency on any RNG crate** — wrap
whatever generator you have in a closure and call the engine directly.

This is the low-level, dependency-minimal foundation (mirroring the
`rand_core`/`rand` split). The companion [`cribler`](https://crates.io/crates/cribler)
crate builds ergonomic named-case batching (`XxxSuite`) and optional
`rand_core` integration on top of it. Use `cribler_core` directly if you want
the engines with the smallest possible dependency footprint, or `cribler` if
you want the batteries-included experience.

## Engines

| Engine | What it detects |
|---|---|
| `chisq` | Chi-squared uniformity test |
| `mcpi` | Monte Carlo estimation of pi (sanity check on 2D uniformity) |
| `serial` | Multi-dimensional chi-squared test over tuples of consecutive outputs (correlation) |
| `runs` | Sequential correlation via monotonic ascending/descending run lengths |
| `ks` | Kolmogorov-Smirnov test against the ideal uniform CDF |
| `birthday` | Marsaglia's DIEHARD birthday-spacings test (short periods, lattice structure) |
| `nist` | NIST SP 800-22 bit-level test suite (Frequency, Block Frequency, Runs, Longest-Run-of-Ones, Cumulative Sums) |
| `paranoid` | Elevates any single test into a battery-level test (NIST SP 800-22 §4.2: pass-proportion + p-value-uniformity checks) |

## Example

```rust
use cribler_core::{ChiSqConfig, run_chisq};

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

For the word-level engines (`birthday`, `nist`), the sampler returns raw
`u64` words instead:

```rust
use cribler_core::{NistConfig, run_nist};

let mut state: u32 = 1;
let mut word_sampler = || {
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    state as u64
};

// word_bits = 32 because the sampler above only produces 32 bits of entropy.
let result = run_nist("xorshift32", &mut word_sampler, 32, NistConfig::default()).unwrap();
println!("{:?}", result.verdict);
```

## Using with an RNG library

No adapter code is needed — just call the generator's output method inside
the closure:

```rust,ignore
// urng
let mut rng = urng::Sfc32::new(0);
let mut sampler = || cribler_core::unit_f64_from_u32(rng.nextu());

// rand (rand_core::Rng)
let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
let mut sampler = || cribler_core::unit_f64_from_u64(rng.next_u64());
```

## License

MIT
