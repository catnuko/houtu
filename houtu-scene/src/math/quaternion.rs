use bevy::math::{DQuat, DVec3};

use super::{Cartesian3, HeadingPitchRoll};

pub trait Quaternion {
    fn from_heading_pitch_roll(hpr: HeadingPitchRoll) -> DQuat;
}
impl Quaternion for DQuat {
    fn from_heading_pitch_roll(hpr: HeadingPitchRoll) -> DQuat {
        let scratchRollQuaternion = DQuat::from_axis_angle(DVec3::UNIT_X, hpr.roll);
        let scratchPitchQuaternion = DQuat::from_axis_angle(DVec3::UNIT_Y, -hpr.pitch);
        let result = scratchPitchQuaternion * scratchRollQuaternion;
        let scratchHeadingQuaternion = DQuat::from_axis_angle(DVec3::UNIT_Z, -hpr.heading);
        return scratchHeadingQuaternion * result;
    }
}
