<script setup lang="ts">
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"

const greetMsg = ref("")
const name = ref("")

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value })
}
</script>

<template>
	<div class="main">
		<div class="container left"></div>
		<div class="container middle"></div>
		<div class="container right"></div>
		<div class="container bottom"></div>
	</div>
</template>

<style lang="scss">
body {
	margin: 0.25em;
}

.main {
	display: flex;
	flex-direction: row;
	flex-wrap: wrap;
	gap: 0.25em;
	> * {
		padding: 0.25em;
		background-color: pink;
		border-radius: 0.25em;
	}
	.middle, .left, .right {
		height: 90vh;
	}
	.middle {
		width: 100%;
	}
	.left {
		width: 27%;
		display: none;
	}
	.right {
		display: none;
	}
	.bottom {
		width: 100%;
		height: 6.5vh;
	}
}
</style>