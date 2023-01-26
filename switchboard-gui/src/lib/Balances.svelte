<script lang="ts">
 import Amount from "$lib/Amount.svelte";
 // amount in Satoshi
 export let main: number = 0;
 export let zcash: number = 0;
</script>
<h2>Balances</h2>
<div class="row">
    <span>
        Main: <Amount amount={main} />
    </span>
</div>
<div class="row">
    <span>
        Zcash: <Amount amount={zcash} />
    </span>
</div>
