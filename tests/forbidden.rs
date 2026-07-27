//! Fails if the source contains an instruction or function that `rewrite.md`
//! does not permit.
//!
//! This runs with `cargo test`, so it catches a bad edit on the machine that
//! made it, not later in CI. Comments are stripped before the scan, so a rule
//! may name the thing it bans.

use std::fs;
use std::path::{Path, PathBuf};

/// `(pattern, why it is not permitted)`
const FORBIDDEN: &[(&str, &str)] = &[
    // Approximate reciprocals. The result differs between processors, and
    // Newton-Raphson steps do not correct it. Use DIV and SQRT.
    ("_mm_rcp_ps", "approximate reciprocal"),
    ("_mm_rcp_ss", "approximate reciprocal"),
    ("_mm256_rcp_ps", "approximate reciprocal"),
    ("_mm_rsqrt_ps", "approximate reciprocal square root"),
    ("_mm_rsqrt_ss", "approximate reciprocal square root"),
    ("_mm256_rsqrt_ps", "approximate reciprocal square root"),
    ("vrecpe", "approximate reciprocal"),
    ("vrecps", "approximate reciprocal step"),
    ("vrsqrte", "approximate reciprocal square root"),
    ("vrsqrts", "approximate reciprocal square root step"),
    // Sum order not written out, and not the same as the x86-64 order.
    (
        "_mm_dp_ps",
        "dot product with a fixed but non-portable sum order",
    ),
    (
        "_mm_dp_pd",
        "dot product with a fixed but non-portable sum order",
    ),
    ("vaddv", "horizontal add; write out the add tree instead"),
    // min and max must give `a < b ? a : b` on every target.
    ("vminq_f32", "ARM FMIN keeps NaN; use compare and select"),
    ("vmaxq_f32", "ARM FMAX keeps NaN; use compare and select"),
    ("vminnmq", "IEEE minNum; use compare and select"),
    ("vmaxnmq", "IEEE maxNum; use compare and select"),
    ("vminvq", "horizontal min; use the same fold as SSE2"),
    ("vmaxvq", "horizontal max; use the same fold as SSE2"),
    (
        "vminnmvq",
        "horizontal IEEE minNum; use the same fold as SSE2",
    ),
    (
        "vmaxnmvq",
        "horizontal IEEE maxNum; use the same fold as SSE2",
    ),
    // Rounding mode must be in the immediate, never in a control register.
    ("_mm_setcsr", "MXCSR change"),
    ("_mm_getcsr", "MXCSR change"),
    ("_MM_SET_FLUSH_ZERO_MODE", "MXCSR flush-to-zero"),
    ("_MM_SET_DENORMALS_ZERO_MODE", "MXCSR denormals-are-zero"),
    (
        "_MM_FROUND_CUR_DIRECTION",
        "takes the rounding mode from MXCSR",
    ),
    (
        "vrndaq",
        "rounds ties away from zero; x86-64 has no such instruction",
    ),
    // One code path, chosen at compile time.
    (
        "is_x86_feature_detected",
        "processor feature test at run time",
    ),
    (
        "is_aarch64_feature_detected",
        "processor feature test at run time",
    ),
    // Fast math.
    ("algebraic_add", "permits reassociation"),
    ("algebraic_sub", "permits reassociation"),
    ("algebraic_mul", "permits reassociation"),
    ("algebraic_div", "permits reassociation"),
    ("fadd_fast", "fast-math intrinsic"),
    ("fsub_fast", "fast-math intrinsic"),
    ("fmul_fast", "fast-math intrinsic"),
    ("fdiv_fast", "fast-math intrinsic"),
    // Transcendental functions of the operating system. Use the pinned `libm`.
    ("f32::sin(", "std transcendental; use math::sin"),
    ("f64::sin(", "std transcendental; use math::sin"),
    ("f32::cos(", "std transcendental; use math::cos"),
    ("f64::cos(", "std transcendental; use math::cos"),
    ("f32::tan(", "std transcendental; use math::tan"),
    ("f64::tan(", "std transcendental; use math::tan"),
    ("f32::exp(", "std transcendental; use math::exp"),
    ("f64::exp(", "std transcendental; use math::exp"),
    ("f32::ln(", "std transcendental; use math::ln"),
    ("f64::ln(", "std transcendental; use math::ln"),
    ("f32::powf(", "std transcendental; use math::powf"),
    ("f64::powf(", "std transcendental; use math::powf"),
    ("f32::atan2(", "std transcendental; use math::atan2"),
    ("f64::atan2(", "std transcendental; use math::atan2"),
    // Ties away from zero. rewrite.md requires ties to even.
    (
        "f32::round(",
        "rounds ties away from zero; use round_ties_even",
    ),
    (
        "f64::round(",
        "rounds ties away from zero; use round_ties_even",
    ),
    // ---------------------------------------------------------------------
    // Integer rules.
    //
    // The integer vector types get hand-written SIMD bodies. The rules below
    // hold that work to the same standard as the float work: one result on
    // every machine, and one instruction set on every machine.
    //
    // Each x86-64 pattern here starts at the underscore that comes after the
    // width, thus one rule catches the 128-bit, the 256-bit and the 512-bit
    // name of the same operation.
    // ---------------------------------------------------------------------
    //
    // AVX512 integer operations. The pinned flag is `-C target-cpu=x86-64-v3`,
    // and that level stops at AVX2. The standard library gates each name below
    // on `avx512f`, `avx512dq` or `avx512bw`, thus one of them breaks the build
    // instead of giving a wrong answer. The rule states the ceiling and stops a
    // person who reaches for a 64-bit lane operation that only AVX512 has.
    ("_mullo_epi64", "AVX512DQ; above the x86-64-v3 ceiling"),
    ("_min_epi64", "AVX512F; above the x86-64-v3 ceiling"),
    ("_max_epi64", "AVX512F; above the x86-64-v3 ceiling"),
    ("_min_epu64", "AVX512F; above the x86-64-v3 ceiling"),
    ("_max_epu64", "AVX512F; above the x86-64-v3 ceiling"),
    ("_abs_epi64", "AVX512F; above the x86-64-v3 ceiling"),
    ("_srai_epi64", "AVX512F; above the x86-64-v3 ceiling"),
    // `_mm_sra_epi64`, `_mm_srav_epi64`, `_mm_sllv_epi16`, `_mm_srav_epi16`
    // and `_mm_srlv_epi16` are above the ceiling also. The shift rules below
    // catch all five, thus they get no rule here. A second rule would report
    // the same line twice.
    //
    // Shifts that take the count from a register or from a vector. The three
    // backends give three answers:
    //
    //   value -8, count -1: `_mm_sllv_epi32` gives 0x00000000, and
    //                       `vshlq_s32` gives 0xfffffffc.
    //   `IVec4::ONE << 32`: scalar Rust masks the count and gives 1.
    //
    // x86-64 treats a count above the lane width as a flush to zero, ARM reads
    // a negative count as a shift in the other direction, and Rust masks the
    // count. No fold-up is possible, thus `Shl` and `Shr` stay scalar until
    // somebody opens this decision again.
    //
    // The forms that hold the count in the instruction encoding stay
    // permitted: `_mm_slli_*` and `_mm_srli_*` and `_mm_srai_epi16` and
    // `_mm_srai_epi32` on x86-64, and `vshlq_n_*` and `vshrq_n_*` on ARM. The
    // author writes that count as a constant, the compiler checks the range,
    // and all three backends then agree. `_srai_epi64` is banned above for the
    // instruction set reason only, not for this one.
    (
        "_sll_epi",
        "shift count in a register; x86-64, ARM and Rust disagree",
    ),
    (
        "_srl_epi",
        "shift count in a register; x86-64, ARM and Rust disagree",
    ),
    (
        "_sra_epi",
        "shift count in a register; x86-64, ARM and Rust disagree",
    ),
    (
        "_sllv_epi",
        "shift count in a vector; x86-64, ARM and Rust disagree",
    ),
    (
        "_srlv_epi",
        "shift count in a vector; x86-64, ARM and Rust disagree",
    ),
    (
        "_srav_epi",
        "shift count in a vector; x86-64, ARM and Rust disagree",
    ),
    (
        "vshlq_s",
        "shift count in a vector; x86-64, ARM and Rust disagree",
    ),
    (
        "vshlq_u",
        "shift count in a vector; x86-64, ARM and Rust disagree",
    ),
    (
        "vshl_s",
        "shift count in a vector; x86-64, ARM and Rust disagree",
    ),
    (
        "vshl_u",
        "shift count in a vector; x86-64, ARM and Rust disagree",
    ),
    (
        "vqshlq_s",
        "saturating shift; x86-64 has no such instruction",
    ),
    (
        "vqshlq_u",
        "saturating shift; x86-64 has no such instruction",
    ),
    (
        "vqshl_s",
        "saturating shift; x86-64 has no such instruction",
    ),
    (
        "vqshl_u",
        "saturating shift; x86-64 has no such instruction",
    ),
    //
    // Horizontal integer reductions on ARM, in the 64-bit register forms. A
    // 4-lane narrow vector is 64 bits wide, thus an integer fold reaches for
    // `vminv_s16` and not for `vminvq_s32`. The existing `vaddv`, `vminvq` and
    // `vmaxvq` rules already catch every `vaddv*` name and every 128-bit min
    // and max name, integer forms included, thus only the gaps are here. The
    // reason is the reason of the float rules: the order of the fold is not
    // written out, and x86-64 has no instruction with the same order.
    ("vminv_", "horizontal min; use the same fold as SSE2"),
    ("vmaxv_", "horizontal max; use the same fold as SSE2"),
    (
        "vaddlv",
        "horizontal widening add; write out the add tree instead",
    ),
    //
    // ARM saturating operations that x86-64 cannot answer. `_mm_abs_epi16` of
    // the most negative value gives that value again, and so does scalar Rust
    // in release. `vqabsq_s16` gives the most positive value instead, thus the
    // two backends part company. x86-64 has no saturating negate at all.
    (
        "vqabs",
        "saturating abs; x86-64 and Rust wrap at the most negative value",
    ),
    ("vqneg", "saturating negate; x86-64 has no such instruction"),
    //
    // Saturating add and subtract. Both processors have these instructions, thus
    // the danger is not a missing instruction. The danger is that they give the
    // wrong answer. The `+` and `-` operators of this library wrap in release,
    // and `saturating_add` and `saturating_sub` are `const fn` and stay scalar.
    // Thus a SIMD body never needs a saturating instruction. If one gets into an
    // `Add` body, release saturates and debug wraps, and the two disagree. The
    // stored bit patterns for `add` do not cross the boundary, thus the golden
    // test does not catch this. These rules catch it instead.
    ("vqadd", "saturating add; the operators of this library wrap"),
    ("vqsub", "saturating subtract; the operators of this library wrap"),
    ("_mm_adds_", "saturating add; the operators of this library wrap"),
    (
        "_mm_subs_",
        "saturating subtract; the operators of this library wrap",
    ),
    //
    // Rounding shifts with the count in a vector. Same fault class as `vshlq_s`:
    // the count is signed, thus a negative count shifts the other way, and no
    // x86-64 instruction agrees. The rounding step has no scalar Rust form.
    ("vrshl", "rounding shift with a vector count; no x86-64 equivalent"),
    (
        "vqrshl",
        "saturating rounding shift; no x86-64 equivalent",
    ),
];

/// Removes `//` line comments and `{# ... #}` Tera comments.
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(i) = rest.find("{#") {
        out.push_str(&rest[..i]);
        match rest[i..].find("#}") {
            Some(j) => rest = &rest[i + j + 2..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);

    out.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, exts, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| exts.contains(&e))
        {
            out.push(path);
        }
    }
}

#[test]
fn no_forbidden_instructions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect(&root.join("src"), &["rs"], &mut files);
    collect(&root.join("templates"), &["tera"], &mut files);
    assert!(!files.is_empty(), "found no source files to scan");

    let mut violations = Vec::new();
    for path in &files {
        let text = strip_comments(&fs::read_to_string(path).unwrap());
        for (line_no, line) in text.lines().enumerate() {
            for (pattern, why) in FORBIDDEN {
                if line.contains(pattern) {
                    violations.push(format!(
                        "{}:{}: `{}` is not permitted ({})",
                        path.strip_prefix(root).unwrap().display(),
                        line_no + 1,
                        pattern,
                        why
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} forbidden item(s) found. See rewrite.md.\n{}",
        violations.len(),
        violations.join("\n")
    );
}
