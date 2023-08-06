<script setup>
// window.CESIUM_BASE_URL = '/Cesium';
import { onBeforeUnmount, onMounted } from 'vue';
Cesium.Ion.defaultAccessToken = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJqdGkiOiI1MmIxYWJmNy0yZDA1LTRiYmQtYmI3Ny1iMGIwNTk5NWQyMWYiLCJpZCI6Mjk5MjQsImlhdCI6MTU5OTIwMDkxOX0.aUw9ehdKoobH0GEq5lp3s3Uk9_QSMZVvFFrsLsAACqc"
const { Cartesian2, Cartesian3,AttributeCompression, Quaternion,VertexFormat,PerspectiveFrustum,FrustumGeometry,Cartographic, Camera, Scene, defaultValue, GeographicProjection, TweenCollection, MapMode2D, SceneMode, Rectangle } = Cesium
const CesiumMath = Cesium.Math
onMounted(() => {
  window.viewer = new Cesium.Viewer("map")
console.log(viewer)
  let position = new Cartesian2(5000,6000);
  const compressed0 = AttributeCompression.compressTextureCoordinates(position);
    let newPosition = new Cartesian2();
  AttributeCompression.decompressTextureCoordinates(compressed0,newPosition)
  console.log(compressed0,newPosition)
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
