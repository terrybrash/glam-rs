#[cfg(feature = "f64")]
mod dvec2_impl;
#[cfg(feature = "f64")]
mod dvec3_impl;
#[cfg(feature = "f64")]
mod dvec4_impl;

#[cfg(feature = "i32")]
mod ivec2_impl;
#[cfg(feature = "i32")]
mod ivec3_impl;
#[cfg(feature = "i32")]
mod ivec4_impl;

#[cfg(feature = "i8")]
mod i8vec2_impl;
#[cfg(feature = "i8")]
mod i8vec3_impl;
#[cfg(feature = "i8")]
mod i8vec4_impl;

#[cfg(feature = "u8")]
mod u8vec2_impl;
#[cfg(feature = "u8")]
mod u8vec3_impl;
#[cfg(feature = "u8")]
mod u8vec4_impl;

#[cfg(feature = "i16")]
mod i16vec2_impl;
#[cfg(feature = "i16")]
mod i16vec3_impl;
#[cfg(feature = "i16")]
mod i16vec4_impl;

#[cfg(feature = "u16")]
mod u16vec2_impl;
#[cfg(feature = "u16")]
mod u16vec3_impl;
#[cfg(feature = "u16")]
mod u16vec4_impl;

#[cfg(feature = "i64")]
mod i64vec2_impl;
#[cfg(feature = "i64")]
mod i64vec3_impl;
#[cfg(feature = "i64")]
mod i64vec4_impl;

#[cfg(feature = "u64")]
mod u64vec2_impl;
#[cfg(feature = "u64")]
mod u64vec3_impl;
#[cfg(feature = "u64")]
mod u64vec4_impl;

#[cfg(feature = "u32")]
mod uvec2_impl;
#[cfg(feature = "u32")]
mod uvec3_impl;
#[cfg(feature = "u32")]
mod uvec4_impl;

mod vec2_impl;
mod vec3_impl;

#[cfg(any(
    not(any(target_arch = "aarch64", target_arch = "x86_64")),
    feature = "scalar-math"
))]
mod scalar;

#[cfg(all(target_arch = "aarch64", not(feature = "scalar-math")))]
mod aarch64;

#[cfg(all(target_arch = "x86_64", not(feature = "scalar-math")))]
mod x86_64;

mod vec_traits;
pub use vec_traits::*;
