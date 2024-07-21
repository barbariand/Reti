//! All the difrent enablable constants that can be used in Reti
#![allow(clippy::excessive_precision)]

///Mathematical constants
pub mod math_constants {
    pub use std::f64::consts::{E, LN_2, PI, SQRT_2, TAU};
    /// The Euler-Mascheroni constant (γ)
    pub const EGAMMA: f64 = 0.577215664901532860606512090082402431_f64;
    /// The golden ratio(φ)
    pub const PHI: f64 = 1.618033988749894848204586834365638118_f64;
}
///SI physics constants
pub mod physics_si {
    ///Universal physical constants
    pub mod universal_constants {
        /// Speed of light in m/s
        pub const C: f64 = 299792458.0;
        /// Planks constant
        pub const H: f64 = 6.626_070_15e-34;
        /// Newtonian constant of gravitation
        /// https://physics.nist.gov/cgi-bin/cuu/Value?bg
        pub const G: f64 = 6.674_30e-11;
        /// Mass of electron
        pub const M_ELECTRON: f64 = 9.109_383_713_9e-31;
        /// Mass of muon
        pub const M_MUON: f64 = 1.883_531_627e-28;
        /// Mass of tau lepton
        pub const M_TAU: f64 = 3.167_54e-27;
        /// Mass of proton
        pub const M_PROTON: f64 = 1.672_621_925_95e-27;
        /// Mass of neutron
        pub const M_NEUTRON: f64 = 1.674_927_500_56e-25;
    }
    ///chemistry constants
    pub mod chem {
        /// Avogados Constant
        pub const N_A: f64 = 6.022_140_76e23;
    }
    ///gravitational constants in meter per second squared
    pub mod gravitation {
        /// The standard definition as given by [Wikipedia/StandardGravity](https://en.wikipedia.org/wiki/Standard_gravity)
        pub const STANDARD_DEFINITION: f64 = 9.80665;
        /// The equatorial definition of gravity given by
        /// Moritz, H. Geodetic Reference System 1980. Journal of Geodesy 74,
        /// 128–133 (2000). [https://doi.org/10.1007/s001900050278](https://doi.org/10.1007/s001900050278)
        pub const EQUATOR: f64 = 9.780_326_771_5;
        /// Sweden's simple definition of gravity
        pub const SWEDEN: f64 = POINT82;
        /// USAs simple definition of gravity
        pub const USA: f64 = POINT82;
        /// 9.78m/s²
        pub const POINT78: f64 = 9.78;
        /// 9.79m/s²
        pub const POINT79: f64 = 9.79;
        /// 9.80m/s²
        pub const POINT80: f64 = 9.80;
        /// 9.81m/s²
        pub const POINT81: f64 = 9.81;
        /// 9.82m/s²
        pub const POINT82: f64 = 9.82;
        /// 9.83m/s²
        pub const POINT83: f64 = 9.83;
        /// 10.0m/s²
        pub const SIMPLE: f64 = 10.0;
    }
}
/// macro for conversion with fixed ratio not used because we don't support
/// imperial rn
#[allow(unused)]
macro_rules! covert_using_ratio {
    ($conversion:path, { $($name:ident => $si:path),* $(,)? }) => {
        $(
            ///Auto converted constant from
            ///[
            #[doc = stringify!($si)]
            ///] using [
            #[doc= stringify!($conversion)]
            /// ]
            pub const $name:f64=const{$si*$conversion};
        )*
    };
}
