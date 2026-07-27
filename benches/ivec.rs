//! Benchmarks for the 4-lane integer vectors.
//!
//! The narrow types `I8Vec4`, `U8Vec4`, `I16Vec4` and `U16Vec4` compile into
//! many scalar instructions today, while `IVec4` and `UVec4` compile into one
//! NEON or SSE2 instruction. `IVec4` and `UVec4` are here as the control
//! group. A change to the narrow types must not move the control group.
//!
//! Each operation gets two shapes:
//!
//! - The plain shape does one operation for each criterion iteration. It shows
//!   the latency of a single operation.
//! - The batch shape runs a slice of 8192 vectors through the operation for
//!   each criterion iteration. It keeps several vectors in flight, thus it
//!   shows the throughput. A type that expands into many scalar instructions
//!   loses most of its time in this shape. Divide the reported time by 8192 to
//!   get the time for one operation.
//!
//! Run this on a native target. Numbers from an emulator have no value.

#[path = "support/macros.rs"]
#[macro_use]
mod macros;
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use glam::{I16Vec4, I64Vec4, I8Vec4, IVec4, U16Vec4, U8Vec4, UVec4};
use std::ops::{Add, Mul, Sub};
use std::time::Duration;
use support::*;

/// Builds both shapes of every operation for one vector type.
///
/// `sub_lhs` gives the left side of the subtract. An unsigned subtract panics
/// in a debug build when the left side is smaller than the right side, thus
/// the unsigned types pass a generator with larger values here. `abs` is
/// absent for the unsigned types, because they have no `abs`.
macro_rules! ivec_benches {
    (
        name => $group:ident,
        ty => $ty:ident,
        desc => $desc:expr,
        rand => $rand:expr,
        sub_lhs => $sub_lhs:expr,
        clamp_min => $clamp_min:expr,
        clamp_max => $clamp_max:expr
        $(, abs => $abs:ident, batch_abs => $batch_abs:ident)?
    ) => {
        pub(crate) fn $group(c: &mut Criterion) {
            #[inline]
            fn clamp_op(v: $ty) -> $ty {
                v.clamp($ty::splat($clamp_min), $ty::splat($clamp_max))
            }

            // One operation for each criterion iteration.
            bench_binop!(add, concat!($desc, " add"), op => add, from => $rand);
            bench_binop!(
                sub,
                concat!($desc, " sub"),
                op => sub,
                from1 => $sub_lhs,
                from2 => $rand
            );
            bench_binop!(mul, concat!($desc, " mul"), op => mul, from => $rand);
            bench_binop!(min, concat!($desc, " min"), op => min, from => $rand);
            bench_binop!(max, concat!($desc, " max"), op => max, from => $rand);
            bench_func!(clamp, concat!($desc, " clamp"), op => clamp_op, from => $rand);
            bench_binop!(cmplt, concat!($desc, " cmplt"), op => cmplt, from => $rand);
            bench_select!(
                select,
                concat!($desc, " select"),
                ty => $ty,
                op => cmplt,
                from => $rand
            );
            bench_binop!(dot, concat!($desc, " dot"), op => dot, from => $rand);
            bench_unop!(
                element_sum,
                concat!($desc, " element_sum"),
                op => element_sum,
                from => $rand
            );
            $( bench_unop!($abs, concat!($desc, " abs"), op => abs, from => $rand); )?

            // One slice of 8192 vectors for each criterion iteration.
            bench_batch_binop!(
                batch_add,
                concat!($desc, " batch add"),
                op => add,
                from => $rand
            );
            bench_batch_binop!(
                batch_sub,
                concat!($desc, " batch sub"),
                op => sub,
                from1 => $sub_lhs,
                from2 => $rand
            );
            bench_batch_binop!(
                batch_mul,
                concat!($desc, " batch mul"),
                op => mul,
                from => $rand
            );
            bench_batch_binop!(
                batch_min,
                concat!($desc, " batch min"),
                op => min,
                from => $rand
            );
            bench_batch_binop!(
                batch_max,
                concat!($desc, " batch max"),
                op => max,
                from => $rand
            );
            bench_batch_func!(
                batch_clamp,
                concat!($desc, " batch clamp"),
                op => clamp_op,
                from => $rand
            );
            bench_batch_binop!(
                batch_cmplt,
                concat!($desc, " batch cmplt"),
                op => cmplt,
                from => $rand
            );
            bench_batch_select!(
                batch_select,
                concat!($desc, " batch select"),
                ty => $ty,
                op => cmplt,
                from => $rand
            );
            bench_batch_binop!(
                batch_dot,
                concat!($desc, " batch dot"),
                op => dot,
                from => $rand
            );
            // Saturating and wrapping arithmetic. These are `const fn` today and
            // thus always scalar. The measurement decides whether dropping
            // `const` is worth it. See rewrite.md.
            bench_batch_binop!(
                batch_sat_add,
                concat!($desc, " batch saturating_add"),
                op => saturating_add,
                from => $rand
            );
            bench_batch_binop!(
                batch_sat_sub,
                concat!($desc, " batch saturating_sub"),
                op => saturating_sub,
                from => $rand
            );
            bench_batch_binop!(
                batch_wrap_add,
                concat!($desc, " batch wrapping_add"),
                op => wrapping_add,
                from => $rand
            );
            bench_batch_binop!(
                batch_wrap_mul,
                concat!($desc, " batch wrapping_mul"),
                op => wrapping_mul,
                from => $rand
            );
            bench_batch_unop!(
                batch_element_sum,
                concat!($desc, " batch element_sum"),
                op => element_sum,
                from => $rand
            );
            $(
                bench_batch_unop!(
                    $batch_abs,
                    concat!($desc, " batch abs"),
                    op => abs,
                    from => $rand
                );
            )?

            add(c);
            sub(c);
            mul(c);
            min(c);
            max(c);
            clamp(c);
            cmplt(c);
            select(c);
            dot(c);
            element_sum(c);
            $( $abs(c); )?

            batch_add(c);
            batch_sub(c);
            batch_mul(c);
            batch_min(c);
            batch_max(c);
            batch_clamp(c);
            batch_cmplt(c);
            batch_select(c);
            batch_dot(c);
            batch_element_sum(c);
            batch_sat_add(c);
            batch_sat_sub(c);
            batch_wrap_add(c);
            batch_wrap_mul(c);
            $( $batch_abs(c); )?
        }
    };
}

