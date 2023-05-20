use bevy::math::{DMat4, DQuat, DVec3, DVec4};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::*;
use crate::ellipsoid::{self, Ellipsoid};

#[derive(Serialize, Deserialize, Debug)]
struct Test {
    north: DVec3,
    east: DVec3,
    up: DVec3,
    south: DVec3,
    west: DVec3,
    down: DVec3,
}
pub fn eastNorthUpToFixedFrame(origin: DVec3, ellipsoid: Option<Ellipsoid>) -> DMat4 {
    let mut scratchFirstCartesian = DVec3::default();
    let mut scratchSecondCartesian = DVec3::default();
    let mut scratchThirdCartesian = DVec3::default();
    let mut scratchCalculateCartesian_east = DVec3::default();
    let mut scratchCalculateCartesian_north = DVec3::default();
    let mut scratchCalculateCartesian_up = DVec3::default();
    let mut scratchCalculateCartesian_west = DVec3::default();
    let mut scratchCalculateCartesian_south = DVec3::default();
    let mut scratchCalculateCartesian_down = DVec3::default();

    if (origin.equals_epsilon(DVec3::ZERO, Some(EPSILON14), None)) {
        // If x, y, and z are zero, use the degenerate local frame, which is a special case
        scratchFirstCartesian = DVec3::from_array([0., 1., 0.]);
        scratchSecondCartesian = DVec3::from_array([-1., 0., 0.]);
        scratchThirdCartesian = DVec3::from_array([0., 0., 1.]);
    } else if (equals_epsilon(origin.x, 0.0, Some(EPSILON14), None)
        && equals_epsilon(origin.y, 0.0, Some(EPSILON14), None))
    {
        // If x and y are zero, assume origin is at a pole, which is a special case.
        scratchFirstCartesian = DVec3::from_array([0., 1., 0.]);
        scratchSecondCartesian = DVec3::from_array([-1., 0., 0.]);
        scratchThirdCartesian = DVec3::from_array([0., 0., 1.]);
    } else {
        let ellipsoid = ellipsoid.unwrap_or(Ellipsoid::WGS84);
        scratchCalculateCartesian_up = ellipsoid.geodeticSurfaceNormal(&origin).unwrap();

        let mut up = scratchCalculateCartesian_up;
        let mut east = scratchCalculateCartesian_east;
        east.x = -origin.y;
        east.y = origin.x;
        east.z = 0.0;
        scratchCalculateCartesian_east = east.normalize();
        east = scratchCalculateCartesian_east.clone();
        scratchCalculateCartesian_north = up.cross(east);
        scratchCalculateCartesian_down = scratchCalculateCartesian_up.multiply_by_scalar(-1.0);
        scratchCalculateCartesian_west = scratchCalculateCartesian_east.multiply_by_scalar(-1.0);
        scratchCalculateCartesian_south = scratchCalculateCartesian_north.multiply_by_scalar(-1.0);

        scratchFirstCartesian = scratchCalculateCartesian_east;
        scratchSecondCartesian = scratchCalculateCartesian_north;
        scratchThirdCartesian = scratchCalculateCartesian_up;
    }
    let result = DMat4::from_cols_array(&[
        scratchFirstCartesian.x,
        scratchFirstCartesian.y,
        scratchFirstCartesian.z,
        0.,
        scratchSecondCartesian.x,
        scratchSecondCartesian.y,
        scratchSecondCartesian.z,
        0.,
        scratchThirdCartesian.x,
        scratchThirdCartesian.y,
        scratchThirdCartesian.z,
        0.,
        origin.x,
        origin.y,
        origin.z,
        1.0,
    ]);
    return result;
}
fn headingPitchRollToFixedFrame(
    origin: DVec3,
    headingPitchRoll: HeadingPitchRoll,
    ellipsoid: Option<Ellipsoid>,
) -> DMat4 {
    let ellipsoid = ellipsoid.unwrap_or(Ellipsoid::WGS84);
    let fixedFrameTransform = eastNorthUpToFixedFrame;
    let hprQuaternion = DQuat::from_heading_pitch_roll(headingPitchRoll);

    let hprMatrix = DMat4::from_scale_rotation_translation(
        DVec3::ZERO,
        hprQuaternion,
        DVec3 {
            x: 1.,
            y: 1.,
            z: 1.,
        },
    );
    return fixedFrameTransform(origin, Some(ellipsoid)).mul_mat4(&hprMatrix);
}
#[cfg(test)]
mod tests {
    use bevy::math::{DVec3, DVec4};

    use super::*;
    #[test]
    fn test_init() {
        let origin = DVec3::new(0.0, 0.0, 1.0);
        let result = eastNorthUpToFixedFrame(origin, None);
        let negativeX = DVec4::new(-1., 0., 0., 0.);
        let negativeY = DVec4::new(0., -1., 0., 0.);
        let negativeZ = DVec4::new(0., 0., -1., 0.);
        println!("{:?}", result);
        assert_eq!(result.col(0).eq(&DVec4::new(0., 1., 0., 0.)), true);
        assert_eq!(result.col(1).eq(&negativeX), true);
        assert_eq!(result.col(2).eq(&DVec4::new(0., 0., 1., 0.)), true);
    }
}
