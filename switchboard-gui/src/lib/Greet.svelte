<script lang="ts">
 import Balances from "$lib/Balances.svelte";
 import { onMount } from "svelte";
 import { invoke } from "@tauri-apps/api/tauri";

 let balances = {};
 let hash = "";

  async function get_balances() {
    balances = await invoke("get_balances");
  }

 async function generate(amount: number) {
   hash = await invoke("generate", { amount });
   await get_balances();
 }

 onMount(async () => {
   await new Promise(r => setTimeout(r, 1000));
   await get_balances();
 });
</script>

<div>
  <div class="row">
    <button on:click={get_balances}>Update</button>
    <button on:click={() => generate(10000)}>Generate</button>
  </div>
  <Balances {...balances} />
  <div class="row">
    {hash}
  </div>
</div>
