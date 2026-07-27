//! Compares the bit patterns of representative operations against a stored file.
//!
//! This test does two jobs at once:
//!
//! 1. It runs on x86-64 and on ARM against the **same** stored file, so it
//!    fails if the two backends stop agreeing.
//! 2. It fails if a new toolchain, a new `libm`, or an edit changes any result.
//!
//! To store new values after an intended change:
//!
//! ```text
//! GLAM_BLESS_GOLDEN=1 cargo test --test golden_bits
//! ```
//!
//! Read the diff before you keep it. Every line is a promise that the
//! simulation gives the same answer on every machine. See rewrite.md.

use glam::{Affine3A, EulerRot, Mat3A, Mat4, Quat, Vec3, Vec3A, Vec4};
use glam::{I16Vec2, I16Vec3, I16Vec4, I64Vec2, I64Vec3, I64Vec4, I8Vec2, I8Vec3, I8Vec4};
use glam::{IVec2, IVec3, IVec4, UVec2, UVec3, UVec4};
use glam::{U16Vec2, U16Vec3, U16Vec4, U64Vec2, U64Vec3, U64Vec4, U8Vec2, U8Vec3, U8Vec4};
use std::fmt::Write;

/// Makes one `Dump` method that prints the lanes of an integer vector as hex.
///
/// The signed types print the bit pattern, thus the cast to the unsigned type
/// of the same width. The width is the count of hex digits of one lane.
macro_rules! int_lane_dump {
    ($name:ident, $vt:ty, $ust:ty, $width:expr, $($lane:ident),+) => {
        fn $name(&mut self, label: &str, v: $vt) {
            write!(self.0, "{label:<32}").unwrap();
            $(write!(self.0, " {:0w$x}", v.$lane as $ust, w = $width).unwrap();)+
            writeln!(self.0).unwrap();
        }
    };
}

struct Dump(String);

impl Dump {
    fn f(&mut self, label: &str, v: f32) {
        writeln!(self.0, "{label:<32} {:08x}", v.to_bits()).unwrap();
    }
    fn v3a(&mut self, label: &str, v: Vec3A) {
        writeln!(
            self.0,
            "{label:<32} {:08x} {:08x} {:08x}",
            v.x.to_bits(),
            v.y.to_bits(),
            v.z.to_bits()
        )
        .unwrap();
    }
    fn v4(&mut self, label: &str, v: Vec4) {
        writeln!(
            self.0,
            "{label:<32} {:08x} {:08x} {:08x} {:08x}",
            v.x.to_bits(),
            v.y.to_bits(),
            v.z.to_bits(),
            v.w.to_bits()
        )
        .unwrap();
    }
    fn q(&mut self, label: &str, v: Quat) {
        writeln!(
            self.0,
            "{label:<32} {:08x} {:08x} {:08x} {:08x}",
            v.x.to_bits(),
            v.y.to_bits(),
            v.z.to_bits(),
            v.w.to_bits()
        )
        .unwrap();
    }
    fn m4(&mut self, label: &str, m: Mat4) {
        write!(self.0, "{label:<32}").unwrap();
        for x in m.to_cols_array() {
            write!(self.0, " {:08x}", x.to_bits()).unwrap();
        }
        writeln!(self.0).unwrap();
    }

    // The integer helpers. A comparison gives a `BVec`, and the bitmask of that
    // `BVec` is the value that goes in the file. Thus the comparison itself is
    // part of the stored bits.
    fn mask(&mut self, label: &str, m: u32) {
        writeln!(self.0, "{label:<32} {m:02x}").unwrap();
    }
    fn s8(&mut self, label: &str, v: u8) {
        writeln!(self.0, "{label:<32} {v:02x}").unwrap();
    }
    fn s16(&mut self, label: &str, v: u16) {
        writeln!(self.0, "{label:<32} {v:04x}").unwrap();
    }
    fn s32(&mut self, label: &str, v: u32) {
        writeln!(self.0, "{label:<32} {v:08x}").unwrap();
    }
    fn s64(&mut self, label: &str, v: u64) {
        writeln!(self.0, "{label:<32} {v:016x}").unwrap();
    }

    int_lane_dump!(i8v2, I8Vec2, u8, 2, x, y);
    int_lane_dump!(i8v3, I8Vec3, u8, 2, x, y, z);
    int_lane_dump!(i8v4, I8Vec4, u8, 2, x, y, z, w);
    int_lane_dump!(u8v2, U8Vec2, u8, 2, x, y);
    int_lane_dump!(u8v3, U8Vec3, u8, 2, x, y, z);
    int_lane_dump!(u8v4, U8Vec4, u8, 2, x, y, z, w);

