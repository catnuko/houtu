use std::ops::Sub;

use bevy::math::{DMat4, DVec2};

use crate::{GeographicProjection, Projection, Rectangle};

pub struct BoundingRectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl BoundingRectangle {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
    pub fn computeViewportTransformation(&self, nearDepthRange: f64, farDepthRange: f64) -> DMat4 {
        let mut result: [f64; 16] = [
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
        ];
        let half_width = self.width * 0.5;
        let half_height = self.height * 0.5;
        let half_depth = (farDepthRange - nearDepthRange) * 0.5;

        let c_0_r_0 = half_width;
        let c_1_r_1 = half_height;
        let c_2_r_2 = half_depth;
        let c_3_r_0 = self.x + half_width;
        let c_3_r_1 = self.y + half_height;
        let c_3_r_2 = nearDepthRange + half_depth;
        let c_3_r_3 = 1.0;

        result[0] = c_0_r_0;
        result[1] = 0.0;
        result[2] = 0.0;
        result[3] = 0.0;
        result[4] = 0.0;
        result[5] = c_1_r_1;
        result[6] = 0.0;
        result[7] = 0.0;
        result[8] = 0.0;
        result[9] = 0.0;
        result[10] = c_2_r_2;
        result[11] = 0.0;
        result[12] = c_3_r_0;
        result[13] = c_3_r_1;
        result[14] = c_3_r_2;
        result[15] = c_3_r_3;

        return DMat4::from_cols_array(&result);
    }
    pub fn from_points(positions: &Vec<DVec2>) -> Self {
        let mut result = BoundingRectangle::new();

        if positions.len() == 0 {
            return result;
        }

        let length = positions.len();

        let mut minimum_x = positions[0].x;
        let mut minimum_y = positions[0].y;

        let mut maximum_x = positions[0].x;
        let mut maximum_y = positions[0].y;

        for i in 1..length {
            let p = positions[i];
            let x = p.x;
            let y = p.y;

            minimum_x = x.min(minimum_x);
            maximum_x = x.max(maximum_x);
            minimum_y = y.min(minimum_y);
            maximum_y = y.max(maximum_y);
        }

        result.x = minimum_x;
        result.y = minimum_y;
        result.width = maximum_x - minimum_x;
        result.height = maximum_y - minimum_y;
        return result;
    }
    pub fn from_rectangle(rectangle: Rectangle) -> Self {
        let mut result = BoundingRectangle::new();
        let projection = GeographicProjection::default();

        let lower_left = projection.project(&rectangle.south_west());
        let mut upper_right = projection.project(&rectangle.north_east());

        upper_right = upper_right.sub(lower_left);

        result.x = lower_left.x;
        result.y = lower_left.y;
        result.width = upper_right.x;
        result.height = upper_right.y;
        return result;
    }
}
