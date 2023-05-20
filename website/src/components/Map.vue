<script setup>
import { onBeforeUnmount, onMounted } from 'vue';
let viewer
onMounted(() => {
    const { Matrix3 ,Cartesian3,Rectangle,HeightmapTessellator,Ellipsoid} = Cesium
    let CesiumMath =Cesium. Math;
    viewer = new Cesium.Viewer('map', {
        terrainProvider: Cesium.createWorldTerrain()
    })
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
