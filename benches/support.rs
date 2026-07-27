#![allow(dead_code)]
use core::f32;
use glam::{
    I16Vec2, I16Vec3, I16Vec4, I64Vec4, I8Vec2, I8Vec3, I8Vec4, IVec4, Mat2, Mat3, Mat3A, Mat4,
    Quat, U16Vec2, U16Vec3, U16Vec4, U8Vec2, U8Vec3, U8Vec4, UVec4, Vec2, Vec3, Vec3A, Vec4,
};

pub struct PCG32 {
    state: u64,
    inc: u64,
}

impl Default for PCG32 {
    fn default() -> Self {
        PCG32::seed(0x853c49e6748fea9b, 0xda3e39cb94b95bdb)
    }
}

impl PCG32 {
    pub fn seed(initstate: u64, initseq: u64) -> Self {
        let mut rng = PCG32 {
            state: 0,
            inc: (initseq << 1) | 1,
        };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(initstate);
        rng.next_u32();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let oldstate = self.state;
        self.state = oldstate
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.inc | 1);
        let xorshifted = ((oldstate >> 18) ^ oldstate) >> 27;
        let rot = oldstate >> 59;
        ((xorshifted >> rot) | (xorshifted << (rot.wrapping_neg() & 31))) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() & 0xffffff) as f32 / 16777216.0
    }
}

pub fn random_vec2(rng: &mut PCG32) -> Vec2 {
    Vec2::new(rng.next_f32(), rng.next_f32())
}

pub fn random_vec3(rng: &mut PCG32) -> Vec3 {
    Vec3::new(rng.next_f32(), rng.next_f32(), rng.next_f32())
}

pub fn random_vec3a(rng: &mut PCG32) -> Vec3A {
    Vec3A::new(rng.next_f32(), rng.next_f32(), rng.next_f32())
}

pub fn random_vec4(rng: &mut PCG32) -> Vec4 {
    Vec4::new(
        rng.next_f32(),
        rng.next_f32(),
        rng.next_f32(),
        rng.next_f32(),
    )
}

// Integer generators.
//
// The plain operators panic on overflow in a debug build, thus each element
// range is small enough that `add`, `sub`, `mul`, `dot` and `element_sum` all
// stay inside the type. `dot` on a 4-lane vector is the tightest limit,
// because it adds four products: the range half-width `M` must satisfy
// `4 * M * M <= MAX`. This makes the range different for each element type.
// The value of an element has no effect on the speed of an integer add, min,
// max or select, thus the different ranges do not spoil the comparison between
// the narrow types and the 32-bit control types.
//
// An unsigned subtract panics when the left side is smaller than the right
// side. The `_high` generators give the left side of such a benchmark. Their
// smallest value is larger than the largest value of the matching plain
// generator, thus the result is never below zero.

fn next_i8(rng: &mut PCG32) -> i8 {
    // -5..=5, because 4 * 5 * 5 = 100 and i8::MAX is 127.
    (rng.next_u32() % 11) as i8 - 5
}

fn next_u8(rng: &mut PCG32) -> u8 {
    // 0..=7, because 4 * 7 * 7 = 196 and u8::MAX is 255.
    (rng.next_u32() % 8) as u8
}

fn next_u8_high(rng: &mut PCG32) -> u8 {
    // 200..=255. Larger than every value of `next_u8`.
    200 + (rng.next_u32() % 56) as u8
}

fn next_i16(rng: &mut PCG32) -> i16 {
    // -90..=90, because 4 * 90 * 90 = 32400 and i16::MAX is 32767.
    (rng.next_u32() % 181) as i16 - 90
}

fn next_u16(rng: &mut PCG32) -> u16 {
    // 0..=127, because 4 * 127 * 127 = 64516 and u16::MAX is 65535.
    (rng.next_u32() % 128) as u16
}

fn next_u16_high(rng: &mut PCG32) -> u16 {
    // 40000..=65535. Larger than every value of `next_u16`.
    40000 + (rng.next_u32() % 25536) as u16
}

fn next_i32(rng: &mut PCG32) -> i32 {
    // -20000..=20000, because 4 * 20000 * 20000 = 1600000000.
    (rng.next_u32() % 40001) as i32 - 20000
}

fn next_u32_small(rng: &mut PCG32) -> u32 {
    // 0..=32767, because 4 * 32767 * 32767 fits in u32.
    rng.next_u32() % 32768
}

fn next_u32_high(rng: &mut PCG32) -> u32 {
    // 3000000000..=3999999999. Larger than every value of `next_u32_small`.
    3_000_000_000 + rng.next_u32() % 1_000_000_000
}

fn next_i64(rng: &mut PCG32) -> i64 {
    // -1e9..=1e9, because 4 * 1e9 * 1e9 fits in i64.
    (rng.next_u32() as i64 % 2_000_000_001) - 1_000_000_000
}

pub fn random_i8vec2(rng: &mut PCG32) -> I8Vec2 {
    I8Vec2::new(next_i8(rng), next_i8(rng))
}

pub fn random_i8vec3(rng: &mut PCG32) -> I8Vec3 {
    I8Vec3::new(next_i8(rng), next_i8(rng), next_i8(rng))
}

