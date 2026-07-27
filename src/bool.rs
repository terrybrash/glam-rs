mod bvec2;
mod bvec3;
mod bvec4;

#[cfg(all(target_arch = "aarch64", not(feature = "scalar-math")))]
mod aarch64;

#[cfg(all(target_arch = "x86_64", not(feature = "scalar-math")))]
mod x86_64;

#[cfg(any(
    not(any(target_arch = "aarch64", target_arch = "x86_64")),
    feature = "scalar-math"
))]
mod scalar;

pub use bvec2::{bvec2, BVec2};
pub use bvec3::{bvec3, BVec3};
pub use bvec4::{bvec4, BVec4};

#[cfg(all(target_arch = "aarch64", not(feature = "scalar-math")))]
pub use aarch64::bvec3a::{bvec3a, BVec3A};
#[cfg(all(target_arch = "aarch64", not(feature = "scalar-math")))]
pub use aarch64::bvec4a::{bvec4a, BVec4A};

#[cfg(all(target_arch = "x86_64", not(feature = "scalar-math")))]
pub use x86_64::bvec3a::{bvec3a, BVec3A};
#[cfg(all(target_arch = "x86_64", not(feature = "scalar-math")))]
pub use x86_64::bvec4a::{bvec4a, BVec4A};

#[cfg(any(
    not(any(target_arch = "aarch64", target_arch = "x86_64")),
    feature = "scalar-math"
))]
pub use scalar::bvec3a::{bvec3a, BVec3A};

#[cfg(any(
    not(any(target_arch = "aarch64", target_arch = "x86_64")),
    feature = "scalar-math"
))]
pub use scalar::bvec4a::{bvec4a, BVec4A};

mod const_test_bvec2 {
    const_assert_eq!(1, core::mem::align_of::<super::BVec2>());
    const_assert_eq!(2, core::mem::size_of::<super::BVec2>());
}

mod const_test_bvec3 {
    const_assert_eq!(1, core::mem::align_of::<super::BVec3>());
    const_assert_eq!(3, core::mem::size_of::<super::BVec3>());
}

mod const_test_bvec4 {
    const_assert_eq!(1, core::mem::align_of::<super::BVec4>());
    const_assert_eq!(4, core::mem::size_of::<super::BVec4>());
}

#[cfg(not(feature = "scalar-math"))]
mod const_test_bvec3a {
    const_assert_eq!(16, core::mem::align_of::<super::BVec3A>());
    const_assert_eq!(16, core::mem::size_of::<super::BVec3A>());
}

#[cfg(not(feature = "scalar-math"))]
mod const_test_bvec4a {
    const_assert_eq!(16, core::mem::align_of::<super::BVec4A>());
    const_assert_eq!(16, core::mem::size_of::<super::BVec4A>());
}
