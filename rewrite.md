# Determinism rules for this library

This library must give the same bits on every machine. This document gives the
rules that make this true. The rules apply to two targets only: x86-64 and ARM.

**Build configuration**

- Set `-C target-cpu=x86-64-v3` for all x86-64 targets. This level contains SSE2
  to AVX2, FMA, F16C, BMI1, BMI2, LZCNT, and POPCNT.
- Set `-C target-cpu=generic` for all ARM targets. This is the ARMv8-A baseline.
  It contains NEON, IEEE-754 floating point, FMLA, and the FRINT instructions.
- Do not use `target-cpu=native`. Do not set different processor features on
  different machines. If you raise the ARM baseline, raise it everywhere at the
  same time.
- Set one rustc version in `rust-toolchain.toml`. Use the same optimization
  flags for all builds. The same LLVM makes the same floating-point operation
  order on all operating systems.
- Write the code in Rust. If you add C or C++ code, build it with the same fixed
  `-march` flag. Do not use `-ffast-math`.

**Instructions that are permitted on x86-64**

- All integer SIMD from SSE2 to AVX2: arithmetic, shifts, shuffles (`PSHUFB`,
  `PALIGNR`), blends, min and max, `PMADD*`, `PMULHRSW`, SAD, pack, unpack,
  permutes, and gathers.
- SSE4.2 string instructions, `CRC32`, and `POPCNT`.
- IEEE-754 floating-point operations, scalar and packed, 128 bits and 256 bits:
  `ADD`, `SUB`, `MUL`, `DIV`, `SQRT`, compares, and all `CVT*` conversions. IEEE
  754 gives the result of each one exactly.
- `MINPS`, `MAXPS`, `MINPD`, and `MAXPD`. These give `a < b ? a : b`, and this
  is the rule this library uses on ARM also.
- `ROUNDPS` and `ROUNDPD` (SSE4.1), with a rounding mode in the immediate. Do
  not use the MXCSR mode.
- F16C conversions (`VCVTPH2PS` and `VCVTPS2PH`). The rounding is fully defined.
- FMA (`VFMADD*`, `VFMSUB*`, `VFNMADD*`, `VFNMSUB*`). IEEE 754 gives the result
  exactly, thus these agree with the ARM `FMLA` and `FMLS` instructions.

**Instructions that are permitted on ARM**

- All integer NEON operations: arithmetic, shifts, table lookups (`TBL`, `TBX`),
  extracts (`EXT`), bitwise select (`BSL`), min and max, and pairwise operations
  with an order that you write out.
- IEEE-754 floating-point operations, scalar and packed, 128 bits: `FADD`,
  `FSUB`, `FMUL`, `FDIV`, `FSQRT`, compares, and all `FCVT*` conversions.
- `FMLA` and `FMLS`. These agree with the x86-64 FMA instructions.
- `FRINTN` and `FRINTZ`. `FRINTN` agrees with `ROUNDPS` immediate 0.
  `FRINTZ` agrees with `ROUNDPS` immediate 3.
- `FRINTM` and `FRINTP`. These agree with `ROUNDPS` immediate 1 and immediate 2.
- Half-precision conversions (`FCVTL`, `FCVTN`). The rounding is fully defined.
- Compares (`FCMGT`, `FCMGE`, `FCMEQ`) with `BSL`, to build min and max.

**Instructions and functions that are not permitted**

On x86-64:

- `RCPPS`, `RCPSS`, `VRCPPS`, `RSQRTPS`, `RSQRTSS`, and `VRSQRTPS`. These give
  results that are not exact, and the results are different on different
  processors. Newton-Raphson steps do not correct this. Use `DIV` and `SQRT`.
- `DPPS` and `DPPD`. ARM has no instruction with the same sum order. Write out
  the multiply, shuffle, and add steps.
- All x87 instructions, and SSE3 `FISTTP`. Do not build for 32-bit x86.
- Changes to MXCSR: no FTZ, no DAZ, and no changes to the rounding mode.

On ARM:

- `FRECPE`, `FRECPS`, `FRSQRTE`, and `FRSQRTS`, and the `vrecpe*` and `vrsqrte*`
  functions. These give results that are not exact. Use `FDIV` and `FSQRT`.
- `FADDV` and the `vaddvq_*` functions. The sum order is not the same as the
  x86-64 order. Write out the add tree.
- `FMIN`, `FMAX`, `FMINNM`, `FMAXNM`, and the `vminq_*`, `vmaxq_*`, `vminnmq_*`,
  and `vmaxnmq_*` functions. These handle NaN and negative zero differently from
  `MINPS` and `MAXPS`. Use compares with `BSL`.
- `FRINTA`. x86-64 has no round instruction that agrees with it. Use `FRINTN`.
- Changes to FPCR: no FZ, no AH, and no changes to the rounding mode.

On all targets:

- Processor feature tests while the program runs: `is_x86_feature_detected!`,
  `is_aarch64_feature_detected!`, the `multiversion` crate, and the runtime
  paths in `simdeez`. Use compile-time features. Use one code path.
