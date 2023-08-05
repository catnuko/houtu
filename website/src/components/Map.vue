<script setup>
// window.CESIUM_BASE_URL = '/Cesium';
import { onBeforeUnmount, onMounted } from 'vue';
Cesium.Ion.defaultAccessToken = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJqdGkiOiI1MmIxYWJmNy0yZDA1LTRiYmQtYmI3Ny1iMGIwNTk5NWQyMWYiLCJpZCI6Mjk5MjQsImlhdCI6MTU5OTIwMDkxOX0.aUw9ehdKoobH0GEq5lp3s3Uk9_QSMZVvFFrsLsAACqc"
const { Cartesian2, Cartesian3, Quaternion,VertexFormat,PerspectiveFrustum,FrustumGeometry,Cartographic, Camera, Scene, defaultValue, GeographicProjection, TweenCollection, MapMode2D, SceneMode, Rectangle } = Cesium
const CesiumMath = Cesium.Math
onMounted(() => {
  window.viewer = new Cesium.Viewer("map")
console.log(viewer)
    const frustum = new PerspectiveFrustum();
    frustum.fov = CesiumMath.toRadians(30.0);
    frustum.aspectRatio = 1920.0 / 1080.0;
    frustum.near = 1.0;
    frustum.far = 3.0;

    const m = FrustumGeometry.createGeometry(
      new FrustumGeometry({
        frustum: frustum,
        origin: Cartesian3.ZERO,
        orientation: Quaternion.IDENTITY,
        vertexFormat: VertexFormat.ALL,
      })
    );
    console.log(m.boundingSphere)
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
