use std::f32::consts::{PI, TAU};
use std::fmt;

use bevy::prelude::{Mesh, Vec3};
use bevy::render::mesh::Indices;
use geodesy::Ellipsoid;
use wgpu::PrimitiveTopology;
impl Default for EllipsoidShape {
    fn default() -> Self {
        let options = EllipsoidPluginOptions::default();
        return EllipsoidShape::new(options);
    }
}

pub struct EllipsoidPluginOptions {
    pub radii: Vec3,
    pub inner_radii: Vec3,
    pub minimum_clock: f32,
    pub maximum_clock: f32,
    pub minimum_cone: f32,
    pub maximum_cone: f32,
    pub stack_partitions: i32,
    pub slice_partitions: i32,
}
impl Default for EllipsoidPluginOptions {
    fn default() -> Self {
        Self {
            radii: [1.0, 1.0, 1.0].into(),
            inner_radii: [1.0, 1.0, 1.0].into(),
            minimum_clock: 0.0,
            maximum_clock: 2.0 * PI,
            minimum_cone: 0.0,
            maximum_cone: PI,
            stack_partitions: 64,
            slice_partitions: 64,
        }
    }
}

pub struct EllipsoidShape {
    pub radii: Vec3,
    pub inner_radii: Vec3,
    pub minimum_clock: f32,
    pub maximum_clock: f32,
    pub minimum_cone: f32,
    pub maximum_cone: f32,
    pub stack_partitions: i32,
    pub slice_partitions: i32,
}
impl EllipsoidShape {
    pub fn new(options: EllipsoidPluginOptions) -> Self {
        Self {
            radii: options.radii,
            inner_radii: options.inner_radii,
            minimum_clock: options.minimum_clock,
            maximum_clock: options.maximum_clock,
            minimum_cone: options.minimum_cone,
            maximum_cone: options.maximum_cone,
            stack_partitions: options.stack_partitions,
            slice_partitions: options.slice_partitions,
        }
    }
    pub fn from_xyz(x:f32,y:f32,z:f32)->Self{
        let options = EllipsoidPluginOptions::default();
        Self{
            radii:[x,y,z].into(),
            inner_radii:[x,y,z].into(),
            minimum_clock: options.minimum_clock,
            maximum_clock: options.maximum_clock,
            minimum_cone: options.minimum_cone,
            maximum_cone: options.maximum_cone,
            stack_partitions: options.stack_partitions,
            slice_partitions: options.slice_partitions,
        }
    }
    pub fn from_vec3(vec3:Vec3)->Self{
        EllipsoidShape::from_xyz(vec3.x, vec3.y, vec3.z)   
    }
    pub fn from_ellipsoid(ellipsoid:Ellipsoid)->Self{
        let a = ellipsoid.semimajor_axis();
        let c = ellipsoid.semiminor_axis();
        let b = a;
        EllipsoidShape::from_xyz(a as f32,b as f32,c as f32)
    }
    pub fn from_WGS84()->Self{
        let ellipsoid = Ellipsoid::named("WGS84").unwrap();
        EllipsoidShape::from_ellipsoid(ellipsoid)
    }
}