    int_lane_dump!(i16v2, I16Vec2, u16, 4, x, y);
    int_lane_dump!(i16v3, I16Vec3, u16, 4, x, y, z);
    int_lane_dump!(i16v4, I16Vec4, u16, 4, x, y, z, w);
    int_lane_dump!(u16v2, U16Vec2, u16, 4, x, y);
    int_lane_dump!(u16v3, U16Vec3, u16, 4, x, y, z);
    int_lane_dump!(u16v4, U16Vec4, u16, 4, x, y, z, w);

    int_lane_dump!(i32v2, IVec2, u32, 8, x, y);
    int_lane_dump!(i32v3, IVec3, u32, 8, x, y, z);
    int_lane_dump!(i32v4, IVec4, u32, 8, x, y, z, w);
    int_lane_dump!(u32v2, UVec2, u32, 8, x, y);
    int_lane_dump!(u32v3, UVec3, u32, 8, x, y, z);
    int_lane_dump!(u32v4, UVec4, u32, 8, x, y, z, w);

    int_lane_dump!(i64v2, I64Vec2, u64, 16, x, y);
    int_lane_dump!(i64v3, I64Vec3, u64, 16, x, y, z);
    int_lane_dump!(i64v4, I64Vec4, u64, 16, x, y, z, w);
    int_lane_dump!(u64v2, U64Vec2, u64, 16, x, y);
    int_lane_dump!(u64v3, U64Vec3, u64, 16, x, y, z);
    int_lane_dump!(u64v4, U64Vec4, u64, 16, x, y, z, w);
}

/// Appends the operations that every integer vector type has.
///
/// The values of `a`, `b`, `p`, `n` and `q` must not overflow, because a debug
/// build panics on an overflow. The boundary cases go through `wrapping_*` and
/// `saturating_*`, which never panic.
macro_rules! int_common {
    (
        $d:ident, $tag:literal, $vf:ident, $sf:ident, $vt:ty, $ust:ty,
        a: $a:expr, b: $b:expr, p: $p:expr,
        clamp: $clo:expr, $chi:expr, div: $n:expr, $q:expr $(,)?
    ) => {{
        let a = <$vt>::from_array($a);
        let b = <$vt>::from_array($b);
        let p = <$vt>::from_array($p);
        let n = <$vt>::from_array($n);
        let q = <$vt>::from_array($q);
        let lo = <$vt>::MIN;
        let hi = <$vt>::MAX;
        let one = <$vt>::ONE;

        $d.0.push_str(concat!("--- ", $tag, " ---\n"));
        $d.$vf(concat!($tag, ".add"), a + b);
        $d.$vf(concat!($tag, ".sub"), a - b);
        $d.$vf(concat!($tag, ".mul"), a * b);
        $d.$vf(concat!($tag, ".min"), a.min(b));
        $d.$vf(concat!($tag, ".max"), a.max(b));
        $d.$vf(
            concat!($tag, ".clamp"),
            a.clamp(<$vt>::splat($clo), <$vt>::splat($chi)),
        );
        $d.mask(concat!($tag, ".cmpeq"), a.cmpeq(b).bitmask());
        $d.mask(concat!($tag, ".cmpne"), a.cmpne(b).bitmask());
        $d.mask(concat!($tag, ".cmplt"), a.cmplt(b).bitmask());
        $d.mask(concat!($tag, ".cmple"), a.cmple(b).bitmask());
        $d.mask(concat!($tag, ".cmpgt"), a.cmpgt(b).bitmask());
        $d.mask(concat!($tag, ".cmpge"), a.cmpge(b).bitmask());
        $d.$vf(concat!($tag, ".select"), <$vt>::select(a.cmplt(b), a, b));
        $d.$vf(concat!($tag, ".bitand"), a & b);
        $d.$vf(concat!($tag, ".bitor"), a | b);
        $d.$vf(concat!($tag, ".bitxor"), a ^ b);
        $d.$vf(concat!($tag, ".not"), !a);
        $d.$sf(concat!($tag, ".dot"), a.dot(b) as $ust);
        $d.$sf(concat!($tag, ".element_sum"), a.element_sum() as $ust);
        $d.$sf(
            concat!($tag, ".element_product"),
            p.element_product() as $ust,
        );
        $d.$sf(concat!($tag, ".min_element"), a.min_element() as $ust);
        $d.$sf(concat!($tag, ".max_element"), a.max_element() as $ust);
        $d.$vf(concat!($tag, ".wrapping_add hi+one"), hi.wrapping_add(one));
        $d.$vf(concat!($tag, ".wrapping_add hi+hi"), hi.wrapping_add(hi));
        $d.$vf(concat!($tag, ".wrapping_sub lo-one"), lo.wrapping_sub(one));
        $d.$vf(concat!($tag, ".wrapping_sub lo-hi"), lo.wrapping_sub(hi));
        $d.$vf(concat!($tag, ".wrapping_mul hi*hi"), hi.wrapping_mul(hi));
        $d.$vf(concat!($tag, ".wrapping_mul lo*lo"), lo.wrapping_mul(lo));
        $d.$vf(concat!($tag, ".wrapping_mul hi*lo"), hi.wrapping_mul(lo));
        $d.$vf(
            concat!($tag, ".saturating_add hi+one"),
            hi.saturating_add(one),
        );
        $d.$vf(
            concat!($tag, ".saturating_add hi+hi"),
            hi.saturating_add(hi),
        );
        $d.$vf(
            concat!($tag, ".saturating_add lo+lo"),
            lo.saturating_add(lo),
        );
        $d.$vf(
            concat!($tag, ".saturating_sub lo-one"),
            lo.saturating_sub(one),
        );
        $d.$vf(
            concat!($tag, ".saturating_sub lo-hi"),
            lo.saturating_sub(hi),
        );
        $d.$vf(
            concat!($tag, ".saturating_sub hi-lo"),
            hi.saturating_sub(lo),
        );
        $d.$vf(concat!($tag, ".div"), n / q);
        $d.$vf(concat!($tag, ".rem"), n % q);
        $d.$vf(concat!($tag, ".shl 1"), a << 1u32);
        $d.$vf(concat!($tag, ".shl 3"), a << 3u32);
        $d.$vf(concat!($tag, ".shr 1"), a >> 1u32);
        $d.$vf(concat!($tag, ".shr 3"), a >> 3u32);
    }};
}

