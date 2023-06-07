<script setup>
import { onBeforeUnmount, onMounted } from 'vue';
Cesium.Ion.defaultAccessToken = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJqdGkiOiI1MmIxYWJmNy0yZDA1LTRiYmQtYmI3Ny1iMGIwNTk5NWQyMWYiLCJpZCI6Mjk5MjQsImlhdCI6MTU5OTIwMDkxOX0.aUw9ehdKoobH0GEq5lp3s3Uk9_QSMZVvFFrsLsAACqc"
const {Cartesian2,Cartesian3,Camera,Scene,defaultValue,GeographicProjection,TweenCollection,MapMode2D   ,SceneMode ,Rectangle} = Cesium
const CesiumMath = Cesium.Math
let viewer
onMounted(() => {
  viewer = new Cesium.Viewer("map")
  // let scene;
  // let camera;

  // let position;
  // let up;
  // let dir;
  // let right;

  // const moveAmount = 3.0;
  // const turnAmount = CesiumMath.PI_OVER_TWO;
  // const rotateAmount = CesiumMath.PI_OVER_TWO;
  // const zoomAmount = 1.0;

  // function FakeScene(projection) {
  //   this.canvas = {
  //     clientWidth: 512,
  //     clientHeight: 384,
  //   };
  //   this.drawingBufferWidth = 1024;
  //   this.drawingBufferHeight = 768;
  //   this.mapProjection = defaultValue(projection, new GeographicProjection());
  //   this.tweens = new TweenCollection();
  //   this.screenSpaceCameraController = {
  //     minimumZoomDistance: 0,
  //     maximumZoomDistance: 5906376272000.0, // distance from the Sun to Pluto in meters.
  //   };
  //   this.camera = undefined;
  //   this.preloadFlightCamera = undefined;
  //   this.context = {
  //     drawingBufferWidth: 1024,
  //     drawingBufferHeight: 768,
  //   };
  //   this.mapMode2D = MapMode2D.INFINITE_2D;
  // }
  // position = Cartesian3.clone(Cartesian3.UNIT_Z);
  //   up = Cartesian3.clone(Cartesian3.UNIT_Y);
  //   dir = Cartesian3.negate(Cartesian3.UNIT_Z, new Cartesian3());
  //   right = Cartesian3.cross(dir, up, new Cartesian3());

  //   scene = new FakeScene();

  //   camera = new Camera(scene);
  //   camera.position = Cartesian3.clone(position);
  //   camera.up = Cartesian3.clone(up);
  //   camera.direction = Cartesian3.clone(dir);
  //   camera.right = Cartesian3.clone(right);

  //   camera.minimumZoomDistance = 0.0;

  //   scene.camera = camera;
  //   scene.preloadFlightCamera = Camera.clone(camera);
  //   camera._scene = scene;
  //   scene.mapMode2D = MapMode2D.INFINITE_2D;
  //   //单元测试
  //   const direction = Cartesian3.normalize(
  //     new Cartesian3(1.0, 1.0, 0.0),
  //     new Cartesian3()
  //   );
  //   camera.move(direction, moveAmount);
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
