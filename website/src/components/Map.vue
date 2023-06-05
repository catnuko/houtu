<script setup>
import { onBeforeUnmount, onMounted } from 'vue';
Cesium.Ion.defaultAccessToken = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJqdGkiOiI1MmIxYWJmNy0yZDA1LTRiYmQtYmI3Ny1iMGIwNTk5NWQyMWYiLCJpZCI6Mjk5MjQsImlhdCI6MTU5OTIwMDkxOX0.aUw9ehdKoobH0GEq5lp3s3Uk9_QSMZVvFFrsLsAACqc"
const {Cartesian2,Cartesian3,Camera,Scene,defaultValue,GeographicProjection,TweenCollection,MapMode2D   } = Cesium
const CesiumMath = Cesium.Math
let viewer
onMounted(() => {
  viewer = new Cesium.Viewer("map")
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