/// Appends the operations that only a signed integer vector type has.
///
/// `x` must not hold the MIN value, because `MIN.abs()` panics in a debug
/// build. `y` holds negative values, thus a right shift of `y` shows the
/// difference between an arithmetic shift and a logical shift.
macro_rules! int_signed {
    (
        $d:ident, $tag:literal, $vf:ident, $vt:ty,
        abs: $x:expr, shr: $y:expr, sb: $h:expr, $l:expr $(,)?
    ) => {{
        let x = <$vt>::from_array($x);
        let y = <$vt>::from_array($y);
        let h = <$vt>::from_array($h);
        let l = <$vt>::from_array($l);

        $d.$vf(concat!($tag, ".abs"), x.abs());
        $d.$vf(concat!($tag, ".signum"), x.signum());
        $d.$vf(concat!($tag, ".shr neg 1"), y >> 1u32);
        $d.$vf(concat!($tag, ".shr neg 3"), y >> 3u32);
        $d.$vf(concat!($tag, ".shl neg 2"), y << 2u32);
        $d.$vf(concat!($tag, ".not neg"), !y);
        $d.mask(concat!($tag, ".is_negative"), y.is_negative_bitmask());
        $d.mask(concat!($tag, ".sb.cmpeq"), h.cmpeq(l).bitmask());
        $d.mask(concat!($tag, ".sb.cmpne"), h.cmpne(l).bitmask());
        $d.mask(concat!($tag, ".sb.cmplt"), h.cmplt(l).bitmask());
        $d.mask(concat!($tag, ".sb.cmple"), h.cmple(l).bitmask());
        $d.mask(concat!($tag, ".sb.cmpgt"), h.cmpgt(l).bitmask());
        $d.mask(concat!($tag, ".sb.cmpge"), h.cmpge(l).bitmask());
        $d.$vf(concat!($tag, ".sb.min"), h.min(l));
        $d.$vf(concat!($tag, ".sb.max"), h.max(l));
        $d.$vf(concat!($tag, ".sb.select"), <$vt>::select(h.cmpgt(l), h, l));
    }};
}

