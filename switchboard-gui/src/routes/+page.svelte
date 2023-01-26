<script>
 import Main from "$lib/Main.svelte";
 import { Tabs, TabList, TabPanel, Tab } from '$lib/tabs.js';
</script>

<h1>Welcome to Switchboard!</h1>

<Tabs>
  <TabList>
    <Tab>Overview</Tab>
    <Tab>Send</Tab>
    <Tab>Receive</Tab>
    <Tab>Transactions</Tab>
    <Tab>Sidechains</Tab>
  </TabList>

  <TabPanel>
    <h2>Overview</h2>
    <div class="row">
      <Main />
    </div>
  </TabPanel>
  <TabPanel>
    <h2>Send</h2>
  </TabPanel>

  <TabPanel>
    <h2>Receive</h2>
  </TabPanel>

  <TabPanel>
    <h2>Transaction</h2>
  </TabPanel>

  <TabPanel>
    <h2>Sidechains</h2>
  </TabPanel>
</Tabs>

<style>
  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }

  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }
</style>