pub fn random_i8vec4(rng: &mut PCG32) -> I8Vec4 {
    I8Vec4::new(next_i8(rng), next_i8(rng), next_i8(rng), next_i8(rng))
}

pub fn random_u8vec2(rng: &mut PCG32) -> U8Vec2 {
    U8Vec2::new(next_u8(rng), next_u8(rng))
}

pub fn random_u8vec3(rng: &mut PCG32) -> U8Vec3 {
    U8Vec3::new(next_u8(rng), next_u8(rng), next_u8(rng))
}

pub fn random_u8vec4(rng: &mut PCG32) -> U8Vec4 {
    U8Vec4::new(next_u8(rng), next_u8(rng), next_u8(rng), next_u8(rng))
}

pub fn random_u8vec4_high(rng: &mut PCG32) -> U8Vec4 {
    U8Vec4::new(
        next_u8_high(rng),
        next_u8_high(rng),
        next_u8_high(rng),
        next_u8_high(rng),
    )
}

pub fn random_i16vec2(rng: &mut PCG32) -> I16Vec2 {
    I16Vec2::new(next_i16(rng), next_i16(rng))
}

pub fn random_i16vec3(rng: &mut PCG32) -> I16Vec3 {
    I16Vec3::new(next_i16(rng), next_i16(rng), next_i16(rng))
}

pub fn random_i16vec4(rng: &mut PCG32) -> I16Vec4 {
    I16Vec4::new(next_i16(rng), next_i16(rng), next_i16(rng), next_i16(rng))
}

pub fn random_u16vec2(rng: &mut PCG32) -> U16Vec2 {
    U16Vec2::new(next_u16(rng), next_u16(rng))
}

pub fn random_u16vec3(rng: &mut PCG32) -> U16Vec3 {
    U16Vec3::new(next_u16(rng), next_u16(rng), next_u16(rng))
}

pub fn random_u16vec4(rng: &mut PCG32) -> U16Vec4 {
    U16Vec4::new(next_u16(rng), next_u16(rng), next_u16(rng), next_u16(rng))
}

pub fn random_u16vec4_high(rng: &mut PCG32) -> U16Vec4 {
    U16Vec4::new(
        next_u16_high(rng),
        next_u16_high(rng),
        next_u16_high(rng),
        next_u16_high(rng),
    )
}

pub fn random_ivec4(rng: &mut PCG32) -> IVec4 {
    IVec4::new(next_i32(rng), next_i32(rng), next_i32(rng), next_i32(rng))
}

pub fn random_uvec4(rng: &mut PCG32) -> UVec4 {
    UVec4::new(
        next_u32_small(rng),
        next_u32_small(rng),
        next_u32_small(rng),
        next_u32_small(rng),
    )
}

pub fn random_uvec4_high(rng: &mut PCG32) -> UVec4 {
    UVec4::new(
        next_u32_high(rng),
        next_u32_high(rng),
        next_u32_high(rng),
        next_u32_high(rng),
    )
}

pub fn random_i64vec4(rng: &mut PCG32) -> I64Vec4 {
    I64Vec4::new(next_i64(rng), next_i64(rng), next_i64(rng), next_i64(rng))
}

pub fn random_nonzero_vec2(rng: &mut PCG32) -> Vec2 {
    loop {
        let v = random_vec2(rng);
        if v.length_squared() > 0.01 {
            return v;
        }
    }
}

pub fn random_nonzero_vec3(rng: &mut PCG32) -> Vec3 {
    loop {
        let v = random_vec3(rng);
        if v.length_squared() > 0.01 {
            return v;
        }
    }
}

pub fn random_f32(rng: &mut PCG32) -> f32 {
    rng.next_f32()
}

pub fn random_radians(rng: &mut PCG32) -> f32 {
    -f32::consts::PI + rng.next_f32() * 2.0 * f32::consts::PI
}

pub fn random_quat(rng: &mut PCG32) -> Quat {
    let yaw = random_radians(rng);
    let pitch = random_radians(rng);
    let roll = random_radians(rng);
    Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, roll)
}

pub fn random_mat2(rng: &mut PCG32) -> Mat2 {
    Mat2::from_cols(random_vec2(rng), random_vec2(rng))
}

pub fn random_mat3(rng: &mut PCG32) -> Mat3 {
    Mat3::from_cols(random_vec3(rng), random_vec3(rng), random_vec3(rng))
}

pub fn random_srt_mat3(rng: &mut PCG32) -> Mat3 {
    Mat3::from_scale_angle_translation(
        random_nonzero_vec2(rng),
        random_radians(rng),
        random_vec2(rng),
    )
}

pub fn random_mat3a(rng: &mut PCG32) -> Mat3A {
    Mat3A::from_cols(random_vec3a(rng), random_vec3a(rng), random_vec3a(rng))
}

pub fn random_srt_mat3a(rng: &mut PCG32) -> Mat3A {
    Mat3A::from_scale_angle_translation(
        random_nonzero_vec2(rng),
        random_radians(rng),
        random_vec2(rng),
    )
}

pub fn random_srt_mat4(rng: &mut PCG32) -> Mat4 {
    Mat4::from_scale_rotation_translation(
        random_nonzero_vec3(rng),
        random_quat(rng),
        random_vec3(rng),
    )
}
