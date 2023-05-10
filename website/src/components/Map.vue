<script setup>
import { onBeforeUnmount, onMounted } from 'vue';
let viewer
onMounted(() => {
    const { Matrix3 ,Cartesian3} = Cesium
    viewer = new Cesium.Viewer('map', {
        terrainProvider: Cesium.createWorldTerrain()
    })
    console.log(Cesium.WebMercatorProjection.MaximumLatitude)
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
