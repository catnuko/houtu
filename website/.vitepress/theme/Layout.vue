<script setup lang="ts">
import { useData } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import Demo from './Demo.vue'
const { Layout } = DefaultTheme
// https://vitepress.dev/reference/runtime-api#usedata
const data = useData()
const { site, frontmatter,page } = data
console.log(data)
</script>

<template>
 <Layout>
  <template #page-top v-if="page.filePath==='demo.md'">
    <Demo></Demo>
  </template>
 </Layout>
</template>

<style>
::view-transition-old(root),
::view-transition-new(root) {
  animation: none;
  mix-blend-mode: normal;
}

::view-transition-old(root),
.dark::view-transition-new(root) {
  z-index: 1;
}

::view-transition-new(root),
.dark::view-transition-old(root) {
  z-index: 9999;
}

.VPSwitchAppearance {
  width: 22px !important;
}

.VPSwitchAppearance .check {
  transform: none !important;
}
</style>