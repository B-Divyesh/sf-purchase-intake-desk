<script lang="ts">
  import { componentInventory } from './lib/components';
</script>

<svelte:head>
  <meta property="og:title" content="Intake Desk — receiving workflow plan" />
  <meta
    property="og:description"
    content="The planned foundation for a dependable supplier-delivery receiving desk."
  />
  <meta property="og:type" content="website" />
  <meta property="og:url" content="https://purchase-intake-desk.sociobot.in/" />
  <meta name="twitter:card" content="summary" />
</svelte:head>

<a class="skip-link" href="#main">Skip to main content</a>

<header class="site-header">
  <a class="wordmark" href="/" aria-label="Intake Desk planning home">
    <img src="/favicon.svg" alt="" width="32" height="32" />
    <span>Intake Desk</span>
  </a>
  <span class="status-label">Planning scaffold · 00</span>
</header>

<main id="main" tabindex="-1">
  <section class="planning-shell" aria-labelledby="page-title">
    <div class="route-mark" aria-hidden="true">RECEIVING / PLAN</div>
    <div class="copy-block">
      <h1 id="page-title">The receiving workflow is planned, not built.</h1>
      <p class="lead">
        This repository now has the product contract, visual tokens, component inventory, and build tooling for M1.
      </p>
      <p>
        Start with the venture plan in <code>.factory/plan.md</code>. It defines the sample delivery, claims, tests, and milestone boundaries.
      </p>
    </div>

    <aside class="manifest" aria-labelledby="manifest-title">
      <div class="manifest__header">
        <span>SCAFFOLD CONTENTS</span>
        <span aria-hidden="true">20 / 20</span>
      </div>
      <h2 id="manifest-title">Builder starting points</h2>
      <dl>
        <div><dt>Design direction</dt><dd>Dock-stamp constructivism</dd></div>
        <div><dt>Components specified</dt><dd>{componentInventory.length}</dd></div>
        <div><dt>M1 claims specified</dt><dd>9</dd></div>
        <div><dt>Product workflows</dt><dd>0 built</dd></div>
      </dl>
    </aside>
  </section>
</main>

<footer>
  <span>Intake Desk planning scaffold</span>
  <span>Built by Param Factory</span>
</footer>