/// Appends the unsigned comparison cases, where the high bit is set.
///
/// x86-64 below AVX-512 has no unsigned packed comparison. A SIMD body must
/// therefore add a sign bias with an XOR before a signed comparison. If that
/// bias is absent, every mask below flips, and this test fails. This is the
/// most valuable case in the file.
macro_rules! int_unsigned {
    (
        $d:ident, $tag:literal, $vf:ident, $vt:ty, $ust:ty,
        hb: $h:expr, $l:expr $(,)?
    ) => {{
        let h = <$vt>::from_array($h);
        let l = <$vt>::from_array($l);
        let mid = <$vt>::splat((<$ust>::MAX >> 1) + 1);

        $d.mask(concat!($tag, ".hb.cmpeq"), h.cmpeq(l).bitmask());
        $d.mask(concat!($tag, ".hb.cmpne"), h.cmpne(l).bitmask());
        $d.mask(concat!($tag, ".hb.cmplt"), h.cmplt(l).bitmask());
        $d.mask(concat!($tag, ".hb.cmple"), h.cmple(l).bitmask());
        $d.mask(concat!($tag, ".hb.cmpgt"), h.cmpgt(l).bitmask());
        $d.mask(concat!($tag, ".hb.cmpge"), h.cmpge(l).bitmask());
        $d.mask(concat!($tag, ".hb.cmpge mid"), h.cmpge(mid).bitmask());
        $d.$vf(concat!($tag, ".hb.min"), h.min(l));
        $d.$vf(concat!($tag, ".hb.max"), h.max(l));
        $d.$vf(concat!($tag, ".hb.clamp"), h.clamp(<$vt>::ZERO, mid));
        $d.$vf(concat!($tag, ".hb.select"), <$vt>::select(h.cmpgt(l), h, l));
        $d.$vf(concat!($tag, ".hb.shr 1"), h >> 1u32);
        $d.$vf(concat!($tag, ".hb.shr 4"), h >> 4u32);
    }};
}

