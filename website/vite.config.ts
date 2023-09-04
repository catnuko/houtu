import { defineConfig } from 'vite'
import wasm from "vite-plugin-wasm";
// https://vitejs.dev/config/
export default defineConfig({
  root:"./",
  plugins: [
    wasm()
  ],
})
