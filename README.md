<div align="center">

  <h1><code>houtu</code></h1>

  <strong>webgpu based high performance 3D earth rendering engine</strong>

  <h3>
    <a href="https://github.com/catnuko/houtu/blob/master/README_ZH.md">中文</a>
    <span> | </span>
    <a href="https://github.com/catnuko/houtu/discussions">discussions</a>
  </h3>
</div>

# ⚠️This is very much work in progress, please use it with discretion.

## Screenshot

Layers of the web Mercator projection，Tile resources from [omniscale](https://maps.omniscale.net),thanks.

![瓦片网格](./www/public/assets/i53pd-qxcsr.gif)

## 🔥Introduction
Use bevy as rendering engine, open source free 3D Earth rendering engine for web side.

Very early stage of the project, look forward to working with you to build the future.

## 🚀Feature
1. pluggable:with bevy as the rendering engine, plugins can be flexibly customized.
2. web-oriented:render to browser using wasm+webgpu.
3. precision:reference cesium, with practical, GIS graphic accuracy.
## 🌎Roadmap
[projects](https://github.com/users/catnuko/projects/1)
1. - [x] 3d globe
2. - [x] 相机控制
3. - [ ] 基本几何图形，多边形，折线，点，圆，球，椭球等形状
4. - [x] 栅格瓦片图层，支持wgs84和web墨卡托投影的切片地图
5. - [ ] 矢量瓦片图层
6. - [ ] 倾斜摄影模型
7. - [ ] 地形
## 📖Documentation
1. chinese development experience，[理论3D地球](https://www.taihe.one/tag/%E7%90%86%E8%AE%BA%E5%9C%B0%E7%90%83)

## 💻Development
```bash
# run
cd houtu-app
cargo run

# It doesn't work in the browser for now

# Run in a browser using a trunk
cd houtu-app
cargo install trunk wasm-bindgen-cli # Yes, you can skip it
trunk serve # Start the service and the console will give the service address，http://127.0.0.1:8080

# Run it in a browser with wasm-server-runner
cd houtu-app
cargo run --target wasm32-unknown-unknown
wasm-server-runner ../target/wasm32-unknown-unknown/debug/houtu-app.wasm

# build
cd houtu-app
cargo build

// Running website (No content)
cd www
pnpm install
pnpm dev
```

## 💓Contribution
Welcome to participate in development.👏👏👏