impl From<EllipsoidShape> for Mesh {
    fn from(sp: EllipsoidShape) -> Self {
        let radii = sp.radii;
        let innerRadii = sp.inner_radii;
        let minimumClock = sp.minimum_clock;
        let maximumClock = sp.maximum_clock;
        let minimumCone = sp.minimum_cone;
        let maximumCone = sp.maximum_cone;
        // let vertexFormat = sp._vertexFormat;
        let mut stackPartitions = sp.stack_partitions + 1;
        let mut slicePartitions = sp.slice_partitions + 1;
        let TWO_PI = TAU;

        slicePartitions = (((slicePartitions as f32) * (maximumClock - minimumClock).abs())
            / TWO_PI)
            .round() as i32;
        stackPartitions =
            (((stackPartitions as f32) * (maximumCone - minimumCone).abs()) / PI).round() as i32;

        if (slicePartitions < 2) {
            slicePartitions = 2;
        }
        if (stackPartitions < 2) {
            stackPartitions = 2;
        }

        let mut i = 0;
        let mut j = 0;
        let mut index = 0;

        // Create arrays for theta and phi. Duplicate first and last angle to
        // allow different normals at the intersections.
        let mut phis = Vec::new();
        phis.push(minimumCone);
        let mut thetas = Vec::new();
        thetas.push(minimumClock);
        for i in 0..(stackPartitions) {
            phis.push(
                minimumCone
                    + ((i as f32) * (maximumCone - minimumCone)) / ((stackPartitions as f32) - 1.),
            );
        }
        phis.push(maximumCone);
        for j in 0..(slicePartitions as i32) {
            thetas.push(
                minimumClock
                    + ((j as f32) * (maximumClock - minimumClock))
                        / ((slicePartitions as f32) - 1.),
            );
        }
        thetas.push(maximumClock);
        let numPhis = phis.len() as i32;
        let numThetas = thetas.len() as i32;

        // Allow for extra indices if there is an inner surface and if we need
        // to close the sides if the clock range is not a full circle
        let mut extraIndices: i32 = 0;
        let mut vertexMultiplier: i32 = 1;
        let mut hasInnerSurface =
            innerRadii.x != radii.x || innerRadii.y != radii.y || innerRadii.z != radii.z;
        let mut isTopOpen = false;
        let mut isBotOpen = false;
        let mut isClockOpen = false;
        if (hasInnerSurface) {
            vertexMultiplier = 2;
            if (minimumCone > 0.0) {
                isTopOpen = true;
                extraIndices += slicePartitions - 1;
            }
            if (maximumCone < PI) {
                isBotOpen = true;
                extraIndices += slicePartitions - 1;
            }
            if ((maximumClock - minimumClock) % TWO_PI > 0.) {
                isClockOpen = true;
                extraIndices += (stackPartitions - 1) * 2 + 1;
            } else {
                extraIndices += 1;
            }
        }
        let vertexCount = (numThetas * numPhis * vertexMultiplier) as i32;
        let mut positions = (0..vertexCount * 3).map(|i| 0.).collect::<Vec<f32>>();
        let mut normals = (0..vertexCount * 3).map(|i| 0.).collect::<Vec<f32>>();
        let mut st = (0..vertexCount * 2).map(|i| 0.).collect::<Vec<f32>>();
        let mut isInner = (0..vertexCount).map(|i| false).collect::<Vec<bool>>();
        let mut negateNormal = (0..vertexCount).map(|i| false).collect::<Vec<bool>>();

        // Multiply by 6 because there are two triangles per sector
        let indexCount = slicePartitions * stackPartitions * vertexMultiplier;
        let numIndices = 6
            * (indexCount + extraIndices + 1
                - (slicePartitions + stackPartitions) * vertexMultiplier);
        let mut indices = (0..numIndices).map(|i| 0).collect::<Vec<i32>>();

        let mut sinPhi = (0..numPhis).map(|i| 0.).collect::<Vec<f32>>();
        let mut cosPhi = (0..numPhis).map(|i| 0.).collect::<Vec<f32>>();
        for i in 0..numPhis {
            sinPhi[i as usize] = (phis[i as usize].sin());
            cosPhi[i as usize] = (phis[i as usize]).cos();
        }
        let mut sinTheta = (0..numThetas).map(|i| 0.).collect::<Vec<f32>>();
        let mut cosTheta = (0..numThetas).map(|i| 0.).collect::<Vec<f32>>();
        for j in 0..numThetas {
            cosTheta[j as usize] = (thetas[j as usize]).cos();
            sinTheta[j as usize] = (thetas[j as usize].sin());
        }
        // Create outer surface
        for i in 0..numPhis {
            for j in 0..numThetas {
                positions[index] = radii.x * sinPhi[i as usize] * cosTheta[j as usize];
                index += 1;
                positions[index] = radii.y * sinPhi[i as usize] * sinTheta[j as usize];
                index += 1;
                positions[index] = radii.z * cosPhi[i as usize];
                index += 1;
            }
        }
        // Create inner surface
        let mut vertexIndex = vertexCount / 2;
        if (hasInnerSurface) {
            for i in 0..numPhis {
                for j in 0..numThetas {
                    positions[index] = innerRadii.x * sinPhi[i as usize] * cosTheta[j as usize];
                    index += 1;
                    positions[index] = innerRadii.y * sinPhi[i as usize] * sinTheta[j as usize];
                    index += 1;
                    positions[index] = innerRadii.z * cosPhi[i as usize];
                    index += 1;
                    // Keep track of which vertices are the inner and which ones
                    // need the normal to be negated
                    isInner[vertexIndex as usize] = true;
                    if (i > 0 && i != numPhis - 1 && j != 0 && j != numThetas - 1) {
                        negateNormal[vertexIndex as usize] = true;
                    }
                    vertexIndex += 1;
                }
            }
        }

        // Create indices for outer surface
        index = 0;
        let mut topOffset;
        let mut bottomOffset;
        for i in 1..numPhis - 2 {
            topOffset = i * numThetas;
            bottomOffset = (i + 1) * numThetas;
            for j in 1..numThetas - 2 {
                indices[index] = bottomOffset + j;
                index += 1;
                indices[index] = bottomOffset + j + 1;
                index += 1;
                indices[index] = topOffset + j + 1;
                index += 1;
                indices[index] = bottomOffset + j;
                index += 1;
                indices[index] = topOffset + j + 1;
                index += 1;
                indices[index] = topOffset + j;
                index += 1;
            }
        }

        // Create indices for inner surface
        if (hasInnerSurface) {
            let offset = numPhis * numThetas;
            for i in 1..numPhis - 2 {
                topOffset = offset + i * numThetas;
                bottomOffset = offset + (i + 1) * numThetas;
                for j in 1..numThetas - 2 {
                    indices[index] = bottomOffset + j;
                    index += 1;
                    indices[index] = topOffset + j;
                    index += 1;
                    indices[index] = topOffset + j + 1;
                    index += 1;
                    indices[index] = bottomOffset + j;
                    index += 1;
                    indices[index] = topOffset + j + 1;
                    index += 1;
                    indices[index] = bottomOffset + j + 1;
                    index += 1;
                }
            }
            for i in 1..numPhis - 2 {
                topOffset = offset + i * numThetas;
                bottomOffset = offset + (i + 1) * numThetas;
                for j in 1..numThetas - 2 {
                    indices[index] = bottomOffset + j;
                    index += 1;
                    indices[index] = topOffset + j;
                    index += 1;
                    indices[index] = topOffset + j + 1;
                    index += 1;
                    indices[index] = bottomOffset + j;
                    index += 1;
                    indices[index] = topOffset + j + 1;
                    index += 1;
                    indices[index] = bottomOffset + j + 1;
                    index += 1;
                }
            }
        }

        let mut outerOffset;
        let mut innerOffset;
        if (hasInnerSurface) {
            if (isTopOpen) {
                // Connect the top of the inner surface to the top of the outer surface
                innerOffset = numPhis * numThetas;
                for i in 1..numThetas - 2 {
                    indices[index] = i;
                    index += 1;
                    indices[index] = i + 1;
                    index += 1;
                    indices[index] = innerOffset + i + 1;
                    index += 1;
                    indices[index] = i;
                    index += 1;
                    indices[index] = innerOffset + i + 1;
                    index += 1;
                    indices[index] = innerOffset + i;
                    index += 1;
                }
            }

            if (isBotOpen) {
                // Connect the bottom of the inner surface to the bottom of the outer surface
                outerOffset = numPhis * numThetas - numThetas;
                innerOffset = numPhis * numThetas * vertexMultiplier - numThetas;
                for i in 1..numThetas - 2 {
                    indices[index] = outerOffset + i + 1;
                    index += 1;
                    indices[index] = outerOffset + i;
                    index += 1;
                    indices[index] = innerOffset + i;
                    index += 1;
                    indices[index] = outerOffset + i + 1;
                    index += 1;
                    indices[index] = innerOffset + i;
                    index += 1;
                    indices[index] = innerOffset + i + 1;
                    index += 1;
                }
            }
        }

        // Connect the edges if clock is not closed
        if (isClockOpen) {
            for i in 1..numPhis - 2 {
                innerOffset = numThetas * numPhis + numThetas * i;
                outerOffset = numThetas * i;
                indices[index] = innerOffset;
                index += 1;
                indices[index] = outerOffset + numThetas;
                index += 1;
                indices[index] = outerOffset;
                index += 1;
                indices[index] = innerOffset;
                index += 1;
                indices[index] = innerOffset + numThetas;
                index += 1;
                indices[index] = outerOffset + numThetas;
                index += 1;
            }
            for i in 1..numPhis - 2 {
                innerOffset = numThetas * numPhis + numThetas * (i + 1) - 1;
                outerOffset = numThetas * (i + 1) - 1;
                indices[index] = outerOffset + numThetas;
                index += 1;
                indices[index] = innerOffset;
                index += 1;
                indices[index] = outerOffset;
                index += 1;

                indices[index] = outerOffset + numThetas;
                index += 1;
                indices[index] = innerOffset + numThetas;
                index += 1;
                indices[index] = innerOffset;
                index += 1;
            }
        }
        let ellipsoidOuter = Ellipsoid::new(radii.x as f64, ((radii.x - radii.z) / radii.x) as f64);
        let ellipsoidInner = Ellipsoid::new(
            innerRadii.x as f64,
            ((innerRadii.x - innerRadii.z) / radii.x) as f64,
        );
        let mut stIndex = 0;
        let mut normalIndex = 0;
        for i in 0..vertexCount {
            let ellipsoid;
            if (isInner[i as usize]) {
                ellipsoid = radii
            } else {
                ellipsoid = innerRadii
            }
            let x = positions[(i * 3) as usize];
            let y = positions[(i * 3 + 1) as usize];
            let z = positions[(i * 3 + 2) as usize];
            let position = Vec3::new(x, y, z);
            let mut normal: Vec3 = geodeticSurfaceNormal(position, ellipsoid);
            if (negateNormal[i as usize]) {
                normal = negate(normal)
            }
            let normalST = negate(normal);
            st[stIndex] = normalST.y.atan2(normalST.x) / TWO_PI + 0.5;
            stIndex += 1;
            st[stIndex] = normal.z.asin() / PI + 0.5;
            stIndex += 1;
            normals[normalIndex] = normal.x;
            normalIndex += 1;
            normals[normalIndex] = normal.y;
            normalIndex += 1;
            normals[normalIndex] = normal.z;
            normalIndex += 1;
        }

        let mut endPositions: Vec<[f32; 3]> = Vec::new();
        let mut endNormals: Vec<[f32; 3]> = Vec::new();
        let mut endST: Vec<[f32; 2]> = Vec::new();
        positions.iter().enumerate().step_by(3).for_each(|(i, x)| {
            endPositions.push([positions[i], positions[i + 1], positions[i + 2]])
        });
        normals.iter().enumerate().step_by(3).for_each(|(i, x)| {
            endNormals.push([normals[i], normals[i + 1], normals[i + 2]]);
        });
        st.iter().enumerate().step_by(2).for_each(|(i, x)| {
            endST.push([st[i], st[i + 1]]);
        });
        let indices2 = Indices::U32(indices.iter().map(|&x| x as u32).collect());
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, endPositions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, endNormals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, endST);
        mesh.set_indices(Some(indices2));
        mesh
    }
}
fn negate(vec: Vec3) -> Vec3 {
    Vec3::new(-vec.x, -vec.y, -vec.z)
}
fn geodeticSurfaceNormal(vec: Vec3, raddi: Vec3) -> Vec3 {
    let x = vec.x;
    let y = vec.y;
    let z = vec.z;
    let oneOverRadiiSquared = 1.0 / (raddi.x * raddi.x + raddi.y * raddi.y + raddi.z * raddi.z);
    Vec3::new(
        x * oneOverRadiiSquared,
        y * oneOverRadiiSquared,
        z * oneOverRadiiSquared,
    )
}
