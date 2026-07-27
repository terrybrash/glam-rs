macro_rules! deps {
    () => {
        "arbitrary approx bytemuck encase mint rand rkyv bytecheck serde speedy zerocopy debug-glam-assert"
    };
}

pub(crate) const FEATURE_SETS: &[&str] = &[
    // The default build.
    "all-types",
    // The scalar reference used by the differential test. Never shipped.
    "all-types scalar-math",
    // The integrations, on their own and with every type.
    deps!(),
    concat!("all-types ", deps!()),
    concat!("all-types scalar-math ", deps!()),
    // Assertions on in a release build.
    concat!("all-types glam-assert ", deps!()),
    // A narrow type set, to check that the per-type features still compile.
    "f64",
    "i32 u32",
];

// A reduced set. Some optional dependencies need a newer rustc.
pub(crate) const MSRV_FEATURES: &str = "all-types arbitrary approx mint speedy debug-glam-assert";

// All optional deps used by clippy, doc, and coverage
pub(crate) const ALL_FEATURES: &str = deps!();

pub fn resolve_sets(index: Option<usize>) -> &'static [&'static str] {
    match index {
        Some(i) => {
            if i == 0 || i > FEATURE_SETS.len() {
                panic!(
                    "feature set index {i} is out of range (1-{})",
                    FEATURE_SETS.len()
                );
            }
            &FEATURE_SETS[i - 1..i]
        }
        None => FEATURE_SETS,
    }
}

pub fn print_feature_sets() {
    for (i, features) in FEATURE_SETS.iter().enumerate() {
        println!("  {}. {features}", i + 1);
    }
}
