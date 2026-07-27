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
use std::fmt::Write;

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
        Quat::from_rotation_arc(Vec3::new(0.2, 0.3, 0.4).normalize(), -Vec3::new(0.2, 0.3, 0.4).normalize()),
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
