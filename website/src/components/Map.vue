<script setup>
import { onBeforeUnmount, onMounted } from 'vue';
let viewer
onMounted(() => {
    const { Matrix3 ,Cartesian3,Rectangle,HeightmapTessellator,Ellipsoid} = Cesium
    let CesiumMath =Cesium. Math;
    viewer = new Cesium.Viewer('map', {
        terrainProvider: Cesium.createWorldTerrain()
    })
    const width = 3;
    const height = 3;
    const options = {
      heightmap: [
        1.0,
        2.0,
        100.0,
        3.0,
        4.0,
        100.0,
        5.0,
        6.0,
        100.0,
        7.0,
        8.0,
        100.0,
        9.0,
        10.0,
        100.0,
        11.0,
        12.0,
        100.0,
        13.0,
        14.0,
        100.0,
        15.0,
        16.0,
        100.0,
        17.0,
        18.0,
        100.0,
      ],
      width: width,
      height: height,
      skirtHeight: 0.0,
      nativeRectangle: {
        west: 10.0,
        south: 30.0,
        east: 20.0,
        north: 40.0,
      },
      rectangle: new Rectangle(
        CesiumMath.toRadians(10.0),
        CesiumMath.toRadians(30.0),
        CesiumMath.toRadians(20.0),
        CesiumMath.toRadians(40.0)
      ),
      structure: {
        stride: 3,
        elementsPerHeight: 2,
        elementMultiplier: 10,
      },
    };
    const results = HeightmapTessellator.computeVertices(options);
    const vertices = results.vertices;

    const ellipsoid = Ellipsoid.WGS84;
    const nativeRectangle = options.nativeRectangle;

    for (let j = 0; j < height; ++j) {
      let latitude = CesiumMath.lerp(
        nativeRectangle.north,
        nativeRectangle.south,
        j / (height - 1)
      );
      latitude = CesiumMath.toRadians(latitude);
      for (let i = 0; i < width; ++i) {
        let longitude = CesiumMath.lerp(
          nativeRectangle.west,
          nativeRectangle.east,
          i / (width - 1)
        );
        longitude = CesiumMath.toRadians(longitude);

        const heightSampleIndex = (j * width + i) * options.structure.stride;
        const heightSample =
          options.heightmap[heightSampleIndex] +
          options.heightmap[heightSampleIndex + 1] * 10.0;

        const expectedVertexPosition = ellipsoid.cartographicToCartesian({
          longitude: longitude,
          latitude: latitude,
          height: heightSample,
        });

        const index = (j * width + i) * 6;
        const vertexPosition = new Cartesian3(
          vertices[index],
          vertices[index + 1],
          vertices[index + 2]
        );

        // expect(vertexPosition).toEqualEpsilon(expectedVertexPosition, 1.0);
        // expect(vertices[index + 3]).toEqual(heightSample);
        // expect(vertices[index + 4]).toEqualEpsilon(
        //   i / (width - 1),
        //   CesiumMath.EPSILON7
        // );
        // expect(vertices[index + 5]).toEqualEpsilon(
        //   1.0 - j / (height - 1),
        //   CesiumMath.EPSILON7
        // );
      }
    }
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
