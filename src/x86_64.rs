use core::arch::x86_64::*;

#[repr(C)]
union UnionCast {
    u32x4: [u32; 4],
    f32x4: [f32; 4],
    m128: __m128,
}

pub const fn m128_from_f32x4(f32x4: [f32; 4]) -> __m128 {
    unsafe { UnionCast { f32x4 }.m128 }
}

/// Calculates the vector 3 dot product and returns answer in x lane of __m128.
#[inline(always)]
pub(crate) unsafe fn dot3_in_x(lhs: __m128, rhs: __m128) -> __m128 {
    let x2_y2_z2_w2 = _mm_mul_ps(lhs, rhs);
    let y2_0_0_0 = _mm_shuffle_ps(x2_y2_z2_w2, x2_y2_z2_w2, 0b00_00_00_01);
    let z2_0_0_0 = _mm_shuffle_ps(x2_y2_z2_w2, x2_y2_z2_w2, 0b00_00_00_10);
    let x2y2_0_0_0 = _mm_add_ss(x2_y2_z2_w2, y2_0_0_0);
    _mm_add_ss(x2y2_0_0_0, z2_0_0_0)
}

/// Calculates the vector 4 dot product and returns answer in x lane of __m128.
#[inline(always)]
pub(crate) unsafe fn dot4_in_x(lhs: __m128, rhs: __m128) -> __m128 {
    let x2_y2_z2_w2 = _mm_mul_ps(lhs, rhs);
    let z2_w2_0_0 = _mm_shuffle_ps(x2_y2_z2_w2, x2_y2_z2_w2, 0b00_00_11_10);
    let x2z2_y2w2_0_0 = _mm_add_ps(x2_y2_z2_w2, z2_w2_0_0);
    let y2w2_0_0_0 = _mm_shuffle_ps(x2z2_y2w2_0_0, x2z2_y2w2_0_0, 0b00_00_00_01);
    _mm_add_ps(x2z2_y2w2_0_0, y2w2_0_0_0)
}

#[inline]
pub(crate) unsafe fn dot3(lhs: __m128, rhs: __m128) -> f32 {
    _mm_cvtss_f32(dot3_in_x(lhs, rhs))
}

#[inline]
pub(crate) unsafe fn dot3_into_m128(lhs: __m128, rhs: __m128) -> __m128 {
    let dot_in_x = dot3_in_x(lhs, rhs);
    _mm_shuffle_ps(dot_in_x, dot_in_x, 0b00_00_00_00)
}

#[inline]
pub(crate) unsafe fn dot4(lhs: __m128, rhs: __m128) -> f32 {
    _mm_cvtss_f32(dot4_in_x(lhs, rhs))
}

#[inline]
pub(crate) unsafe fn dot4_into_m128(lhs: __m128, rhs: __m128) -> __m128 {
    let dot_in_x = dot4_in_x(lhs, rhs);
    _mm_shuffle_ps(dot_in_x, dot_in_x, 0b00_00_00_00)
}

#[inline]
pub(crate) unsafe fn m128_floor(v: __m128) -> __m128 {
    // SSE4.1. The rounding mode is in the immediate, not in MXCSR, and it
    // agrees with ARM's `FRINTM`. See rewrite.md.
    _mm_floor_ps(v)
}

#[inline]
pub(crate) unsafe fn m128_ceil(v: __m128) -> __m128 {
    // SSE4.1, and it agrees with ARM's `FRINTP`. See rewrite.md.
    _mm_ceil_ps(v)
}

#[inline]
pub(crate) unsafe fn m128_abs(v: __m128) -> __m128 {
    _mm_and_ps(v, _mm_castsi128_ps(_mm_set1_epi32(0x7f_ff_ff_ff)))
}

#[inline]
pub(crate) unsafe fn m128_round(v: __m128) -> __m128 {
    // Round to nearest, ties to even. The mode is in the immediate, so MXCSR
    // cannot change it. This agrees with ARM's `FRINTN`. See rewrite.md.
    _mm_round_ps::<{ _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC }>(v)
}

#[inline]
pub(crate) unsafe fn m128_trunc(v: __m128) -> __m128 {
    // Round toward zero, and it agrees with ARM's `FRINTZ`. See rewrite.md.
    _mm_round_ps::<{ _MM_FROUND_TO_ZERO | _MM_FROUND_NO_EXC }>(v)
}

// 256-bit helpers for `Mat4`.
//
// A 256-bit lane-wise operation gives exactly the result of two 128-bit
// operations on the same lanes, thus a `Mat4` operation that processes two
// columns at once still agrees with the NEON backend bit for bit. ARM keeps
// the 128-bit code. The two backends must agree on results, not on the count
// of instructions. See rewrite.md.
//
// `x86-64-v3` contains AVX2 and FMA, thus these need no feature test and every
// SSE intrinsic in this crate is VEX encoded. There is no AVX to SSE
// transition penalty.

/// Reads two adjacent columns of a `Mat4` as one 256-bit value.
///
/// `Mat4` is aligned to 16, not to 32, thus this must be the unaligned load.
/// Raising the alignment would change the size and the layout of `Mat4`.
///
/// # Safety
///
/// `p` must point to 8 readable `f32` values.
#[inline]
pub(crate) unsafe fn m256_load_cols(p: *const f32) -> __m256 {
    _mm256_loadu_ps(p)
}

/// Joins two 128-bit values into one 256-bit value.
#[inline]
pub(crate) unsafe fn m256_join(lo: __m128, hi: __m128) -> __m256 {
    _mm256_insertf128_ps::<1>(_mm256_castps128_ps256(lo), hi)
}

/// The low 128-bit lane, which holds the first of the two columns.
#[inline]
pub(crate) unsafe fn m256_low(v: __m256) -> __m128 {
    _mm256_castps256_ps128(v)
}

/// The high 128-bit lane, which holds the second of the two columns.
#[inline]
pub(crate) unsafe fn m256_high(v: __m256) -> __m128 {
    _mm256_extractf128_ps::<1>(v)
}