ivec_benches!(
    name => i8vec4_benches,
    ty => I8Vec4,
    desc => "i8vec4",
    rand => random_i8vec4,
    sub_lhs => random_i8vec4,
    clamp_min => -3,
    clamp_max => 3,
    abs => abs,
    batch_abs => batch_abs
);

ivec_benches!(
    name => u8vec4_benches,
    ty => U8Vec4,
    desc => "u8vec4",
    rand => random_u8vec4,
    sub_lhs => random_u8vec4_high,
    clamp_min => 1,
    clamp_max => 5
);

ivec_benches!(
    name => i16vec4_benches,
    ty => I16Vec4,
    desc => "i16vec4",
    rand => random_i16vec4,
    sub_lhs => random_i16vec4,
    clamp_min => -45,
    clamp_max => 45,
    abs => abs,
    batch_abs => batch_abs
);

ivec_benches!(
    name => u16vec4_benches,
    ty => U16Vec4,
    desc => "u16vec4",
    rand => random_u16vec4,
    sub_lhs => random_u16vec4_high,
    clamp_min => 20,
    clamp_max => 90
);

ivec_benches!(
    name => ivec4_benches,
    ty => IVec4,
    desc => "ivec4",
    rand => random_ivec4,
    sub_lhs => random_ivec4,
    clamp_min => -10000,
    clamp_max => 10000,
    abs => abs,
    batch_abs => batch_abs
);

ivec_benches!(
    name => uvec4_benches,
    ty => UVec4,
    desc => "uvec4",
    rand => random_uvec4,
    sub_lhs => random_uvec4_high,
    clamp_min => 8000,
    clamp_max => 24000
);

ivec_benches!(
    name => i64vec4_benches,
    ty => I64Vec4,
    desc => "i64vec4",
    rand => random_i64vec4,
    sub_lhs => random_i64vec4,
    clamp_min => -500_000_000,
    clamp_max => 500_000_000,
    abs => abs,
    batch_abs => batch_abs
);

// The list holds 148 benchmarks. The default criterion times make the run
// longer than 20 minutes, thus the times below are shorter. Keep them the same
// for the baseline run and for every later run, or the two are not comparable.
criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));
    targets =
        i8vec4_benches,
        u8vec4_benches,
        i16vec4_benches,
        u16vec4_benches,
        ivec4_benches,
        uvec4_benches,
        i64vec4_benches,
);

criterion_main!(benches);
