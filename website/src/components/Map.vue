<script setup>
import { onBeforeUnmount, onMounted } from 'vue';
let viewer
onMounted(()=>{
    const {Matrix3} = Cesium
    viewer = new Cesium.Viewer('map', {
        terrainProvider: Cesium.createWorldTerrain()
    })
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
})
onBeforeUnmount(()=>{
    viewer.destroy()
})
</script>

<template>
<div id="map"></div>
</template>

<style scoped>
#map{
    z-index: 0;
    width: 100%;
    height: 100%;
}
</style>
