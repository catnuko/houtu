<div align="center">

  <h1><code>后土</code></h1>

  <strong>基于webgpu的高性能的真实地球渲染引擎</strong>

  <h3>
    <a href="#">暂无文档</a>
    <span> | </span>
    <a href="https://imdodo.com/s/211509">dodo交流群-后土地球</a>
  </h3>
</div>

# **注意：本项目还在试验阶段，请斟酌使用。**

## 截图

四叉树调度

![瓦片网格](./website/public/assets/tutieshi_640x344_11s.gif)

## 🔥介绍
使用bevy作为渲染引擎，面向web端，目标成为国内一流的开源免费三维地球渲染引擎。

项目极早期阶段，望与诸位才子共建未来。

野蛮时代将去，未来是科技的未来。
## 🚀特性
1. bevy作为渲染引擎，高度可拆卸，定制自己需要的功能。
2. 使用wasm+webgpu渲染web端，主打高性能高颜值。
3. 参考cesium，具备实用性，GIS图形的精确性。
## 路线
详情查看仓库的[Projects](https://github.com/users/catnuko/projects/1)
1. - [x] 3d globe
2. - [x] 相机控制
3. - [ ] 基本几何图形，多边形，折线，点，圆，球，椭球等形状
4. - [ ] 栅格瓦片图层
5. - [ ] 矢量瓦片图层
6. - [ ] 倾斜摄影模型
7. - [ ] 地形
## 💻开发
```bash
// 运行程序
cd ./houtu
cargo run

// 构建程序
// 暂无法构建

// 运行网站（暂无内容）
cd www
pnpm install
pnpm dev
```

## 💓贡献
佛系参与，强烈欢迎。👏👏👏
