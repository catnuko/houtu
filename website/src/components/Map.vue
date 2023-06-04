<script setup>
import { onBeforeUnmount, onMounted } from 'vue';
const {Cartesian2,Cartesian3,Camera,Scene,defaultValue,GeographicProjection,TweenCollection,MapMode2D   } = Cesium
const CesiumMath = Cesium.Math
let viewer
onMounted(() => {
    let scene;
  let camera;

  let position;
  let up;
  let dir;
  let right;

  const moveAmount = 3.0;
  const turnAmount = CesiumMath.PI_OVER_TWO;
  const rotateAmount = CesiumMath.PI_OVER_TWO;
  const zoomAmount = 1.0;

  function FakeScene(projection) {
    this.canvas = {
      clientWidth: 512,
      clientHeight: 384,
    };
    this.drawingBufferWidth = 1024;
    this.drawingBufferHeight = 768;
    this.mapProjection = defaultValue(projection, new GeographicProjection());
    this.tweens = new TweenCollection();
    this.screenSpaceCameraController = {
      minimumZoomDistance: 0,
      maximumZoomDistance: 5906376272000.0, // distance from the Sun to Pluto in meters.
    };
    this.camera = undefined;
    this.preloadFlightCamera = undefined;
    this.context = {
      drawingBufferWidth: 1024,
      drawingBufferHeight: 768,
    };
    this.mapMode2D = MapMode2D.INFINITE_2D;
  }
  position = Cartesian3.clone(Cartesian3.UNIT_Z);
    up = Cartesian3.clone(Cartesian3.UNIT_Y);
    dir = Cartesian3.negate(Cartesian3.UNIT_Z, new Cartesian3());
    right = Cartesian3.cross(dir, up, new Cartesian3());

    scene = new FakeScene();

    camera = new Camera(scene);
    camera.position = Cartesian3.clone(position);
    camera.up = Cartesian3.clone(up);
    camera.direction = Cartesian3.clone(dir);
    camera.right = Cartesian3.clone(right);

    camera.minimumZoomDistance = 0.0;

    scene.camera = camera;
    scene.preloadFlightCamera = Camera.clone(camera);
    camera._scene = scene;
    scene.mapMode2D = MapMode2D.INFINITE_2D;
  const windowCoord = new Cartesian2(
      scene.canvas.clientWidth / 2,
      scene.canvas.clientHeight
    );
    const ray = camera.getPickRay(windowCoord);

    const windowHeight =
      camera.frustum.near * Math.tan(camera.frustum.fovy * 0.5);
    const expectedDirection = Cartesian3.normalize(
      new Cartesian3(0.0, -windowHeight, -1.0),
      new Cartesian3()
    );
    console.log(camera)
    console.log(camera.position)
    console.log(ray,)
    console.log(expectedDirection)
   
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
