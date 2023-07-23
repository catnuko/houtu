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
        let halfWidth = self.width * 0.5;
        let halfHeight = self.height * 0.5;
        let halfDepth = (farDepthRange - nearDepthRange) * 0.5;

        let column0Row0 = halfWidth;
        let column1Row1 = halfHeight;
        let column2Row2 = halfDepth;
        let column3Row0 = self.x + halfWidth;
        let column3Row1 = self.y + halfHeight;
        let column3Row2 = nearDepthRange + halfDepth;
        let column3Row3 = 1.0;

        result[0] = column0Row0;
        result[1] = 0.0;
        result[2] = 0.0;
        result[3] = 0.0;
        result[4] = 0.0;
        result[5] = column1Row1;
        result[6] = 0.0;
        result[7] = 0.0;
        result[8] = 0.0;
        result[9] = 0.0;
        result[10] = column2Row2;
        result[11] = 0.0;
        result[12] = column3Row0;
        result[13] = column3Row1;
        result[14] = column3Row2;
        result[15] = column3Row3;

        return DMat4::from_cols_array(&result);
    }
    pub fn fromPoints(positions: &Vec<DVec2>) -> Self {
        let mut result = BoundingRectangle::new();

        if positions.len() == 0 {
            return result;
        }

        let length = positions.len();

        let mut minimumX = positions[0].x;
        let mut minimumY = positions[0].y;

        let mut maximumX = positions[0].x;
        let mut maximumY = positions[0].y;

        for i in 1..length {
            let p = positions[i];
            let x = p.x;
            let y = p.y;

            minimumX = x.min(minimumX);
            maximumX = x.max(maximumX);
            minimumY = y.min(minimumY);
            maximumY = y.max(maximumY);
        }

        result.x = minimumX;
        result.y = minimumY;
        result.width = maximumX - minimumX;
        result.height = maximumY - minimumY;
        return result;
    }
    pub fn fromRectangle(rectangle: Rectangle) -> Self {
        let mut result = BoundingRectangle::new();
        let projection = GeographicProjection::default();

        let lowerLeft = projection.project(&rectangle.south_west());
        let mut upperRight = projection.project(&rectangle.north_east());

        upperRight = upperRight.sub(lowerLeft);

        result.x = lowerLeft.x;
        result.y = lowerLeft.y;
        result.width = upperRight.x;
        result.height = upperRight.y;
        return result;
    }
}
