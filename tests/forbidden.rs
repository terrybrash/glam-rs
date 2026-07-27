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
    ("_mm_dp_ps", "dot product with a fixed but non-portable sum order"),
    ("_mm_dp_pd", "dot product with a fixed but non-portable sum order"),
    ("vaddv", "horizontal add; write out the add tree instead"),
    // min and max must give `a < b ? a : b` on every target.
    ("vminq_f32", "ARM FMIN keeps NaN; use compare and select"),
    ("vmaxq_f32", "ARM FMAX keeps NaN; use compare and select"),
    ("vminnmq", "IEEE minNum; use compare and select"),
    ("vmaxnmq", "IEEE maxNum; use compare and select"),
    ("vminvq", "horizontal min; use the same fold as SSE2"),
    ("vmaxvq", "horizontal max; use the same fold as SSE2"),
    ("vminnmvq", "horizontal IEEE minNum; use the same fold as SSE2"),
    ("vmaxnmvq", "horizontal IEEE maxNum; use the same fold as SSE2"),
    // Rounding mode must be in the immediate, never in a control register.
    ("_mm_setcsr", "MXCSR change"),
    ("_mm_getcsr", "MXCSR change"),
    ("_MM_SET_FLUSH_ZERO_MODE", "MXCSR flush-to-zero"),
    ("_MM_SET_DENORMALS_ZERO_MODE", "MXCSR denormals-are-zero"),
    ("_MM_FROUND_CUR_DIRECTION", "takes the rounding mode from MXCSR"),
    ("vrndaq", "rounds ties away from zero; x86-64 has no such instruction"),
    // One code path, chosen at compile time.
    ("is_x86_feature_detected", "processor feature test at run time"),
    ("is_aarch64_feature_detected", "processor feature test at run time"),
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
    ("f32::round(", "rounds ties away from zero; use round_ties_even"),
    ("f64::round(", "rounds ties away from zero; use round_ties_even"),
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
