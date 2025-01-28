use crate::utils::RealFieldCopy;
use crate::ENU;
use core::convert::From;
use core::ops::Neg;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

/// Local azimuth-elevation-range (AER) spherical coordinates
///
/// This struct represents the horizontal (azimuth) and vertical (elevation) angels
/// and distance (slant-range) in the local geodetic coordinate system.
///
/// The `serde` feature allows this to be Serialized / Deserialized.
/// If serialized into json, it will look like this. Enabled thought
/// the `serde` feature
/// ```json
/// {
///    "azimuth": 0.0,
///    "elevation": 0.0,
///    "range": 0.0
/// }
/// ```
/// Note: azimuth and elevation values will be in radians
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct AER<N> {
    // Represented as radians
    azimuth: N,
    // Represented as radians
    elevation: N,
    range: N,
}

impl<N: RealFieldCopy> AER<N>
where
    f64: From<N>,
{
    /// Create a new AER vector
    ///
    /// # Arguments
    /// - `azimuth` in degrees (0 to 360) TODO: check if clamping is required
    /// - `elevation` in degrees (-90 to 90)
    /// - `range` in meters TODO:, must be positive?
    ///
    pub fn from_degrees_and_meters(azimuth: N, elevation: N, range: N) -> AER<N> {
        AER {
            azimuth: N::from_f64(f64::from(azimuth).to_radians()).unwrap(),
            elevation: N::from_f64(f64::from(elevation).to_radians()).unwrap(),
            range,
        }
    }

    /// Create a new AER vector
    ///
    /// # Arguments
    /// - `azimuth` in radians
    /// - `elevation` in radians
    /// - `range` in meters
    ///
    pub fn from_radians_and_meters(azimuth: N, elevation: N, range: N) -> AER<N> {
        AER {
            azimuth,
            elevation,
            range,
        }
    }

    /// Get azimuth in degrees
    pub fn azimuth_degrees(&self) -> N {
        N::from_f64(f64::from(self.azimuth).to_degrees()).unwrap()
    }

    /// Get elevation in degrees
    pub fn elevation_degrees(&self) -> N {
        N::from_f64(f64::from(self.elevation).to_degrees()).unwrap()
    }

    // Get azimuth in radians
    pub fn azimuth_radians(&self) -> N {
        self.azimuth
    }

    /// Get elevation in radians
    pub fn elevation_radians(&self) -> N {
        self.elevation
    }

    /// Get range in meters
    pub fn range(&self) -> N {
        N::from_f64(f64::from(self.range).to_degrees()).unwrap()
    }
}

impl<N: RealFieldCopy> From<ENU<N>> for AER<N>
where
    f64: From<N>,
{
    fn from(enu: ENU<N>) -> Self {
        let horizontal_distance = N::from_f64(f64::sqrt(
            f64::from(enu.east()).powi(2) + f64::from(enu.north()).powi(2),
        ))
        .unwrap();
        AER {
            azimuth: enu.east().atan2(enu.north()),
            elevation: enu.up().atan2(horizontal_distance),
            range: enu.norm(),
        }
    }
}

impl<N: RealFieldCopy + Neg<Output = N>> From<AER<N>> for ENU<N> {
    fn from(aer: AER<N>) -> ENU<N> {
        ENU::new(
            aer.azimuth.sin() * aer.elevation.cos() * aer.range,
            aer.azimuth.cos() * aer.elevation.cos() * aer.range,
            aer.elevation.sin() * aer.range,
        )
    }
}

#[cfg(test)]
impl Arbitrary for AER<f64> {
    fn arbitrary<G: Gen>(g: &mut G) -> AER<f64> {
        use rand::Rng;
        let azimuth = g.gen_range(-360.0, 360.0);
        let elevation = g.gen_range(-90.0, 90.0);
        let range = g.gen_range(0.0, 10000000.0);

        AER::from_degrees_and_meters(azimuth, elevation, range)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::enu::ENU;
    use assert::close;

    #[test]
    #[cfg_attr(not(feature = "serde"), ignore)]
    fn test_ser_de() {
        #[cfg(feature = "serde")]
        {
            use serde_test::{assert_tokens, Token};
            let aer: AER<f64> = AER {
                azimuth: 2.0,
                elevation: 1.0,
                range: 3.0,
            };
            assert_tokens(
                &aer,
                &[
                    Token::Struct {
                        name: "AER",
                        len: 3,
                    },
                    Token::Str("azimuth"),
                    Token::F64(2.0),
                    Token::Str("elevation"),
                    Token::F64(1.0),
                    Token::Str("range"),
                    Token::F64(3.0),
                    Token::StructEnd,
                ],
            );
        }
        #[cfg(not(feature = "serde"))]
        {
            panic!("This test requires the serde feature to be enabled");
        }
    }

    #[test]
    fn double_conversion_is_identity() {
        let enu = ENU::new(1.0, 2.0, 3.0);
        let double_conversion: ENU<f64> = ENU::from(AER::from(enu));
        close(enu.east(), double_conversion.east(), 0.0001);
        close(enu.north(), double_conversion.north(), 0.0001);
        close(enu.up(), double_conversion.up(), 0.0001);
        close(enu.norm(), double_conversion.norm(), 0.0001);

        let aer = AER::from_degrees_and_meters(10.0, 20.0, 30.0);
        let double_conversion = AER::from(ENU::from(aer));
        close(
            aer.azimuth_degrees(),
            double_conversion.azimuth_degrees(),
            0.0001,
        );
        close(
            aer.elevation_degrees(),
            double_conversion.elevation_degrees(),
            0.0001,
        );
        close(aer.range(), double_conversion.range(), 0.0001);
    }

    #[test]
    fn known_good_enu_conversion() {
        //values taken from matlab aer2enu: https://mathworks.com/help/map/ref/aer2enu.html
        let azimuth = 34.1160_f64.to_radians();
        let elevation = 4.1931_f64.to_radians();
        let range = 15.1070;
        let x_east = 8.4504;
        let y_north = 12.4737;
        let z_up = 1.1046;

        let enu = ENU::new(x_east, y_north, z_up);
        let aer = AER {
            azimuth,
            elevation,
            range,
        };
        close(enu.east(), ENU::from(aer).east(), 0.001);
        close(enu.north(), ENU::from(aer).north(), 0.001);
        close(enu.up(), ENU::from(aer).up(), 0.001);
        close(
            aer.azimuth_radians(),
            AER::from(enu).azimuth_radians(),
            0.001,
        );
        close(
            aer.elevation_radians(),
            AER::from(enu).elevation_radians(),
            0.001,
        );
        close(aer.range(), AER::from(enu).range(), 0.01);
    }
}
