<script setup>
import { onBeforeUnmount, onMounted } from 'vue';
let viewer
onMounted(() => {
    viewer = new Cesium.Viewer('map', {
        terrainProvider: Cesium.createWorldTerrain()
    })
    viewer.imageryLayers.addImageryProvider(new Cesium.UrlTemplateImageryProvider({
        url:"https://maps.omniscale.net/v2/houtu-b8084b0b/style.default/{z}/{x}/{y}.png",
        tilingScheme:new Cesium.WebMercatorTilingScheme(),
    }))
    

    let tiling_scheme= new Cesium.GeographicTilingScheme()
    let level = 0
    let num_of_x_tiles = tiling_scheme.getNumberOfXTilesAtLevel(level);
    let num_of_y_tiles = tiling_scheme.getNumberOfYTilesAtLevel(level);
    for(let y=0;y<num_of_y_tiles;y++){
      for (let x=0;x<num_of_x_tiles;x++){
        let width = 16;
            let height = 16;
        let heigmapTerrainData = new Cesium.HeightmapTerrainData({
          buffer: new Uint8Array(width * height),
          width: width,
          height: height,
        })
        heigmapTerrainData._createMeshSync({tilingScheme:tiling_scheme,x,y,level})
      }
    }
    viewer.entities.add({
        position: Cesium.Cartesian3.fromDegrees(0.0, 6378137, 0.0),
        box: {
            dimensions: new Cesium.Cartesian3(400000.0, 300000.0, 500000.0),
            material: Cesium.Color.RED.withAlpha(0.5),
            outline: true,
            outlineColor: Cesium.Color.BLACK,
        },
    })
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
