<script setup lang="ts">
// window.CESIUM_BASE_URL = '/Cesium';
import { onBeforeUnmount, onMounted } from 'vue';
Cesium.Ion.defaultAccessToken =
	'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJqdGkiOiI1MmIxYWJmNy0yZDA1LTRiYmQtYmI3Ny1iMGIwNTk5NWQyMWYiLCJpZCI6Mjk5MjQsImlhdCI6MTU5OTIwMDkxOX0.aUw9ehdKoobH0GEq5lp3s3Uk9_QSMZVvFFrsLsAACqc';
const {
	Cartesian2,
	Cartesian3,
	AttributeCompression,
	Quaternion,
	VertexFormat,
	PerspectiveFrustum,
	FrustumGeometry,
	Cartographic,
	Camera,
	Scene,
	defaultValue,
	GeographicProjection,
	TweenCollection,
	MapMode2D,
	SceneMode,
	Rectangle,
} = Cesium;
const CesiumMath = Cesium.Math;
let viewer: Cesium.Viewer;
onMounted(() => {
	viewer = new Cesium.Viewer('map', { baseLayer: false });
	console.log(viewer);
	var TDTURL_CONFIG = {
		TDT_IMG_W:
			'http://{s}.tianditu.gov.cn/img_w/wmts?service=wmts&request=GetTile&version=1.0.0' +
			'&LAYER=img&tileMatrixSet=w&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}' +
			'&style=default&format=tiles&tk=5ac36718ffda736958317e215b8664a7', //在线天地图影像服务地址(墨卡托投影)
		TDT_VEC_W:
			'http://{s}.tianditu.gov.cn/vec_w/wmts?service=wmts&request=GetTile&version=1.0.0' +
			'&LAYER=vec&tileMatrixSet=w&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}' +
			'&style=default&format=tiles&tk=5ac36718ffda736958317e215b8664a7', //在线天地图矢量地图服务(墨卡托投影)
		TDT_CIA_W:
			'http://{s}.tianditu.gov.cn/cia_w/wmts?service=wmts&request=GetTile&version=1.0.0' +
			'&LAYER=cia&tileMatrixSet=w&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}' +
			'&style=default.jpg&tk=5ac36718ffda736958317e215b8664a7', //在线天地图影像中文标记服务(墨卡托投影)
		TDT_CVA_W:
			'http://{s}.tianditu.gov.cn/cva_w/wmts?service=wmts&request=GetTile&version=1.0.0' +
			'&LAYER=cva&tileMatrixSet=w&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}' +
			'&style=default.jpg&tk=5ac36718ffda736958317e215b8664a7', //在线天地图矢量中文标记服务(墨卡托投影)
		TDT_IMG_C:
			'http://{s}.tianditu.gov.cn/img_c/wmts?service=wmts&request=GetTile&version=1.0.0' +
			'&LAYER=img&tileMatrixSet=c&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}' +
			'&style=default&format=tiles&tk=5ac36718ffda736958317e215b8664a7', //在线天地图影像服务地址(经纬度)
		TDT_VEC_C:
			'http://{s}.tianditu.gov.cn/vec_c/wmts?service=wmts&request=GetTile&version=1.0.0' +
			'&LAYER=vec&tileMatrixSet=c&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}' +
			'&style=default&format=tiles&tk=5ac36718ffda736958317e215b8664a7', //在线天地图矢量地图服务(经纬度)
		TDT_CIA_C:
			'http://{s}.tianditu.gov.cn/cia_c/wmts?service=wmts&request=GetTile&version=1.0.0' +
			'&LAYER=cia&tileMatrixSet=c&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}' +
			'&style=default&format=tiles&tk=5ac36718ffda736958317e215b8664a7', //在线天地图影像中文标记服务(经纬度)
		TDT_CVA_C:
			'http://{s}.tianditu.gov.cn/cva_c/wmts?service=wmts&request=GetTile&version=1.0.0' +
			'&LAYER=cva&tileMatrixSet=c&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}' +
			'&style=default&format=tiles&tk=5ac36718ffda736958317e215b8664a7', //在线天地图矢量中文标记服务(经纬度)
	};
	viewer.imageryLayers.remove(viewer.imageryLayers._layers[0])
	//天地图影像中文标记服务（经纬度）
	// var tdtCva = new Cesium.WebMapTileServiceImageryProvider({
	// 	url: TDTURL_CONFIG.TDT_IMG_C,
	// 	layer: 'tdtImg_c',
	// 	style: 'default',
	// 	format: 'tiles',
	// 	tileMatrixSetID: 'c',
	// 	subdomains: ['t0', 't1', 't2', 't3', 't4', 't5', 't6', 't7'],
	// 	tilingScheme: new Cesium.GeographicTilingScheme(),
	// 	tileMatrixLabels: ['1', '2', '3', '4', '5', '6', '7', '8', '9', '10', '11', '12', '13', '14', '15', '16', '17', '18', '19'],
	// 	maximumLevel: 18,
	// });
	// viewer.imageryLayers.addImageryProvider(tdtCva);
	var tdtCva = new Cesium.WebMapTileServiceImageryProvider({
		url: TDTURL_CONFIG.TDT_IMG_W,
		layer: 'tdtImgLayer',
		style: 'default',
		format: "image/jpeg",
		tileMatrixSetID: 'GoogleMapsCompatible',
		subdomains: ['t0', 't1', 't2', 't3', 't4', 't5', 't6', 't7'],
		maximumLevel: 18,
	});
	viewer.imageryLayers.addImageryProvider(tdtCva);
	// var tdtCva = new Cesium.WebMapTileServiceImageryProvider({
	// 	url: TDTURL_CONFIG.TDT_CIA_W,
	// 	layer: 'tdtImgLayer',
	// 	style: 'default',
	// 	format: "image/jpeg",
	// 	tileMatrixSetID: 'c',
	// 	subdomains: ['t0', 't1', 't2', 't3', 't4', 't5', 't6', 't7'],
	// 	maximumLevel: 18,
	// });
	// viewer.imageryLayers.addImageryProvider(tdtCva);

});
onBeforeUnmount(() => {
	viewer.destroy();
});
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
