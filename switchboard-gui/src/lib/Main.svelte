<script lang="ts">
 import BlockCounts from "$lib/BlockCounts.svelte";
 import Balances from "$lib/Balances.svelte";
 import { onMount } from "svelte";
 import { invoke } from "@tauri-apps/api/tauri";

 let balances = {};
 let block_counts = {};

 async function get_balances() {
     balances = await invoke("get_balances");
 }

 async function get_block_counts() {
     block_counts = await invoke("get_block_counts");
 }

 async function update() {
   await get_balances();
   await get_block_counts();
 }

 async function generate(amount: number) {
   await invoke("generate", { amount });
   await update();
 }

 onMount(async () => {
   await new Promise(r => setTimeout(r, 1000));
   await update();
 });
</script>

<div>
  <div class="row">
    <button on:click={update}>Update</button>
    <button on:click={() => generate(10000)}>Generate</button>
  </div>
  <Balances {...balances} />
  <BlockCounts {...block_counts} />
</div>
