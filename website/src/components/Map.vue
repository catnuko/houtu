<script setup>
window.CESIUM_BASE_URL = '/Cesium';
import { onBeforeUnmount, onMounted } from 'vue';
import * as Cesium from 'cesium'
import "cesium/Build/Cesium/Widgets/widgets.css";
Cesium.Ion.defaultAccessToken = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJqdGkiOiI1MmIxYWJmNy0yZDA1LTRiYmQtYmI3Ny1iMGIwNTk5NWQyMWYiLCJpZCI6Mjk5MjQsImlhdCI6MTU5OTIwMDkxOX0.aUw9ehdKoobH0GEq5lp3s3Uk9_QSMZVvFFrsLsAACqc"
const { Cartesian2, Cartesian3, Cartographic, Camera, Scene, defaultValue, GeographicProjection, TweenCollection, MapMode2D, SceneMode, Rectangle } = Cesium
const CesiumMath = Cesium.Math
onMounted(() => {
  window.viewer = new Cesium.Viewer("map")
console.log(viewer)
viewer.scene.globe._surface._debug.enableDebugOutput=true
  let p_wc = new Cartesian3(
    0, 0, 17020972.5
  );
  let d_wc = new Cartesian3(0, 0, -1)
  let u_wc = new Cartesian3(0, 1, 0)
  let v = viewer.camera.frustum.computeCullingVolume(p_wc, d_wc, u_wc);
  console.log(v)
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
