<script setup>
import { onBeforeUnmount, onMounted } from 'vue';
let viewer
onMounted(() => {
    const { Matrix3 ,Cartesian3} = Cesium
    viewer = new Cesium.Viewer('map', {
        terrainProvider: Cesium.createWorldTerrain()
    })
    
    const returnedResult = Cesium.Ellipsoid.WGS84.cartesianToCartographic(
      new Cesium.Cartesian3(1e-50, 1e-60, 1e-70)
    );
    const a = new Matrix3(4.0, -1.0, 1.0, -1.0, 3.0, -2.0, 1.0, -2.0, 3.0);

const expectedDiagonal = new Matrix3(
    3.0,
    0.0,
    0.0,
    0.0,
    6.0,
    0.0,
    0.0,
    0.0,
    1.0
);

const decomposition = Matrix3.computeEigenDecomposition(a);
console.log(decomposition)
let bo = Matrix3.equalsEpsilon(decomposition.diagonal, expectedDiagonal, Cesium.Math.EPSILON14);
console.log(bo)
let jMatrixTranspose = Matrix3.fromArray([
    0.816496551,
    -0.577350259,
    0.0,
    0.0577350259,
    0.0816496551,
    0.0,
    0.0,
    0.0,
    1.0,])
let diagMatrix = Matrix3.fromArray([
    2.44948959,
    1.73205066,
    0.0,
    -3.46410155,
    4.89897919,
    0.0,
    0.0,
    0.0,
    0.999999998,
]);
let next_diagMatrix = Matrix3.multiply(jMatrixTranspose, diagMatrix, new Cesium.Matrix3());
console.log(next_diagMatrix)
const positions = [
    new Cartesian3(2.0, 0.0, 0.0),
    new Cartesian3(0.0, 3.0, 0.0),
    new Cartesian3(0.0, 0.0, 4.0),
    new Cartesian3(-2.0, 0.0, 0.0),
    new Cartesian3(0.0, -3.0, 0.0),
    new Cartesian3(0.0, 0.0, -4.0),
  ];
  const box = Cesium.OrientedBoundingBox.fromPoints(positions);

})
onBeforeUnmount(() => {
    viewer.destroy()
})
</script>

<template>
    <div id="map"></div>
</template>

<style scoped>
#map {
    z-index: 0;
    width: 100%;
    height: 100%;
}
</style>
