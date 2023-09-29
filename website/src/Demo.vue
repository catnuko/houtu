<template>
    <div class="map-container">
        <canvas id="rgis"></canvas>
    </div>
</template>
<script setup lang="ts">
import * as wasm from "../../houtu/pkg/";
import {onMounted} from 'vue'
onMounted(()=>{
    wasm.run()
})
</script>
<style scoped>
.map-container{
    width: fit-content;
    margin: auto;
}
</style>