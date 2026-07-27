// ---------------------------------------------------------------------------
// Exact operations
//
// IEEE 754 defines the result of each one, so x86-64 and ARM give the same
// bits. Each lowers to one instruction on both. See rewrite.md.
// ---------------------------------------------------------------------------

#[inline(always)]
pub(crate) fn abs(f: f64) -> f64 {
    f64::abs(f)
}

#[inline(always)]
pub(crate) fn sqrt(f: f64) -> f64 {
    f64::sqrt(f)
}

#[inline(always)]
pub(crate) fn copysign(f: f64, sign: f64) -> f64 {
    f64::copysign(f, sign)
}

#[inline(always)]
pub(crate) fn signum(f: f64) -> f64 {
    f64::signum(f)
}

#[inline(always)]
pub(crate) fn trunc(f: f64) -> f64 {
    f64::trunc(f)
}

#[inline(always)]
pub(crate) fn ceil(f: f64) -> f64 {
    f64::ceil(f)
}

#[inline(always)]
pub(crate) fn floor(f: f64) -> f64 {
    f64::floor(f)
}

#[inline(always)]
pub(crate) fn mul_add(a: f64, b: f64, c: f64) -> f64 {
    f64::mul_add(a, b, c)
}

/// Rounds to the nearest integer. A tie goes to the even value.
///
/// `f64::round` sends a tie away from zero, and x86-64 has no instruction for
/// that rule. This rule lowers to `ROUNDPS` on x86-64 and to `FRINTN` on ARM,
/// so both give the same bits. See rewrite.md.
#[inline(always)]
pub(crate) fn round(f: f64) -> f64 {
    f64::round_ties_even(f)
}

// ---------------------------------------------------------------------------
// Transcendental operations
//
// The math library of the operating system gives different results on each
// platform. These use one pinned version of the pure-Rust `libm` crate on
// every target instead. See rewrite.md.
// ---------------------------------------------------------------------------

#[inline(always)]
pub(crate) fn acos_approx(f: f64) -> f64 {
    libm::acos(f64::clamp(f, -1.0, 1.0))
}

#[inline(always)]
pub(crate) fn atan2(f: f64, other: f64) -> f64 {
    libm::atan2(f, other)
}

#[allow(unused)]
#[inline(always)]
pub(crate) fn cos(f: f64) -> f64 {
    libm::cos(f)
}

#[allow(unused)]
#[inline(always)]
pub(crate) fn sin(f: f64) -> f64 {
    libm::sin(f)
}

#[inline(always)]
pub(crate) fn sin_cos(f: f64) -> (f64, f64) {
    libm::sincos(f)
}

#[inline(always)]
pub(crate) fn tan(f: f64) -> f64 {
    libm::tan(f)
}

#[inline(always)]
pub(crate) fn exp(f: f64) -> f64 {
    libm::exp(f)
}

#[inline(always)]
pub(crate) fn exp2(f: f64) -> f64 {
    libm::exp2(f)
}

#[inline(always)]
pub(crate) fn ln(f: f64) -> f64 {
    libm::log(f)
}

#[inline(always)]
pub(crate) fn log2(f: f64) -> f64 {
    libm::log2(f)
}

#[inline(always)]
pub(crate) fn powf(f: f64, n: f64) -> f64 {
    libm::pow(f, n)
}

// ---------------------------------------------------------------------------
// Built from the operations above
// ---------------------------------------------------------------------------

#[inline]
pub fn div_euclid(a: f64, b: f64) -> f64 {
    // Based on https://doc.rust-lang.org/src/std/f64.rs.html#293
    let q = trunc(a / b);
    if a % b < 0.0 {
        return if b > 0.0 { q - 1.0 } else { q + 1.0 };
    }
    q
}

#[inline]
pub fn rem_euclid(a: f64, b: f64) -> f64 {
    let r = a % b;
    if r < 0.0 {
        r + abs(b)
    } else {
        r
    }
}