fn dump() -> String {
    let mut d = Dump(String::new());

    // Values chosen so the results are not exactly representable.
    let a = Vec4::new(0.1, 0.2, 0.3, 0.4);
    let b = Vec4::new(1.7, -2.3, 3.9, -4.1);
    let c3 = Vec3A::new(0.1, 0.2, 0.3);
    let d3 = Vec3A::new(1.7, -2.3, 3.9);

    d.0.push_str("--- reductions ---\n");
    d.f("vec4.dot", a.dot(b));
    d.f("vec4.length", b.length());
    d.f("vec4.length_squared", b.length_squared());
    d.f("vec4.element_sum", b.element_sum());
    d.f("vec4.element_product", b.element_product());
    d.v4("vec4.normalize", b.normalize());
    d.f("vec3a.dot", c3.dot(d3));
    d.f("vec3a.length", d3.length());
    d.f("vec3a.element_sum", d3.element_sum());
    d.v3a("vec3a.normalize", d3.normalize());
    d.v3a("vec3a.cross", c3.cross(d3));

    d.0.push_str("--- min / max ---\n");
    d.v4("vec4.min", a.min(b));
    d.v4("vec4.max", a.max(b));
    d.f("vec4.min_element", b.min_element());
    d.f("vec4.max_element", b.max_element());
    d.f("vec3a.min_element", d3.min_element());
    d.f("vec3a.max_element", d3.max_element());

    d.0.push_str("--- min / max with NaN ---\n");
    let n = Vec4::new(f32::NAN, 2.0, 3.0, 4.0);
    let m = Vec4::new(1.0, f32::NAN, 3.5, 4.5);
    d.v4("nan.min", n.min(m));
    d.v4("nan.max", n.max(m));
    d.f("nan.min_element", n.min_element());
    d.f("nan.max_element", n.max_element());
    d.v4("nan.clamp", Vec4::NAN.clamp(Vec4::NEG_ONE, Vec4::ONE));
    d.v3a(
        "nan.min_element3",
        Vec3A::new(f32::NAN, 2.0, 3.0).min(Vec3A::new(1.0, f32::NAN, 3.5)),
    );

    d.0.push_str("--- signed zero ---\n");
    let zp = Vec4::new(0.0, -0.0, 0.0, -0.0);
    let zn = Vec4::new(-0.0, 0.0, -0.0, 0.0);
    d.v4("zero.min", zp.min(zn));
    d.v4("zero.max", zp.max(zn));

    d.0.push_str("--- rounding ---\n");
    let r = Vec4::new(2.5, -2.5, 0.5, 1.5);
    d.v4("round", r.round());
    d.v4("floor", r.floor());
    d.v4("ceil", r.ceil());
    d.v4("trunc", r.trunc());
    d.v4("fract", r.fract());

    d.0.push_str("--- division and square root ---\n");
    d.v4("recip", b.recip());
    d.v4("div", a / b);
    d.f("sqrt_via_length", Vec4::new(2.0, 0.0, 0.0, 0.0).length());

    d.0.push_str("--- matrix ---\n");
    let mat = Mat4::from_cols(
        Vec4::new(0.1, 0.2, 0.3, 0.4),
        Vec4::new(1.7, -2.3, 3.9, -4.1),
        Vec4::new(-0.7, 5.1, 0.02, 9.3),
        Vec4::new(3.3, -1.1, 2.2, 1.0),
    );
    d.v4("mat4 * vec4", mat * a);
    d.m4("mat4 * mat4", mat * mat);
    d.m4("mat4.inverse", mat.inverse());
    d.f("mat4.determinant", mat.determinant());
    d.m4("mat4.transpose", mat.transpose());
    d.v3a("mat4.transform_point3a", mat.transform_point3a(c3));
    d.v3a("mat4.transform_vector3a", mat.transform_vector3a(c3));
    d.v3a(
        "mat4.project_point3",
        Vec3A::from(mat.project_point3(Vec3::from(c3))),
    );

    let m3 = Mat3A::from_cols(c3, d3, Vec3A::new(-0.7, 5.1, 0.02));
    d.v3a("mat3a * vec3a", m3 * c3);
    d.f("mat3a.determinant", m3.determinant());

    d.0.push_str("--- quaternion ---\n");
    let q1 = Quat::from_xyzw(0.1, 0.2, 0.3, 0.4).normalize();
    let q2 = Quat::from_xyzw(-0.5, 0.11, 0.7, 0.2).normalize();
    d.q("quat.normalize q1", q1);
    d.q("quat.normalize q2", q2);
    d.q("quat.mul", q1 * q2);
    d.q("quat.slerp(0.25)", q1.slerp(q2, 0.25));
    d.q("quat.slerp(0.5)", q1.slerp(q2, 0.5));
    d.q("quat.lerp(0.25)", q1.lerp(q2, 0.25));
    d.v3a("quat.mul_vec3a", q1 * c3);
    d.f("quat.dot", q1.dot(q2));
    d.q("quat.from_rotation_x", Quat::from_rotation_x(0.7));
    d.q(
        "quat.from_euler_yxz",
        Quat::from_euler(EulerRot::YXZ, 0.3, 0.4, 0.5),
    );
    d.q(
        "quat.from_rotation_arc",
        Quat::from_rotation_arc(
            Vec3::new(0.2, 0.3, 0.4).normalize(),
            -Vec3::new(0.2, 0.3, 0.4).normalize(),
        ),
    );

    d.0.push_str("--- affine ---\n");
    let af = Affine3A::from_scale_rotation_translation(
        Vec3::new(1.1, 2.2, 0.3),
        q1,
        Vec3::new(-3.0, 4.0, 5.0),
    );
    d.v3a("affine.transform_point3a", af.transform_point3a(c3));
    d.v3a("affine.transform_vector3a", af.transform_vector3a(c3));

    d.0.push_str("--- camera ---\n");
    d.m4(
        "perspective_rh_gl",
        glam::camera::rh::proj::opengl::perspective(0.9, 1.7, 0.1, 1000.0),
    );
    d.m4(
        "look_at_rh",
        glam::camera::rh::view::look_at_mat4(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
        ),
    );

    // The integer vector types. The `-> BVec -> bitmask` path puts the
    // comparison result itself in the file. The `wrapping_*` and
    // `saturating_*` cases sit at the type boundary, where a SIMD body is
    // most likely to differ from the scalar body. See rewrite.md.
    int_common!(
        d, "i8vec2", i8v2, s8, I8Vec2, u8,
        a: [3, -7], b: [5, 2], p: [2, -3],
        clamp: -4, 6, div: [-7, 7], [2, -2],
    );
    int_signed!(
        d, "i8vec2", i8v2, I8Vec2,
        abs: [-5, 0], shr: [-64, -1],
        sb: [i8::MAX, -1], [-1, i8::MAX],
    );
    int_common!(
        d, "i8vec3", i8v3, s8, I8Vec3, u8,
        a: [3, -7, 2], b: [5, 2, -9], p: [2, -3, 1],
        clamp: -4, 6, div: [-7, 7, -8], [2, -2, 3],
    );
    int_signed!(
        d, "i8vec3", i8v3, I8Vec3,
        abs: [-5, 0, 9], shr: [-64, -1, 25],
        sb: [i8::MAX, -1, i8::MIN], [-1, i8::MAX, 0],
    );
    int_common!(
        d, "i8vec4", i8v4, s8, I8Vec4, u8,
        a: [3, -7, 2, -11], b: [5, 2, -9, 4], p: [2, -3, 1, -2],
        clamp: -4, 6, div: [-7, 7, -8, 9], [2, -2, 3, -4],
    );
    int_signed!(
        d, "i8vec4", i8v4, I8Vec4,
        abs: [-5, 0, 9, -1], shr: [-64, -1, 25, -7],
        sb: [i8::MAX, -1, i8::MIN, 0], [-1, i8::MAX, 0, i8::MIN],
    );
    int_common!(
        d, "u8vec2", u8v2, s8, U8Vec2, u8,
        a: [11, 7], b: [3, 2], p: [2, 3],
        clamp: 4, 9, div: [7, 9], [2, 4],
    );
    int_unsigned!(
        d, "u8vec2", u8v2, U8Vec2, u8,
        hb: [u8::MAX, 1], [1, u8::MAX],
    );
    int_common!(
        d, "u8vec3", u8v3, s8, U8Vec3, u8,
        a: [11, 7, 9], b: [3, 2, 4], p: [2, 3, 1],
        clamp: 4, 9, div: [7, 9, 8], [2, 4, 3],
    );
    int_unsigned!(
        d, "u8vec3", u8v3, U8Vec3, u8,
        hb: [u8::MAX, 1, 0x80], [1, u8::MAX, 0x7f],
    );
    int_common!(
        d, "u8vec4", u8v4, s8, U8Vec4, u8,
        a: [11, 7, 9, 5], b: [3, 2, 4, 1], p: [2, 3, 1, 2],
        clamp: 4, 9, div: [7, 9, 8, 5], [2, 4, 3, 2],
    );
    int_unsigned!(
        d, "u8vec4", u8v4, U8Vec4, u8,
        hb: [u8::MAX, 1, 0x80, 0x7f], [1, u8::MAX, 0x7f, 0x80],
    );
    int_common!(
        d, "i16vec2", i16v2, s16, I16Vec2, u16,
        a: [3, -7], b: [5, 2], p: [2, -3],
        clamp: -4, 6, div: [-7, 7], [2, -2],
    );
    int_signed!(
        d, "i16vec2", i16v2, I16Vec2,
        abs: [-5, 0], shr: [-64, -1],
        sb: [i16::MAX, -1], [-1, i16::MAX],
    );
    int_common!(
        d, "i16vec3", i16v3, s16, I16Vec3, u16,
        a: [3, -7, 2], b: [5, 2, -9], p: [2, -3, 1],
        clamp: -4, 6, div: [-7, 7, -8], [2, -2, 3],
    );
    int_signed!(
        d, "i16vec3", i16v3, I16Vec3,
        abs: [-5, 0, 9], shr: [-64, -1, 25],
        sb: [i16::MAX, -1, i16::MIN], [-1, i16::MAX, 0],
    );
    int_common!(
        d, "i16vec4", i16v4, s16, I16Vec4, u16,
        a: [3, -7, 2, -11], b: [5, 2, -9, 4], p: [2, -3, 1, -2],
        clamp: -4, 6, div: [-7, 7, -8, 9], [2, -2, 3, -4],
    );
    int_signed!(
        d, "i16vec4", i16v4, I16Vec4,
        abs: [-5, 0, 9, -1], shr: [-64, -1, 25, -7],
        sb: [i16::MAX, -1, i16::MIN, 0], [-1, i16::MAX, 0, i16::MIN],
    );
    int_common!(
        d, "u16vec2", u16v2, s16, U16Vec2, u16,
        a: [11, 7], b: [3, 2], p: [2, 3],
        clamp: 4, 9, div: [7, 9], [2, 4],
    );
    int_unsigned!(
        d, "u16vec2", u16v2, U16Vec2, u16,
        hb: [u16::MAX, 1], [1, u16::MAX],
    );
    int_common!(
        d, "u16vec3", u16v3, s16, U16Vec3, u16,
        a: [11, 7, 9], b: [3, 2, 4], p: [2, 3, 1],
        clamp: 4, 9, div: [7, 9, 8], [2, 4, 3],
    );
    int_unsigned!(
        d, "u16vec3", u16v3, U16Vec3, u16,
        hb: [u16::MAX, 1, 0x8000], [1, u16::MAX, 0x7fff],
    );
    int_common!(
        d, "u16vec4", u16v4, s16, U16Vec4, u16,
        a: [11, 7, 9, 5], b: [3, 2, 4, 1], p: [2, 3, 1, 2],
        clamp: 4, 9, div: [7, 9, 8, 5], [2, 4, 3, 2],
    );
    int_unsigned!(
        d, "u16vec4", u16v4, U16Vec4, u16,
        hb: [u16::MAX, 1, 0x8000, 0x7fff], [1, u16::MAX, 0x7fff, 0x8000],
    );
    int_common!(
        d, "ivec2", i32v2, s32, IVec2, u32,
        a: [3, -7], b: [5, 2], p: [2, -3],
        clamp: -4, 6, div: [-7, 7], [2, -2],
    );
    int_signed!(
        d, "ivec2", i32v2, IVec2,
        abs: [-5, 0], shr: [-64, -1],
        sb: [i32::MAX, -1], [-1, i32::MAX],
    );
    int_common!(
        d, "ivec3", i32v3, s32, IVec3, u32,
        a: [3, -7, 2], b: [5, 2, -9], p: [2, -3, 1],
        clamp: -4, 6, div: [-7, 7, -8], [2, -2, 3],
    );
    int_signed!(
        d, "ivec3", i32v3, IVec3,
        abs: [-5, 0, 9], shr: [-64, -1, 25],
        sb: [i32::MAX, -1, i32::MIN], [-1, i32::MAX, 0],
    );
    int_common!(
        d, "ivec4", i32v4, s32, IVec4, u32,
        a: [3, -7, 2, -11], b: [5, 2, -9, 4], p: [2, -3, 1, -2],
        clamp: -4, 6, div: [-7, 7, -8, 9], [2, -2, 3, -4],
    );
    int_signed!(
        d, "ivec4", i32v4, IVec4,
        abs: [-5, 0, 9, -1], shr: [-64, -1, 25, -7],
        sb: [i32::MAX, -1, i32::MIN, 0], [-1, i32::MAX, 0, i32::MIN],
    );
    int_common!(
        d, "uvec2", u32v2, s32, UVec2, u32,
        a: [11, 7], b: [3, 2], p: [2, 3],
        clamp: 4, 9, div: [7, 9], [2, 4],
    );
    int_unsigned!(
        d, "uvec2", u32v2, UVec2, u32,
        hb: [u32::MAX, 1], [1, u32::MAX],
    );
    int_common!(
        d, "uvec3", u32v3, s32, UVec3, u32,
        a: [11, 7, 9], b: [3, 2, 4], p: [2, 3, 1],
        clamp: 4, 9, div: [7, 9, 8], [2, 4, 3],
    );
    int_unsigned!(
        d, "uvec3", u32v3, UVec3, u32,
        hb: [u32::MAX, 1, 0x8000_0000], [1, u32::MAX, 0x7fff_ffff],
    );
    int_common!(
        d, "uvec4", u32v4, s32, UVec4, u32,
        a: [11, 7, 9, 5], b: [3, 2, 4, 1], p: [2, 3, 1, 2],
        clamp: 4, 9, div: [7, 9, 8, 5], [2, 4, 3, 2],
    );
    int_unsigned!(
        d, "uvec4", u32v4, UVec4, u32,
        hb: [u32::MAX, 1, 0x8000_0000, 0x7fff_ffff], [1, u32::MAX, 0x7fff_ffff, 0x8000_0000],
    );
    int_common!(
        d, "i64vec2", i64v2, s64, I64Vec2, u64,
        a: [3, -7], b: [5, 2], p: [2, -3],
        clamp: -4, 6, div: [-7, 7], [2, -2],
    );
    int_signed!(
        d, "i64vec2", i64v2, I64Vec2,
        abs: [-5, 0], shr: [-64, -1],
        sb: [i64::MAX, -1], [-1, i64::MAX],
    );
    int_common!(
        d, "i64vec3", i64v3, s64, I64Vec3, u64,
        a: [3, -7, 2], b: [5, 2, -9], p: [2, -3, 1],
        clamp: -4, 6, div: [-7, 7, -8], [2, -2, 3],
    );
    int_signed!(
        d, "i64vec3", i64v3, I64Vec3,
        abs: [-5, 0, 9], shr: [-64, -1, 25],
        sb: [i64::MAX, -1, i64::MIN], [-1, i64::MAX, 0],
    );
    int_common!(
        d, "i64vec4", i64v4, s64, I64Vec4, u64,
        a: [3, -7, 2, -11], b: [5, 2, -9, 4], p: [2, -3, 1, -2],
        clamp: -4, 6, div: [-7, 7, -8, 9], [2, -2, 3, -4],
    );
    int_signed!(
        d, "i64vec4", i64v4, I64Vec4,
        abs: [-5, 0, 9, -1], shr: [-64, -1, 25, -7],
        sb: [i64::MAX, -1, i64::MIN, 0], [-1, i64::MAX, 0, i64::MIN],
    );
    int_common!(
        d, "u64vec2", u64v2, s64, U64Vec2, u64,
        a: [11, 7], b: [3, 2], p: [2, 3],
        clamp: 4, 9, div: [7, 9], [2, 4],
    );
    int_unsigned!(
        d, "u64vec2", u64v2, U64Vec2, u64,
        hb: [u64::MAX, 1], [1, u64::MAX],
    );
    int_common!(
        d, "u64vec3", u64v3, s64, U64Vec3, u64,
        a: [11, 7, 9], b: [3, 2, 4], p: [2, 3, 1],
        clamp: 4, 9, div: [7, 9, 8], [2, 4, 3],
    );
    int_unsigned!(
        d, "u64vec3", u64v3, U64Vec3, u64,
        hb: [u64::MAX, 1, 0x8000_0000_0000_0000], [1, u64::MAX, 0x7fff_ffff_ffff_ffff],
    );
    int_common!(
        d, "u64vec4", u64v4, s64, U64Vec4, u64,
        a: [11, 7, 9, 5], b: [3, 2, 4, 1], p: [2, 3, 1, 2],
        clamp: 4, 9, div: [7, 9, 8, 5], [2, 4, 3, 2],
    );
    int_unsigned!(
        d, "u64vec4", u64v4, U64Vec4, u64,
        hb: [u64::MAX, 1, 0x8000_0000_0000_0000, 0x7fff_ffff_ffff_ffff], [1, u64::MAX, 0x7fff_ffff_ffff_ffff, 0x8000_0000_0000_0000],
    );

    d.0
}