- Fast math: `fadd_fast`, `fmul_fast`, the algebraic functions (`algebraic_add`,
  `algebraic_mul`), and the C flags that turn them on.
- These std functions: `sin`, `cos`, `tan`, `exp`, `ln`, `log2`, `powf`, and
  `atan2`. They call the libm of the operating system, and each platform gives
  different results. Use one fixed version of the pure-Rust `libm` crate, or a
  fixed function of your own.
- A fixed function of your own is permitted where speed matters, if every step
  is a multiply, an add, a compare or a select. `sin_vec4` in the quaternion
  code is one: it computes four sines at once for `slerp`, and all three
  backends give the same bits. Such a function gives a result that is not
  exact, but it gives the *same* result everywhere, which is the rule.
- These std functions are permitted: `sqrt`, `abs`, `copysign`, `floor`, `ceil`,
  `trunc`, `mul_add`, and `%`.

**Rules for the code**

- Write FMA with `mul_add`. Do not let the compiler join `a * b + c` into one
  instruction.
- Put the FMA operations at the same positions on x86-64 and on ARM.
- Do floating-point sums in an order that you write out. Use a fixed shuffle and
  add tree. Do not use `reduce_sum` or a function that does the same.
- Use the same add tree on x86-64 and on ARM.
- Make min and max give `a < b ? a : b` on all targets.
- Round to nearest, with ties to even. `FRINTN` and `ROUNDPS` immediate 0 do
  this, thus no target needs extra code. Do not use `f32::round`, because it
  moves ties away from zero and no x86-64 instruction does this.
- Do not branch on the operating system. Branch on the processor type only to
  select the SSE2 backend or the NEON backend. The two backends must give the
  same bits.
- Examine each new dependency before you add it. Look for reciprocal
  instructions that are not exact, fast-math builds, processor feature tests,
  and changes to MXCSR or FPCR.
- Do not write floating-point values as text. Use the bit patterns.
- Do not put data in the payload of a NaN.

**Rules for integer vectors**

- Integer arithmetic is exact, thus a SIMD body and a scalar body give the same
  bits. The risk is not rounding. The risk is that the two processors define an
  edge case differently.
- Do not put `Shl`, `Shr`, `Div` or `Rem` in a SIMD body. For value -8 and count
  -1, x86-64 `VPSLLVD` gives 0 and ARM `SSHL` gives -4, and scalar Rust masks the
  count and gives neither. Neither processor has a packed integer divide.
- Do not use a saturating instruction for an operation that wraps. `+`, `-`, `*`,
  negate and `abs` wrap in release. `VQADD`, `VQSUB`, `VQABS`, `VQNEG`, `PADDS`
  and `PSUBS` saturate. `saturating_add` and the other saturating methods are
  `const fn` and stay scalar, thus a SIMD body never needs these instructions.
  `tests/forbidden.rs` bans them.
- A SIMD body for an operation that can overflow is for release only. Rust panics
  on overflow in debug and wraps in release, and a SIMD lane always wraps. Gate
  such a body on `not(debug_assertions)`. The correct gate is `overflow_checks`,
  but it is unstable on the pinned compiler. Thus a release profile that sets
  `overflow-checks = true` gets the wrapping body and no panic.
- Give the 8-bit and 16-bit vectors plain `#[repr(C)]` fields. Do not give them a
  `__m128i` or `int16x4_t` field. The size and the alignment must not change,
  because `encase`, `bytemuck`, `zerocopy` and `rkyv` read them.
- Measure before you write a SIMD body, and measure again after. A hand-written
  body works on one vector at a time and can stop the compiler from working on
  many. This made the 8-bit add 64 percent slower, and that body was removed.
- Keep `saturating_*`, `wrapping_*` and `checked_*` as `const fn`. The `const`
  keyword is not a cost here. The compiler vectorizes these across the elements
  of a loop and uses the full register, thus `I16Vec4::saturating_add` measures
  1.30 microseconds for 8192 vectors while the hand-written SIMD `+` measures
  2.90. A SIMD body for these would be about two times SLOWER.
- The trait boundary, not the arithmetic, is the cost of `+`, `-` and `*` on the
  narrow types. The same field arithmetic in a closure measures 0.98
  microseconds, and behind `impl Add` it measures 4.60. `#[inline(always)]` does
  not change this. The 8-byte struct becomes one 64-bit register at the function
  boundary, and the SWAR happens before the caller can vectorize. In a hot loop,
  prefer `a.wrapping_add(b)` to `a + b`.

**Tests**

- Keep the scalar backend. Use it as the reference for tests. Do not ship it.
- Write a test that sends the same data through the scalar backend, the SSE2
  backend, and the NEON backend. The test compares the bits.
- Store known bit patterns in tests. These tests fail if a new toolchain changes
  a result.
- Run all tests on x86-64 and on ARM in CI.
- Add a check that fails if the code contains an instruction from the list of
  instructions that are not permitted.

**Summary**

All machines run the same code path with the same IEEE-defined instructions.
Thus determinism has four parts: pin the toolchain, remove the instructions that
are not exact, control libm, and keep the operation order the same on both
processor types. The tests in CI keep this true.
