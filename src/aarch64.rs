use core::arch::aarch64::*;

union UnionCast {
    // u32x4: [u32; 4],
    f32x4: [f32; 4],
    v: float32x4_t,
}

#[inline]
pub const fn f32x4_from_array(f32x4: [f32; 4]) -> float32x4_t {
    unsafe { UnionCast { f32x4 }.v }
}

/// Calculates the vector 3 dot product and returns the answer in lane 0.
///
/// The sum order is written out, and it is the same order the SSE2 backend
/// uses: `(x + y) + z`. Do not replace this with `vaddvq_f32`, because the
/// order of that instruction is not the same. See rewrite.md.
#[inline]
pub(crate) unsafe fn dot3_in_x(lhs: float32x4_t, rhs: float32x4_t) -> float32x4_t {
    let x2_y2_z2_w2 = vmulq_f32(lhs, rhs);
    let y2 = vdupq_laneq_f32(x2_y2_z2_w2, 1);
    let z2 = vdupq_laneq_f32(x2_y2_z2_w2, 2);
    let x2y2 = vaddq_f32(x2_y2_z2_w2, y2);
    vaddq_f32(x2y2, z2)
}

#[inline]
pub(crate) unsafe fn dot3_into_f32x4(lhs: float32x4_t, rhs: float32x4_t) -> float32x4_t {
    vdupq_laneq_f32(dot3_in_x(lhs, rhs), 0)
}

/// Calculates the vector 4 dot product.
///
/// The sum order is written out, and it is the same order the SSE2 backend
/// uses: `(x + z) + (y + w)`. Do not replace this with `vaddvq_f32`. See
/// rewrite.md.
#[inline]
pub(crate) unsafe fn dot4(lhs: float32x4_t, rhs: float32x4_t) -> f32 {
    let x2_y2_z2_w2 = vmulq_f32(lhs, rhs);
    let x2_y2 = vget_low_f32(x2_y2_z2_w2);
    let z2_w2 = vget_high_f32(x2_y2_z2_w2);
    let x2z2_y2w2 = vadd_f32(x2_y2, z2_w2);
    vget_lane_f32(x2z2_y2w2, 0) + vget_lane_f32(x2z2_y2w2, 1)
}

#[inline]
pub(crate) unsafe fn dot4_into_f32x4(lhs: float32x4_t, rhs: float32x4_t) -> float32x4_t {
    let dot = dot4(lhs, rhs);
    vld1q_dup_f32(&dot as *const f32)
}
