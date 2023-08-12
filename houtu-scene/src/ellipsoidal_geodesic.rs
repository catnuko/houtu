use std::f64::consts::{PI, TAU};

use bevy::math::DVec3;

use crate::{
    ellipsoid::{self, Ellipsoid},
    math::Cartesian3,
    Cartographic, EPSILON12,
};
#[derive(Default)]
pub struct EllipsoidGeodesicConstants {
    pub a: f64,
    pub b: f64,
    pub f: f64,
    pub cosineHeading: f64,
    pub sineHeading: f64,
    pub tanU: f64,
    pub cosineU: f64,
    pub sineU: f64,
    pub sigma: f64,
    pub sineAlpha: f64,
    pub sineSquaredAlpha: f64,
    pub cosineSquaredAlpha: f64,
    pub cosineAlpha: f64,
    pub u2Over4: f64,
    pub u4Over16: f64,
    pub u6Over64: f64,
    pub u8Over256: f64,
    pub a0: f64,
    pub a1: f64,
    pub a2: f64,
    pub a3: f64,
    pub distanceRatio: f64,
}
impl Default for EllipsoidGeodesic {
    fn default() -> Self {
        Self::new(Cartographic::default(), Cartographic::default())
    }
}
pub struct EllipsoidGeodesic {
    pub start: Cartographic,
    pub end: Cartographic,
    startHeading: f64,
    endHeading: f64,
    distance: f64,
    uSquared: f64,
    ellipsoid: Ellipsoid,
    constants: EllipsoidGeodesicConstants,
}
impl EllipsoidGeodesic {
    pub fn new(start: Cartographic, end: Cartographic) -> Self {
        EllipsoidGeodesic {
            start: start,
            end: end,
            startHeading: 0.,
            endHeading: 0.,
            distance: 0.,
            uSquared: 0.,
            ellipsoid: Ellipsoid::WGS84,
            constants: EllipsoidGeodesicConstants::default(),
        }
    }
    pub fn setEndPoints(&mut self, start: Cartographic, end: Cartographic) {
        self.computeProperties(&start, &end);
    }
    pub fn interpolateUsingFraction(&mut self, fraction: f64) -> Cartographic {
        return self.interpolateUsingSurfaceDistance(self.distance * fraction);
    }
    pub fn interpolateUsingSurfaceDistance(&self, distance: f64) -> Cartographic {
        let constants = &self.constants;

        let s = constants.distanceRatio + distance / constants.b;

        let cosine2S = (2.0 * s).cos();
        let cosine4S = (4.0 * s).cos();
        let cosine6S = (6.0 * s).cos();
        let sine2S = (2.0 * s).sin();
        let sine4S = (4.0 * s).sin();
        let sine6S = (6.0 * s).sin();
        let sine8S = (8.0 * s).sin();

        let s2 = s * s;
        let s3 = s * s2;

        let u8Over256 = constants.u8Over256;
        let u2Over4 = constants.u2Over4;
        let u6Over64 = constants.u6Over64;
        let u4Over16 = constants.u4Over16;
        let mut sigma = (2.0 * s3 * u8Over256 * cosine2S) / 3.0
            + s * (1.0 - u2Over4 + (7.0 * u4Over16) / 4.0 - (15.0 * u6Over64) / 4.0
                + (579.0 * u8Over256) / 64.0
                - (u4Over16 - (15.0 * u6Over64) / 4.0 + (187.0 * u8Over256) / 16.0) * cosine2S
                - ((5.0 * u6Over64) / 4.0 - (115.0 * u8Over256) / 16.0) * cosine4S
                - (29.0 * u8Over256 * cosine6S) / 16.0)
            + (u2Over4 / 2.0 - u4Over16 + (71.0 * u6Over64) / 32.0 - (85.0 * u8Over256) / 16.0)
                * sine2S
            + ((5.0 * u4Over16) / 16.0 - (5.0 * u6Over64) / 4.0 + (383.0 * u8Over256) / 96.0)
                * sine4S
            - s2 * ((u6Over64 - (11.0 * u8Over256) / 2.0) * sine2S
                + (5.0 * u8Over256 * sine4S) / 2.0)
            + ((29.0 * u6Over64) / 96.0 - (29.0 * u8Over256) / 16.0) * sine6S
            + (539.0 * u8Over256 * sine8S) / 1536.0;

        let theta = (sigma.sin() * constants.cosineAlpha).asin();
        let latitude = ((constants.a / constants.b) * theta.tan()).atan();

        // Redefine in terms of relative argument of latitude.
        sigma = sigma - constants.sigma;

        let cosineTwiceSigmaMidpoint = (2.0 * constants.sigma + sigma).cos();

        let sineSigma = (sigma).sin();
        let cosineSigma = (sigma).cos();

        let cc = constants.cosineU * cosineSigma;
        let ss = constants.sineU * sineSigma;

        let lambda = (sineSigma * constants.sineHeading).atan2(cc - ss * constants.cosineHeading);

        let l = lambda
            - computeDeltaLambda(
                constants.f,
                constants.sineAlpha,
                constants.cosineSquaredAlpha,
                sigma,
                sineSigma,
                cosineSigma,
                cosineTwiceSigmaMidpoint,
            );

        return Cartographic::new(self.start.longitude + l, latitude, 0.0);
    }
    fn computeProperties(&mut self, start: &Cartographic, end: &Cartographic) {
        let firstCartesian = self.ellipsoid.cartographic_to_cartesian(start).normalize();
        let lastCartesian = self.ellipsoid.cartographic_to_cartesian(end).normalize();

        if (firstCartesian.angle_between(lastCartesian).abs() - PI).abs() < 0.0125 {
            panic!("")
        }

        //>>includeEnd('debug');

        self.vincentyInverseFormula(
            self.ellipsoid.maximum_radius,
            self.ellipsoid.minimum_radius,
            start.longitude,
            start.latitude,
            end.longitude,
            end.latitude,
        );

        self.start = start.clone();
        self.end = end.clone();
        self.start.height = 0.;
        self.end.height = 0.;

        self.setConstants();
    }
    fn vincentyInverseFormula(
        &mut self,
        major: f64,
        minor: f64,
        firstLongitude: f64,
        firstLatitude: f64,
        secondLongitude: f64,
        secondLatitude: f64,
    ) {
        let eff = (major - minor) / major;
        let l = secondLongitude - firstLongitude;

        let u1 = ((1.0 - eff) * firstLatitude.tan()).atan();
        let u2 = ((1.0 - eff) * secondLatitude.tan()).atan();

        let cosineU1 = u1.cos();
        let sineU1 = u1.sin();
        let cosineU2 = u2.cos();
        let sineU2 = u2.sin();

        let cc = cosineU1 * cosineU2;
        let cs = cosineU1 * sineU2;
        let ss = sineU1 * sineU2;
        let sc = sineU1 * cosineU2;

        let mut lambda = l;
        let mut lambdaDot = TAU;

        let mut cosineLambda = lambda.cos();
        let mut sineLambda = lambda.sin();

        let mut sigma;
        let mut cosineSigma;
        let mut sineSigma;
        let mut cosineSquaredAlpha;
        let mut cosineTwiceSigmaMidpoint;

        loop {
            cosineLambda = lambda.cos();
            sineLambda = lambda.sin();

            let temp = cs - sc * cosineLambda;
            sineSigma = (cosineU2 * cosineU2 * sineLambda * sineLambda + temp * temp).sqrt();
            cosineSigma = ss + cc * cosineLambda;

            sigma = sineSigma.atan2(cosineSigma);

            let sineAlpha;

            if sineSigma == 0.0 {
                sineAlpha = 0.0;
                cosineSquaredAlpha = 1.0;
            } else {
                sineAlpha = (cc * sineLambda) / sineSigma;
                cosineSquaredAlpha = 1.0 - sineAlpha * sineAlpha;
            }

            lambdaDot = lambda;

            cosineTwiceSigmaMidpoint = cosineSigma - (2.0 * ss) / cosineSquaredAlpha;

            if !cosineTwiceSigmaMidpoint.is_finite() {
                cosineTwiceSigmaMidpoint = 0.0;
            }

            lambda = l + computeDeltaLambda(
                eff,
                sineAlpha,
                cosineSquaredAlpha,
                sigma,
                sineSigma,
                cosineSigma,
                cosineTwiceSigmaMidpoint,
            );
            if (lambda - lambdaDot).abs() <= EPSILON12 {
                break;
            }
        }

        let uSquared = (cosineSquaredAlpha * (major * major - minor * minor)) / (minor * minor);
        let A = 1.0
            + (uSquared * (4096.0 + uSquared * (uSquared * (320.0 - 175.0 * uSquared) - 768.0)))
                / 16384.0;
        let B = (uSquared * (256.0 + uSquared * (uSquared * (74.0 - 47.0 * uSquared) - 128.0)))
            / 1024.0;

        let cosineSquaredTwiceSigmaMidpoint = cosineTwiceSigmaMidpoint * cosineTwiceSigmaMidpoint;
        let deltaSigma = B
            * sineSigma
            * (cosineTwiceSigmaMidpoint
                + (B * (cosineSigma * (2.0 * cosineSquaredTwiceSigmaMidpoint - 1.0)
                    - (B * cosineTwiceSigmaMidpoint
                        * (4.0 * sineSigma * sineSigma - 3.0)
                        * (4.0 * cosineSquaredTwiceSigmaMidpoint - 3.0))
                        / 6.0))
                    / 4.0);

        let distance = minor * A * (sigma - deltaSigma);

        let startHeading = (cosineU2 * sineLambda).atan2(cs - sc * cosineLambda);
        let endHeading = (cosineU1 * sineLambda).atan2(cs * cosineLambda - sc);

        self.distance = distance;
        self.startHeading = startHeading;
        self.endHeading = endHeading;
        self.uSquared = uSquared;
    }
    fn setConstants(&mut self) {
        let uSquared = self.uSquared;
        let a = self.ellipsoid.maximum_radius;
        let b = self.ellipsoid.minimum_radius;
        let f = (a - b) / a;

        let cosineHeading = self.startHeading.cos();
        let sineHeading = self.startHeading.sin();

        let tanU = (1. - f) * self.start.latitude.tan();

        let cosineU = 1.0 / (1.0 + tanU * tanU).sqrt();
        let sineU = cosineU * tanU;

        let sigma = tanU.atan2(cosineHeading);

        let sineAlpha = cosineU * sineHeading;
        let sineSquaredAlpha = sineAlpha * sineAlpha;

        let cosineSquaredAlpha = 1.0 - sineSquaredAlpha;
        let cosineAlpha = cosineSquaredAlpha.sqrt();

        let u2Over4 = uSquared / 4.0;
        let u4Over16 = u2Over4 * u2Over4;
        let u6Over64 = u4Over16 * u2Over4;
        let u8Over256 = u4Over16 * u4Over16;

        let a0 = 1.0 + u2Over4 - (3.0 * u4Over16) / 4.0 + (5.0 * u6Over64) / 4.0
            - (175.0 * u8Over256) / 64.0;
        let a1 = 1.0 - u2Over4 + (15.0 * u4Over16) / 8.0 - (35.0 * u6Over64) / 8.0;
        let a2 = 1.0 - 3.0 * u2Over4 + (35.0 * u4Over16) / 4.0;
        let a3 = 1.0 - 5.0 * u2Over4;

        let distanceRatio = a0 * sigma
            - (a1 * (2.0 * sigma).sin() * u2Over4) / 2.0
            - (a2 * (4.0 * sigma).sin() * u4Over16) / 16.0
            - (a3 * (6.0 * sigma).sin() * u6Over64) / 48.0
            - ((8.0 * sigma).sin() * 5.0 * u8Over256) / 512.;

        let constants = &mut self.constants;

        constants.a = a;
        constants.b = b;
        constants.f = f;
        constants.cosineHeading = cosineHeading;
        constants.sineHeading = sineHeading;
        constants.tanU = tanU;
        constants.cosineU = cosineU;
        constants.sineU = sineU;
        constants.sigma = sigma;
        constants.sineAlpha = sineAlpha;
        constants.sineSquaredAlpha = sineSquaredAlpha;
        constants.cosineSquaredAlpha = cosineSquaredAlpha;
        constants.cosineAlpha = cosineAlpha;
        constants.u2Over4 = u2Over4;
        constants.u4Over16 = u4Over16;
        constants.u6Over64 = u6Over64;
        constants.u8Over256 = u8Over256;
        constants.a0 = a0;
        constants.a1 = a1;
        constants.a2 = a2;
        constants.a3 = a3;
        constants.distanceRatio = distanceRatio;
    }
}

fn computeDeltaLambda(
    f: f64,
    sineAlpha: f64,
    cosineSquaredAlpha: f64,
    sigma: f64,
    sineSigma: f64,
    cosineSigma: f64,
    cosineTwiceSigmaMidpoint: f64,
) -> f64 {
    let C = computeC(f, cosineSquaredAlpha);

    return ((1.0 - C)
        * f
        * sineAlpha
        * (sigma
            + C * sineSigma
                * (cosineTwiceSigmaMidpoint
                    + C * cosineSigma
                        * (2.0 * cosineTwiceSigmaMidpoint * cosineTwiceSigmaMidpoint - 1.0))));
}
fn computeC(f: f64, cosineSquaredAlpha: f64) -> f64 {
    return ((f * cosineSquaredAlpha * (4.0 + f * (4.0 - 3.0 * cosineSquaredAlpha))) / 16.0);
}
