<template>
    <canvas id="rgis"></canvas>
</template>
<script setup lang="ts">
import * as wasm from "../../houtu/pkg/";
import {onMounted} from 'vue'
onMounted(()=>{
    wasm.run()
})
</script>