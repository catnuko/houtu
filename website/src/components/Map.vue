<script setup>
import { onBeforeUnmount, onMounted } from 'vue';
let viewer
onMounted(() => {
    const { Matrix3 } = Cesium
    viewer = new Cesium.Viewer('map', {
        terrainProvider: Cesium.createWorldTerrain()
    })
    
    const returnedResult = Cesium.Ellipsoid.WGS84.cartesianToCartographic(
      new Cesium.Cartesian3(1e-50, 1e-60, 1e-70)
    );


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
