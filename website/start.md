---
outline: deep
---


# Start

## Clone
```bash
git clone https://github.com/catnuko/houtu
```

## Run In Native
```bash
# run
cd houtu-app
cargo run
```

## Run In Browser With trunk
```bash
cd houtu-app
# Yes, you can skip it
cargo install trunk wasm-bindgen-cli
# Start the service and the console will give the service address，http://127.0.0.1:8080
trunk serve
```

## Run In Browser With wasm-server-runner
```bash
cd houtu-app
cargo run --target wasm32-unknown-unknown
wasm-server-runner ../target/wasm32-unknown-unknown/debug/houtu-app.wasm
```

## Build
```bash
# build
cd houtu-app
cargo build
```

## Write Document
```bash
cd website
pnpm install
pnpm docs:dev
```