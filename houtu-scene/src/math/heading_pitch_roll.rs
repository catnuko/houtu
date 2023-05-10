use super::*;
use bevy::math::DQuat;
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct HeadingPitchRoll {
    pub heading: f64,
    pub pitch: f64,
    pub roll: f64,
}
impl HeadingPitchRoll {
    fn new(heading: f64, pitch: f64, roll: f64) -> Self {
        return Self {
            heading: heading,
            pitch: pitch,
            roll: roll,
        };
    }
    fn from_quaternion(quaternion: DQuat) -> Self {
        let mut result = HeadingPitchRoll::default();
        let mut test = 2. * (quaternion.w * quaternion.y - quaternion.z * quaternion.x);
        let mut denominatorRoll =
            1. - 2. * (quaternion.x * quaternion.x + quaternion.y * quaternion.y);
        let mut numeratorRoll = 2. * (quaternion.w * quaternion.x + quaternion.y * quaternion.z);
        let mut denominatorHeading =
            1.0 - 2. * (quaternion.y * quaternion.y + quaternion.z * quaternion.z);
        let mut numeratorHeading = 2. * (quaternion.w * quaternion.z + quaternion.x * quaternion.y);
        result.heading = -1.0 * numeratorHeading.atan2(denominatorHeading);
        result.roll = numeratorRoll.atan2(denominatorRoll);
        result.pitch = -1.0 * test.clamp(-1.0, 1.0).asin();
        return result;
    }
    fn from_degrees(heading: f64, pitch: f64, roll: f64) -> Self {
        return Self {
            heading: heading.to_radians(),
            pitch: pitch.to_radians(),
            roll: roll.to_radians(),
        };
    }
    fn from_radians(heading: f64, pitch: f64, roll: f64) -> Self {
        return Self {
            heading: heading,
            pitch: pitch,
            roll: roll,
        };
    }
    fn equals_epsilon(
        &self,
        right: HeadingPitchRoll,
        relative_epsilon: Option<f64>,
        absolute_epsilon: Option<f64>,
    ) -> bool {
        return self.eq(&right)
            || equals_epsilon(
                self.heading,
                right.heading,
                relative_epsilon,
                absolute_epsilon,
            ) && equals_epsilon(self.pitch, right.pitch, relative_epsilon, absolute_epsilon)
                && equals_epsilon(self.roll, right.roll, relative_epsilon, absolute_epsilon);
    }
}
#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use bevy::math::DVec3;

    use super::*;
    #[test]
    fn test_from_quaternion() {
        let deg2rad = RADIANS_PER_DEGREE;

        let testingTab: Vec<DVec3> = [
            [0., 0., 0.],
            [90. * deg2rad, 0., 0.],
            [-90. * deg2rad, 0., 0.],
            [0., 89. * deg2rad, 0.],
            [0., -89. * deg2rad, 0.],
            [0., 0., 90. * deg2rad],
            [0., 0., -90. * deg2rad],
            [30. * deg2rad, 30. * deg2rad, 30. * deg2rad],
            [-30. * deg2rad, -30. * deg2rad, 45. * deg2rad],
        ]
        .iter()
        .map(|x| DVec3::from_array(x.clone()))
        .collect();
        let mut hpr = HeadingPitchRoll::default();
        for i in 0..testingTab.len() {
            let init = testingTab[i];
            hpr.heading = init[0];
            hpr.pitch = init[1];
            hpr.roll = init[2];

            let result = HeadingPitchRoll::from_quaternion(DQuat::from_heading_pitch_roll(hpr));
            assert!(equals_epsilon(
                init[0],
                result.heading,
                Some(EPSILON11),
                None
            ));
            assert!(equals_epsilon(init[1], result.pitch, Some(EPSILON11), None));
            assert!(equals_epsilon(init[2], result.roll, Some(EPSILON11), None));
        }
    }
}
