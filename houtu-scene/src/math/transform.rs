use bevy::math::{DMat4, DVec3};
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
pub fn localFrameToFixedFrameGenerator(
    firstAxis: &'static str,
    secondAxis: &'static str,
) -> impl Fn(DVec3, Option<Ellipsoid>) -> DMat4 {
    let json = r#"
{
    "up": {
      "south": "east",
      "north": "west",
      "west": "south",
      "east": "north"
    },
    "down": {
      "south": "west",
      "north": "east",
      "west": "north",
      "east": "south"
    },
    "south": {
      "up": "west",
      "down": "east",
      "west": "down",
      "east": "up"
    },
    "north": {
      "up": "east",
      "down": "west",
      "west": "up",
      "east": "down"
    },
    "west": {
      "up": "north",
      "down": "south",
      "north": "down",
      "south": "up"
    },
    "east": {
      "up": "south",
      "down": "north",
      "north": "up",
      "south": "down"
    }
  }
"#;
    let vectorProductLocalFrame: Value = serde_json::from_str(json).unwrap();
    let json2 = r#"
{
    "north": [-1, 0, 0],
    "east": [0, 1, 0],
    "up": [0, 0, 1],
    "south": [1, 0, 0],
    "west": [0, -1, 0],
    "down": [0, 0, -1]
  }
"#;
    let degeneratePositionLocalFrame: Test = serde_json::from_str(json2).unwrap();
    let thirdAxis = &vectorProductLocalFrame[firstAxis][secondAxis];
    return move |origin: DVec3, ellipsoid: Option<Ellipsoid>| -> DMat4 {
        let mut result = DMat4::default();
        let mut scratchFirstCartesian = DVec3::default();
        let mut scratchSecondCartesian = DVec3::default();
        let mut scratchThirdCartesian = DVec3::default();
        // let b = &degeneratePositionLocalFrame[firstAxis];
        println!("b: {:?}", &degeneratePositionLocalFrame);
        let mut scratchCalculateCartesian_east = DVec3::default();
        let mut scratchCalculateCartesian_north = DVec3::default();
        let mut scratchCalculateCartesian_up = DVec3::default();
        let mut scratchCalculateCartesian_west = DVec3::default();
        let mut scratchCalculateCartesian_south = DVec3::default();
        let mut scratchCalculateCartesian_down = DVec3::default();

        // if (origin.equals_epsilon(DVec3::ZERO, Some(EPSILON14), None)) {
        //     // If x, y, and z are zero, use the degenerate local frame, which is a special case
        //     scratchFirstCartesian = DVec3::from_array(degeneratePositionLocalFrame[firstAxis]);
        //     scratchSecondCartesian = DVec3::from_array(degeneratePositionLocalFrame[secondAxis]);
        //     scratchThirdCartesian = DVec3::from_array(degeneratePositionLocalFrame[thirdAxis]);
        // } else if (equals_epsilon(origin.x, 0.0, Some(EPSILON14), None)
        //     && equals_epsilon(origin.y, 0.0, Some(EPSILON14), None))
        // {
        //     // If x and y are zero, assume origin is at a pole, which is a special case.
        //     let sign = origin.z.signum();
        //     scratchFirstCartesian = DVec3::from_array(degeneratePositionLocalFrame[firstAxis]);
        //     if firstAxis != "east" && firstAxis != "west" {
        //         scratchFirstCartesian = scratchFirstCartesian.multiply_by_scalar(sign);
        //     }
        //     scratchSecondCartesian = DVec3::from_array(degeneratePositionLocalFrame[secondAxis]);
        //     if secondAxis != "east" && secondAxis != "west" {
        //         scratchSecondCartesian = scratchSecondCartesian.multiply_by_scalar(sign);
        //     }
        //     scratchThirdCartesian = DVec3::from_array(degeneratePositionLocalFrame[thirdAxis]);
        //     if thirdAxis != "east" && thirdAxis != "west" {
        //         scratchThirdCartesian = scratchThirdCartesian.multiply_by_scalar(sign);
        //     }
        // } else {
        //     let ellipsoid = ellipsoid.unwrap_or(Ellipsoid::WGS84);
        //     scratchCalculateCartesian_up = ellipsoid.geodeticSurfaceNormal(&origin);

        //     let mut up = scratchCalculateCartesian_up;
        //     let mut east = scratchCalculateCartesian_east;
        //     east.x = -origin.y;
        //     east.y = origin.x;
        //     east.z = 0.0;
        //     scratchCalculateCartesian_east = east.normalize();
        //     scratchCalculateCartesian_north = up.cross(east);
        //     scratchCalculateCartesian_down = scratchCalculateCartesian_up.multiply_by_scalar(-1.0);
        //     scratchCalculateCartesian_west =
        //         scratchCalculateCartesian_east.multiply_by_scalar(-1.0);
        //     scratchCalculateCartesian_south =
        //         scratchCalculateCartesian_north.multiply_by_scalar(-1.0);

        //     scratchFirstCartesian = scratchCalculateCartesian[firstAxis];
        //     scratchSecondCartesian = scratchCalculateCartesian[secondAxis];
        //     scratchThirdCartesian = scratchCalculateCartesian[thirdAxis];
        // }
        // result[0] = scratchFirstCartesian.x;
        // result[1] = scratchFirstCartesian.y;
        // result[2] = scratchFirstCartesian.z;
        // result[3] = 0.0;
        // result[4] = scratchSecondCartesian.x;
        // result[5] = scratchSecondCartesian.y;
        // result[6] = scratchSecondCartesian.z;
        // result[7] = 0.0;
        // result[8] = scratchThirdCartesian.x;
        // result[9] = scratchThirdCartesian.y;
        // result[10] = scratchThirdCartesian.z;
        // result[11] = 0.0;
        // result[12] = origin.x;
        // result[13] = origin.y;
        // result[14] = origin.z;
        // result[15] = 1.0;
        return result;
    };
}
#[cfg(test)]
mod tests {
    use bevy::math::{DVec3, DVec4};

    use super::*;
    #[test]
    fn test_init() {
        let origin = DVec3::new(0.0, 0.0, 1.0);
        let localFrameToFixedFrame = localFrameToFixedFrameGenerator("east", "north");
        let result = localFrameToFixedFrame(origin, None);
        let negativeX = DVec4::new(-1., 0., 0., 0.);
        let negativeY = DVec4::new(0., -1., 0., 0.);
        let negativeZ = DVec4::new(0., 0., -1., 0.);
        println!("{:?}", result);
        assert_eq!(result.col(0).eq(&DVec4::new(0., 1., 0., 0.)), true);
        assert_eq!(result.col(1).eq(&negativeX), true);
        assert_eq!(result.col(2).eq(&DVec4::new(0., 0., 1., 0.)), true);
    }
}
