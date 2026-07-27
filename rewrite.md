**Build configuration**

- Set `-C target-cpu=x86-64-v3` in `.cargo/config.toml` for all platforms. This target contains SSE2 to AVX2, FMA, F16C, BMI1/2, LZCNT, and POPCNT.
- Do not use `target-cpu=native`. Do not set different CPU features on different platforms.
- Set one rustc version in `rust-toolchain.toml`. Use the same optimization flags for all builds. The same LLVM makes the same FP operation order on all OSes.
- Build C/C++ code for simulation math with the same fixed `-march`. Do not use `-ffast-math`. Pure Rust simulation code is best.

**Instructions that are permitted (same result on Intel and AMD)**

- All integer SIMD in SSE2 to AVX2: arithmetic, shifts, shuffles (`PSHUFB`, `PALIGNR`), blends, min/max, `PMADD*`, `PMULHRSW`, SAD, pack/unpack, permutes, gathers.
- SSE4.2 string instructions, `CRC32`, `POPCNT`.
- IEEE-754 float operations, scalar and packed, 128 and 256 bits: `ADD/SUB/MUL/DIV/SQRT`, `MIN/MAX` (NaN behavior is fixed by the spec), compares, `ADDSUB/HADD/HSUB` (lane order is fixed by the spec), all `CVT*` conversions.
- `ROUND*` (SSE4.1), only with an explicit rounding mode in the immediate. Do not use the MXCSR mode.
- F16C conversions (`VCVTPH2PS`, `VCVTPS2PH`). The rounding is fully specified.
- FMA (`VFMADD*`, `VFMSUB*`, `VFNMADD*`, `VFNMSUB*`). The result is correctly rounded per IEEE 754.
- `DPPS`/`DPPD`. The sum order is fixed by the SDM. But explicit mul + shuffle + add is easier to examine.

**Instructions and functions that are not permitted**

- `RCPPS/RCPSS/VRCPPS` and `RSQRTPS/RSQRTSS/VRSQRTPS`. These give approximate results that are different for each vendor and microarchitecture. Newton-Raphson steps do not correct this. Use full `DIV` and `SQRT`.
- All x87 instructions, and SSE3 `FISTTP`. Do not make 32-bit x86 builds.
- Changes to MXCSR: no FTZ/DAZ, no rounding-mode changes. Examine dependencies, specially audio/DSP crates (`_MM_SET_FLUSH_ZERO_MODE`). Default subnormal behavior is the same on all vendors.
- Runtime feature dispatch in simulation code: `is_x86_feature_detected!`, `multiversion`, simdeez runtime-dispatch paths. Use compile-time features only. Use one code path.
- Fast-math: `fadd_fast`, `fmul_fast`, algebraic intrinsics, and C flags that set them.
- std transcendental functions: `sin`, `cos`, `tan`, `exp`, `ln`, `powf`, `atan2`. These use the OS libm, and results are different on each platform. Use one fixed version of the pure-Rust `libm` crate, or your own fixed functions.
- These std functions are safe: `sqrt`, `abs`, `copysign`, `floor`, `ceil`, `round`, `trunc`, `mul_add`, `%` (fmod).

**Procedures**

- Write FMA explicitly with `mul_add`. Do not let the compiler contract `a * b + c`.
- Do FP reductions in a fixed order with an explicit shuffle/add tree. Do not use generic `reduce_sum` APIs.
- Keep the simulation on one thread, or use deterministic parallelism: fixed chunk boundaries, sequential math in each chunk, fixed merge order. Do not use work-stealing on FP state. Rayon/orx-parallel defaults are safe for the renderer only.
- Use deterministic containers and a fixed iteration order. Do not use the default `HashMap`. Keep a seeded RNG in the simulation state. Do not give clock or frame-time data to the simulation.
- Run the simulation on the CPU only. The GPU does the render only, and can be nondeterministic.
- Make a hash of the simulation state each frame. Hash the `to_bits()` patterns. Run cross-platform replay tests in CI to find desyncs after a dependency or toolchain change.

**Do not**

- Do not serialize floats as text in the simulation or network path. Use binary bit patterns only.
- Do not branch on the OS or platform in simulation code.
- Do not add physics or math middleware before you examine it for rsqrt approximations, fast-math builds, runtime dispatch, and MXCSR changes.
- Do not use NaN payloads in gameplay logic. This is deterministic on x86 only, and stops if you add ARM.

**Summary:** All machines run the same code path with the same IEEE-specified instructions. Thus determinism is: pin the toolchain, remove the four approximation instructions, control libm and threads. The replay-hash test in CI keeps this true.