#[test]
fn golden_bits() {
    // The SSE2 and NEON backends share one file, and that is what makes this a
    // cross-processor check. The scalar backend has its own file, and the two
    // files differ on three lines:
    //
    //   * `mat4.inverse`, because the scalar backend keeps its own inverse;
    //   * `quat.mul`, which differs by one ulp in one lane;
    //   * `quat.mul_vec3a`, because the two SIMD backends fuse `w * w - b2`
    //     into one FMA and the scalar backend does not. The scalar
    //     `Quat::mul_vec3` is shared with `DQuat`, so it stays unfused.
    //
    // A fourth difference is a defect. The scalar backend is a test reference
    // and is not shipped. See rewrite.md.
    let name = if cfg!(feature = "scalar-math") {
        "/tests/golden_bits_scalar.txt"
    } else {
        "/tests/golden_bits.txt"
    };
    let path = format!("{}{name}", env!("CARGO_MANIFEST_DIR"));
    let path = path.as_str();
    let actual = dump();

    if std::env::var_os("GLAM_BLESS_GOLDEN").is_some() {
        std::fs::write(path, &actual).unwrap();
        eprintln!("stored new values in {path}");
        return;
    }

    let expected = std::fs::read_to_string(path)
        .expect("tests/golden_bits.txt is missing; run with GLAM_BLESS_GOLDEN=1 to create it");

    if actual != expected {
        let mut report = String::new();
        for (i, (e, a)) in expected.lines().zip(actual.lines()).enumerate() {
            if e != a {
                report.push_str(&format!("line {}:\n  stored: {e}\n  now:    {a}\n", i + 1));
            }
        }
        if expected.lines().count() != actual.lines().count() {
            report.push_str(&format!(
                "line count changed: stored {}, now {}\n",
                expected.lines().count(),
                actual.lines().count()
            ));
        }
        panic!(
            "bit patterns changed. Either the two backends disagree, or the \
             toolchain or a dependency changed a result. See rewrite.md.\n\n{report}"
        );
    }
}
