var TDTURL_CONFIG={
    TDT_IMG_W:"http://{s}.tianditu.gov.cn/img_w/wmts?service=wmts&request=GetTile&version=1.0.0" +
    "&LAYER=img&tileMatrixSet=w&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}" +
    "&style=default&format=tiles&tk=5aaa55b9147f14d9e34f00f1a110e9b9"   //在线天地图影像服务地址(墨卡托投影)
    ,TDT_VEC_W:"http://{s}.tianditu.gov.cn/vec_w/wmts?service=wmts&request=GetTile&version=1.0.0" +
    "&LAYER=vec&tileMatrixSet=w&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}" +
    "&style=default&format=tiles&tk=5aaa55b9147f14d9e34f00f1a110e9b9"   //在线天地图矢量地图服务(墨卡托投影)
    ,TDT_CIA_W:"http://{s}.tianditu.gov.cn/cia_w/wmts?service=wmts&request=GetTile&version=1.0.0" +
    "&LAYER=cia&tileMatrixSet=w&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}" +
    "&style=default.jpg&tk=5aaa55b9147f14d9e34f00f1a110e9b9"            //在线天地图影像中文标记服务(墨卡托投影)
    ,TDT_CVA_W:"http://{s}.tianditu.gov.cn/cva_w/wmts?service=wmts&request=GetTile&version=1.0.0" +
    "&LAYER=cva&tileMatrixSet=w&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}" +
    "&style=default.jpg&tk=5aaa55b9147f14d9e34f00f1a110e9b9"            //在线天地图矢量中文标记服务(墨卡托投影)
    ,TDT_IMG_C:"http://{s}.tianditu.gov.cn/img_c/wmts?service=wmts&request=GetTile&version=1.0.0" +
    "&LAYER=img&tileMatrixSet=c&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}" +
    "&style=default&format=tiles&tk=5aaa55b9147f14d9e34f00f1a110e9b9"  //在线天地图影像服务地址(经纬度)
    ,TDT_VEC_C:"http://{s}.tianditu.gov.cn/vec_c/wmts?service=wmts&request=GetTile&version=1.0.0" +
    "&LAYER=vec&tileMatrixSet=c&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}" +
    "&style=default&format=tiles&tk=5aaa55b9147f14d9e34f00f1a110e9b9"   //在线天地图矢量地图服务(经纬度)
    ,TDT_CIA_C:"http://{s}.tianditu.gov.cn/cia_c/wmts?service=wmts&request=GetTile&version=1.0.0" +
    "&LAYER=cia&tileMatrixSet=c&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}" +
    "&style=default&format=tiles&tk=5aaa55b9147f14d9e34f00f1a110e9b9"      //在线天地图影像中文标记服务(经纬度)
    ,TDT_CVA_C:"http://{s}.tianditu.gov.cn/cva_c/wmts?service=wmts&request=GetTile&version=1.0.0" +
    "&LAYER=cva&tileMatrixSet=c&TileMatrix={TileMatrix}&TileRow={TileRow}&TileCol={TileCol}" +
    "&style=default&format=tiles&tk=5aaa55b9147f14d9e34f00f1a110e9b9"       //在线天地图矢量中文标记服务(经纬度)
};

    const { Matrix3 ,Cartesian3,Rectangle,HeightmapTessellator,Ellipsoid} = Cesium
    let CesiumMath =Cesium. Math;
    viewer = new Cesium.Viewer('map', {
        terrainProvider: Cesium.createWorldTerrain()
    })
            //天地图影像中文标记服务（经纬度）
            var tdtCva=new Cesium.WebMapTileServiceImageryProvider({
        url: TDTURL_CONFIG.TDT_CIA_C,
        layer: "tdtCva",
        style: "default",
        format:  "tiles",
        tileMatrixSetID: "c",
        subdomains:["t0","t1","t2","t3","t4","t5","t6","t7"],
        tilingScheme:new Cesium.GeographicTilingScheme(),
       	tileMatrixLabels:["1","2","3","4","5","6","7","8","9","10","11","12","13","14","15","16","17","18","19"],
      	maximumLevel:18,
        show: false
    });
    var layers = viewer.imageryLayers;
    layers.addImageryProvider(tdtCva);
    //设置初始位置
    viewer.camera.setView( {
        destination: Cesium.Cartesian3.fromDegrees(108.961727,34.246506)
    } );



    let tiling_scheme= new Cesium.WebMercatorTilingScheme()
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