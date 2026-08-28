<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { appendEvent, verifyEventChain } from './lib/domain/audit';
  import { compareDecimal, differenceLabel, parseDecimal } from './lib/domain/decimal';
  import { receiptCsv } from './lib/domain/csv';
  import type { DemoState, PurchaseOrderLine } from './lib/domain/model';
  import { routeMeta } from './lib/domain/routes';
  import { demoRepository } from './lib/storage/demoRepository';

  const productionOrigin = 'https://purchase-intake-desk.sociobot.in';
  const buildSha = import.meta.env.VITE_BUILD_SHA || 'dev';
  let pathname = window.location.pathname;
  let meta = routeMeta(pathname);
  let state: DemoState | null = null;
  let loading = false;
  let busy = false;
  let notice = '';
  let error = '';
  let search = '';
  let scanCode = '';
  let connectionOnline = navigator.onLine;
  let finalizeDialog: HTMLDialogElement;
  let correctionDialog: HTMLDialogElement;
  let resetDialog: HTMLDialogElement;
  let correctionReason = '';
  let chainVerified = false;
  let lastDialogTrigger: HTMLElement | null = null;

  $: meta = routeMeta(pathname);
  $: isDemo = pathname === '/demo' || pathname.startsWith('/demo/');
  $: canonical = `${productionOrigin}${pathname === '/404' ? '/404' : pathname}`;
  $: filtered = search.trim()
    ? state && `${state.poNumber} ${state.supplier}`.toLowerCase().includes(search.trim().toLowerCase())
    : Boolean(state);

  onMount(() => {
    const query = new URLSearchParams(window.location.search);
    if (pathname === '/' && query.get('demo') === '1') {
      history.replaceState({}, '', '/demo?demo=1');
      pathname = '/demo';
    }
    if (pathname === '/404.html') {
      history.replaceState({}, '', '/404');
      pathname = '/404';
    }
    if (pathname === '/demo' || pathname.startsWith('/demo/')) void loadDemo();

    const pop = () => {
      pathname = window.location.pathname;
      if (pathname === '/demo' || pathname.startsWith('/demo/')) void loadDemo();
      void focusPageTitle(false);
    };
    const online = () => { connectionOnline = true; notice = 'Connection restored. Demo data stays on this device.'; };
    const offline = () => { connectionOnline = false; notice = 'Saved on this device. Keep this tab until you reconnect.'; };
    window.addEventListener('popstate', pop);
    window.addEventListener('online', online);
    window.addEventListener('offline', offline);
    void registerOfflineShell();
    return () => {
      window.removeEventListener('popstate', pop);
      window.removeEventListener('online', online);
      window.removeEventListener('offline', offline);
    };
  });

  async function registerOfflineShell() {
    if (!('serviceWorker' in navigator)) return;
    try {
      const registration = await navigator.serviceWorker.register('/sw.js');
      await navigator.serviceWorker.ready;
      const worker = registration.active;
      const urls = ['/', '/index.html', '/favicon.svg', '/apple-touch-icon.svg', '/og-image.svg'];
      for (const entry of performance.getEntriesByType('resource')) {
        const url = new URL(entry.name);
        if (url.origin === location.origin) urls.push(`${url.pathname}${url.search}`);
      }
      if (worker) {
        const channel = new MessageChannel();
        const done = new Promise<void>((resolve) => { channel.port1.onmessage = () => resolve(); });
        worker.postMessage({ type: 'CACHE_URLS', urls: [...new Set(urls)] }, [channel.port2]);
        await done;
      }
      Object.assign(window, { __INTAKE_SW_READY__: true });
    } catch {
      notice = 'Offline setup did not finish. Reload once while connected, then try again.';
    }
  }

  async function loadDemo() {
    if (state) return;
    loading = true;
    error = '';
    try {
      state = await demoRepository.load();
      chainVerified = state.events.length ? await verifyEventChain(state.events) : true;
    } catch {
      error = 'The sample could not open on this device. Allow local storage, then retry.';
    } finally {
      loading = false;
    }
  }

  async function focusPageTitle(scroll = true) {
    await tick();
    const heading = document.querySelector<HTMLElement>('main h1');
    heading?.focus({ preventScroll: !scroll });
    const announcer = document.getElementById('route-announcer');
    if (announcer) announcer.textContent = heading?.textContent ?? '';
  }

  function navigate(event: MouseEvent, destination: string) {
    if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
    event.preventDefault();
    history.pushState({}, '', destination);
    pathname = new URL(destination, location.origin).pathname;
    if (pathname === '/demo' || pathname.startsWith('/demo/')) void loadDemo();
    window.scrollTo(0, 0);
    void focusPageTitle();
  }

  async function save(next: DemoState, message: string) {
    state = next;
    try {
      await demoRepository.save(next);
      notice = message;
      error = '';
    } catch {
      error = 'This change was not saved. Free device storage, then try again.';
    }
  }

  function updateLine(lineId: string, patch: Partial<PurchaseOrderLine>) {
    if (!state || state.status === 'finalized') return;
    const lines = state.lines.map((line) => line.id === lineId ? { ...line, ...patch } : line);
    void save({ ...state, status: 'draft', lines }, 'Saved on this device.');
  }

  function adjust(line: PurchaseOrderLine, amount: number) {
    const current = Number(line.receivedEach);
    if (!Number.isFinite(current)) return;
    updateLine(line.id, { receivedEach: String(Math.max(0, current + amount)) });
  }

  function scan(event: SubmitEvent) {
    event.preventDefault();
    const code = scanCode.trim().toUpperCase();
    const found = state?.lines.find((line) => line.code === code);
    if (!found) {
      error = `${code || 'That code'} is not on NB-1047. Check the packing list or choose a line below.`;
      return;
    }
    error = '';
    notice = `${found.code} found. Enter its received count.`;
    void tick().then(() => document.getElementById(`received-${found.id}`)?.focus());
    scanCode = '';
  }

  async function tryCamera() {
    error = '';
    if (!navigator.mediaDevices?.getUserMedia) {
      notice = 'Camera scanning is not supported here. Type the item code instead.';
      document.getElementById('scan-code')?.focus();
      return;
    }
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'environment' } });
      stream.getTracks().forEach((track) => track.stop());
      notice = 'Camera access works. This demo uses typed or keyboard scanner codes.';
    } catch {
      notice = 'Camera access was not allowed. Type the item code instead.';
      document.getElementById('scan-code')?.focus();
    }
  }

  function openDialog(dialog: HTMLDialogElement, trigger: EventTarget | null) {
    lastDialogTrigger = trigger instanceof HTMLElement ? trigger : null;
    dialog.showModal();
  }

  function closeDialog(dialog: HTMLDialogElement) {
    dialog.close();
    lastDialogTrigger?.focus();
  }

  function validationMessage(): string {
    if (!state) return 'The sample is still loading.';
    const invalid = state.lines.find((line) => !parseDecimal(line.receivedEach));
    if (invalid) return `${invalid.code} needs a received count of zero or more.`;
    const damaged = state.lines.find((line) => line.condition === 'damaged' && (!parseDecimal(line.damagedEach) || compareDecimal(line.damagedEach, '0') === 0));
    if (damaged) return `${damaged.code} needs the number of damaged items.`;
    return '';
  }

  function requestFinalize(event: MouseEvent) {
    const problem = validationMessage();
    if (problem) {
      error = `${problem} Enter the count, then finalize again.`;
      void tick().then(() => document.getElementById('error-summary')?.focus());
      return;
    }
    openDialog(finalizeDialog, event.currentTarget);
  }

  async function finalizeReceipt() {
    if (!state || busy) return;
    busy = true;
    const events = await appendEvent(state.events, {
      type: 'finalized', at: state.receivedAt, actor: state.receivedBy,
      summary: 'Finalized receipt with 2 each short and 1 seal damaged.',
    });
    const next = { ...state, status: 'finalized' as const, revision: state.revision + 1, events };
    await save(next, 'Receipt finalized. The original counts are now read-only.');
    chainVerified = await verifyEventChain(events);
    busy = false;
    finalizeDialog.close();
    history.pushState({}, '', `/demo/receipts/${next.receiptId}`);
    pathname = `/demo/receipts/${next.receiptId}`;
    window.scrollTo(0, 0);
    await focusPageTitle();
  }

  async function recordCorrection() {
    if (!state || !correctionReason.trim() || busy) return;
    busy = true;
    const events = await appendEvent(state.events, {
      type: 'corrected', at: '2026-08-28T10:06:00', actor: state.receivedBy,
      summary: `Correction recorded: ${correctionReason.trim()}`,
    });
    await save({ ...state, events }, 'Correction added as a new event. The final receipt was not rewritten.');
    chainVerified = await verifyEventChain(events);
    correctionReason = '';
    busy = false;
    correctionDialog.close();
    lastDialogTrigger?.focus();
  }

  function downloadCsv() {
    if (!state) return;
    const blob = new Blob([receiptCsv(state)], { type: 'text/csv;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'intake-desk-NB-1047-receipt-v1.csv';
    anchor.click();
    URL.revokeObjectURL(url);
    notice = 'Receipt CSV downloaded with three item rows.';
  }

  async function resetDemo() {
    if (busy) return;
    busy = true;
    try {
      state = await demoRepository.reset();
      chainVerified = true;
      notice = 'Demo reset. NB-1047 is back to the original sample.';
      error = '';
      resetDialog.close();
      history.pushState({}, '', '/demo?demo=1');
      pathname = '/demo';
      await focusPageTitle();
    } catch (reason) {
      error = reason instanceof Error ? reason.message : 'The demo could not be reset. Reload and try again.';
      resetDialog.close();
    } finally { busy = false; }
  }

  async function startForReal(event: MouseEvent) {
    event.preventDefault();
    try { state = await demoRepository.reset(); } catch { /* public page remains available */ }
    history.pushState({}, '', '/#pricing');
    pathname = '/';
    await tick();
    document.getElementById('pricing')?.scrollIntoView();
    await focusPageTitle(false);
  }
</script>

<svelte:head>
  <title>{meta.title}</title>
  <meta name="description" content={meta.description} />
  <link rel="canonical" href={canonical} />
  <meta property="og:title" content={meta.title} />
  <meta property="og:description" content={meta.description} />
  <meta property="og:type" content="website" />
  <meta property="og:url" content={canonical} />
  <meta property="og:image" content={`${productionOrigin}/og-image.svg`} />
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content={meta.title} />
  <meta name="twitter:description" content={meta.description} />
  <meta name="twitter:image" content={`${productionOrigin}/og-image.svg`} />
</svelte:head>

<a class="skip-link" href="#main">Skip to main content</a>
<div id="route-announcer" class="visually-hidden" aria-live="polite"></div>

<header class="site-header">
  <a class="wordmark" href="/" onclick={(event) => navigate(event, '/')} aria-label="Intake Desk home">
    <img src="/favicon.svg" alt="" width="32" height="32" /><span>Intake Desk</span>
  </a>
  <nav aria-label="Main navigation">
    <a href="/demo" aria-current={isDemo ? 'page' : undefined} onclick={(event) => navigate(event, '/demo')}>Demo</a>
    <a href="/#pricing">Pricing</a>
    <a href="/privacy" aria-current={pathname === '/privacy' ? 'page' : undefined} onclick={(event) => navigate(event, '/privacy')}>Privacy</a>
  </nav>
</header>

{#if isDemo}
  <aside class="demo-banner" aria-label="Demo status">
    <strong>Demo — sample data, nothing is saved</strong><span>Changes stay in this browser.</span>
    <div class="banner-actions"><button class="text-button" type="button" onclick={(event) => openDialog(resetDialog, event.currentTarget)}>Reset demo</button><a href="/#pricing" onclick={startForReal}>Start for real</a></div>
  </aside>
{/if}

{#if notice}<div class="live-notice" role="status"><span>{notice}</span><button class="notice-close" type="button" aria-label="Dismiss status" onclick={() => notice = ''}>×</button></div>{/if}

<main id="main" tabindex="-1">
  {#if pathname === '/'}
    <section class="hero" aria-labelledby="page-title">
      <div class="hero-copy">
        <div class="route-mark">RECEIVING / 01</div>
        <h1 id="page-title" tabindex="-1">Check deliveries against the purchase order.</h1>
        <p class="lead">For small receiving teams that need a clear record before the supplier van leaves.</p>
        <div class="hero-action"><a class="button primary" href="/demo" onclick={(event) => navigate(event, '/demo')}>Try it with sample data</a><span>Opens one ready PO. No account.</span></div>
        <ul class="plain-facts" aria-label="Product facts"><li>Keeps counts through a network drop.</li><li>Exports receipt CSV.</li><li>Dock plan: $149 per site each month.</li></ul>
      </div>
      <aside class="manifest-preview" aria-labelledby="preview-title">
        <div class="manifest-header"><span>WEST YARD</span><span>NB-1047</span></div>
        <h2 id="preview-title">Northline Bearings</h2><p class="muted">Packing list NL-8821</p>
        <div class="preview-row"><span>BRG-6204</span><strong>120 / 120</strong><span class="status success">Matched</span></div>
        <div class="preview-row"><span>BLT-A42</span><strong>46 / 48</strong><span class="status danger">2 short</span></div>
        <div class="preview-row"><span>SEAL-28</span><strong>24 / 24</strong><span class="status danger">1 damaged</span></div>
        <a class="route-link" href="/demo/receive/po-nb-1047" onclick={(event) => navigate(event, '/demo/receive/po-nb-1047')}>Open this sample <span aria-hidden="true">→</span></a>
      </aside>
    </section>

    <section class="landing-section product-preview" aria-labelledby="product-title">
      <div class="section-index">02 / LIVE RECEIPT</div><h2 id="product-title">See the difference while you count</h2>
      <p>Expected, received, and damaged quantities stay together. A shortage becomes a receipt record, not a note to copy later.</p>
      <div class="count-strip" aria-label="Example count for BLT-A42"><div><span>Item</span><strong>BLT-A42</strong></div><div><span>Expected</span><strong>48 each</strong></div><div><span>Received</span><strong>46 each</strong></div><div class="difference"><span>Difference</span><strong>2 each short</strong></div></div>
    </section>

    <section class="landing-section" aria-labelledby="works-title">
      <div class="section-index">03 / HOW IT WORKS</div><h2 id="works-title">Finish a receipt in three steps</h2>
      <ol class="steps"><li><strong>Find the purchase order.</strong><span>Scan or type an item code from the packing list.</span></li><li><strong>Count what arrived.</strong><span>See shortages, extra items, and damage beside each line.</span></li><li><strong>Finalize the receipt.</strong><span>Keep the original event and add later corrections separately.</span></li></ol>
    </section>

    <section class="landing-section boundaries" aria-labelledby="limits-title">
      <div class="section-index">04 / CLEAR BOUNDARIES</div><h2 id="limits-title">A receiving desk, not an ERP</h2>
      <p>Intake Desk does not run accounting, inventory planning, or supplier payments. The demo uses sample data in your browser.</p><a href="/privacy" onclick={(event) => navigate(event, '/privacy')}>Read the privacy notice</a>
    </section>

    <section id="pricing" class="landing-section pricing" aria-labelledby="pricing-title">
      <div><div class="section-index">05 / DOCK PLAN</div><h2 id="pricing-title">One receiving site</h2><p>Production purchase orders, receipts, discrepancies, exports, and staff access.</p></div>
      <div class="price"><strong>$149</strong><span>USD per site each month</span><p>Accounts open in the next milestone.</p></div>
    </section>
  {:else if isDemo}
    {#if loading}
      <section class="state-panel" aria-labelledby="page-title"><div class="route-mark">DEMO / LOADING</div><h1 id="page-title" tabindex="-1">Loading the receiving desk.</h1><p>Loading open purchase orders…</p><div class="loading-rule" aria-hidden="true"></div></section>
    {:else if error && !state}
      <section class="state-panel" aria-labelledby="page-title"><div class="route-mark">DEMO / STORAGE</div><h1 id="page-title" tabindex="-1">The sample could not open.</h1><p>{error}</p><button class="button secondary" type="button" onclick={loadDemo}>Retry the sample</button></section>
    {:else if state}
      {#if pathname === '/demo'}
        <section class="app-heading" aria-labelledby="page-title"><div class="route-mark">WEST YARD / INBOX</div><h1 id="page-title" tabindex="-1">Find the delivery.</h1><p class="lead">Scan the packing list or search the supplier and purchase order.</p></section>
        <section class="search-sheet" aria-label="Purchase order search"><label for="po-search">Purchase order or supplier</label><input id="po-search" type="search" bind:value={search} placeholder="Try NB-1047" /></section>
        <section aria-labelledby="due-title">
          <div class="section-heading"><h2 id="due-title">Due today</h2><span>1 purchase order</span></div>
          {#if filtered}
            <a class="po-row" href="/demo/purchase-orders/{state.purchaseOrderId}" onclick={(event) => navigate(event, `/demo/purchase-orders/${state!.purchaseOrderId}`)}>
              <div><span class="eyebrow">{state.poNumber}</span><strong>{state.supplier}</strong></div>
              <dl><div><dt>Lines</dt><dd>3</dd></div><div><dt>List</dt><dd>{state.packingList}</dd></div><div><dt>State</dt><dd>{state.status === 'finalized' ? 'Received' : 'Due'}</dd></div></dl><span class="row-action">Review <span aria-hidden="true">→</span></span>
            </a>
          {:else}
            <div class="state-panel compact"><h3>No matching purchase orders</h3><p>Clear the search to see NB-1047.</p><button class="button secondary" onclick={() => search = ''}>Clear search</button></div>
          {/if}
        </section>
      {:else if pathname.startsWith('/demo/purchase-orders/')}
        <section class="app-heading split-heading" aria-labelledby="page-title"><div><div class="route-mark">PURCHASE ORDER / {state.poNumber}</div><h1 id="page-title" tabindex="-1">Review {state.poNumber}.</h1><p class="lead">{state.supplier} · Packing list {state.packingList}</p></div><span class="status-stamp">{state.status === 'finalized' ? 'Received' : 'Due today'}</span></section>
        <section class="manifest-sheet" aria-labelledby="manifest-lines-title">
          <div class="section-heading"><h2 id="manifest-lines-title">Expected lines</h2><span>Revision {state.revision}</span></div>
          <div class="manifest-table" role="table" aria-label="Purchase order lines"><div class="manifest-row manifest-labels" role="row"><span role="columnheader">Item</span><span role="columnheader">Ordered</span><span role="columnheader">Expected</span></div>{#each state.lines as line}<div class="manifest-row" role="row"><span role="cell"><strong>{line.code}</strong><small>{line.description}</small></span><span role="cell">{line.ordered} {line.orderUnit === 'case' ? 'cases' : 'each'}</span><span role="cell">{line.expectedEach} each</span></div>{/each}</div>
          <div class="sheet-actions">{#if state.status === 'finalized'}<a class="button primary" href="/demo/receipts/{state.receiptId}" onclick={(event) => navigate(event, `/demo/receipts/${state!.receiptId}`)}>Open finalized receipt</a>{:else}<a class="button primary" href="/demo/receive/{state.purchaseOrderId}" onclick={(event) => navigate(event, `/demo/receive/${state!.purchaseOrderId}`)}>Count this delivery</a>{/if}<a href="/demo" onclick={(event) => navigate(event, '/demo')}>Back to purchase orders</a></div>
        </section>
      {:else if pathname.startsWith('/demo/receive/')}
        <section class="app-heading" aria-labelledby="page-title"><div class="route-mark">RECEIVE / {state.poNumber}</div><h1 id="page-title" tabindex="-1">Count the delivery.</h1><p class="lead">{state.supplier} · Packing list {state.packingList}</p></section>
        <div class:offline={!connectionOnline} class="connection-strip" role="status"><strong>{connectionOnline ? 'Saved on this device' : 'Offline · saved on this device'}</strong><span>{connectionOnline ? 'Demo changes are not sent anywhere.' : 'Keep this tab. You can finish the sample now.'}</span></div>
        {#if error}<div id="error-summary" class="error-summary" role="alert" tabindex="-1"><strong>Check this receipt</strong><span>{error}</span></div>{/if}
        <section class="scan-sheet" aria-labelledby="scan-title">
          <div><h2 id="scan-title">Find an item</h2><p>Use a keyboard scanner or type the code.</p></div>
          <form onsubmit={scan}><label for="scan-code">Item code</label><div class="field-action"><input id="scan-code" autocomplete="off" bind:value={scanCode} placeholder="BRG-6204" /><button class="button secondary" type="submit">Find item</button></div></form>
          <button class="text-button" type="button" onclick={tryCamera}>Check camera fallback</button>
        </section>
        <section aria-labelledby="count-title">
          <div class="section-heading"><h2 id="count-title">Received counts</h2><span>3 lines · quantities in each</span></div>
          <div class="receipt-lines">
            {#each state.lines as line}
              {@const comparison = parseDecimal(line.receivedEach) ? compareDecimal(line.receivedEach, line.expectedEach) : 0}
              <article class:has-difference={comparison !== 0 || line.condition === 'damaged'} class="receipt-line" id="row-{line.id}">
                <header><div><span class="eyebrow">{line.code}</span><h3>{line.description}</h3></div><span class:success={comparison === 0 && line.condition === 'good'} class:danger={comparison !== 0 || line.condition === 'damaged'} class="status">{!parseDecimal(line.receivedEach) ? 'Count needed' : line.condition === 'damaged' ? `${line.damagedEach} damaged` : differenceLabel(line.expectedEach, line.receivedEach)}</span></header>
                <p class="conversion">{line.orderUnit === 'case' ? `${line.ordered} cases × ${line.unitsPerCase} = ` : ''}<strong>{line.expectedEach} each expected</strong></p>
                <div class="count-grid">
                  <div class="quantity-field"><label for="received-{line.id}">Received, each</label><div class="stepper"><button type="button" aria-label={`Remove one ${line.code}`} onclick={() => adjust(line, -1)} disabled={state.status === 'finalized'}>−</button><input id="received-{line.id}" inputmode="decimal" value={line.receivedEach} aria-invalid={!parseDecimal(line.receivedEach)} oninput={(event) => updateLine(line.id, { receivedEach: event.currentTarget.value })} disabled={state.status === 'finalized'} /><button type="button" aria-label={`Add one ${line.code}`} onclick={() => adjust(line, 1)} disabled={state.status === 'finalized'}>+</button></div></div>
                  <div><label for="condition-{line.id}">Condition</label><select id="condition-{line.id}" value={line.condition} onchange={(event) => updateLine(line.id, { condition: event.currentTarget.value as PurchaseOrderLine['condition'], damagedEach: event.currentTarget.value === 'damaged' && line.damagedEach === '0' ? '1' : line.damagedEach })} disabled={state.status === 'finalized'}><option value="good">Good</option><option value="damaged">Damaged</option></select></div>
                  {#if line.condition === 'damaged'}<div><label for="damaged-{line.id}">Damaged, each</label><input id="damaged-{line.id}" inputmode="decimal" value={line.damagedEach} oninput={(event) => updateLine(line.id, { damagedEach: event.currentTarget.value })} disabled={state.status === 'finalized'} /></div>{/if}
                </div>
                {#if comparison !== 0 || line.condition === 'damaged'}<label for="note-{line.id}">Discrepancy note</label><textarea id="note-{line.id}" rows="2" value={line.note} oninput={(event) => updateLine(line.id, { note: event.currentTarget.value })} disabled={state.status === 'finalized'}></textarea>{/if}
              </article>
            {/each}
          </div>
        </section>
        <section class="evidence-tray" aria-labelledby="evidence-title"><div><h2 id="evidence-title">Sample evidence</h2><p>One dock note is attached to this local sample.</p></div><figure><svg viewBox="0 0 160 100" role="img" aria-label="Sample sketch showing the damaged seal"><rect x="1" y="1" width="158" height="98" fill="none" stroke="currentColor" stroke-width="2"/><circle cx="58" cy="50" r="25" fill="none" stroke="currentColor" stroke-width="7"/><path d="M76 30l9-13M80 38l16-5" stroke="currentColor" stroke-width="3"/><path d="M105 30h37M105 43h26M105 56h31" stroke="currentColor" stroke-width="2"/></svg><figcaption>Sam’s sample damage sketch · SEAL-28</figcaption></figure></section>
        <div class="finalize-bar"><div><strong>Ready to record</strong><span>2 each short · 1 seal damaged</span></div>{#if state.status === 'finalized'}<a class="button primary" href="/demo/receipts/{state.receiptId}" onclick={(event) => navigate(event, `/demo/receipts/${state!.receiptId}`)}>Open finalized receipt</a>{:else}<button class="button primary" type="button" onclick={requestFinalize}>Finalize receipt</button>{/if}</div>
      {:else if pathname.startsWith('/demo/receipts/')}
        <section class="app-heading split-heading" aria-labelledby="page-title"><div><div class="route-mark">RECEIPT / {state.poNumber}</div><h1 id="page-title" tabindex="-1">Receipt {state.poNumber}.</h1><p class="lead">Finalized for {state.supplier} at 09:40.</p></div><span class="received-stamp">Received</span></section>
        {#if state.status !== 'finalized'}
          <div class="error-summary"><strong>No finalized receipt yet</strong><span>Count the delivery and finalize it first.</span><a href="/demo/receive/{state.purchaseOrderId}" onclick={(event) => navigate(event, `/demo/receive/${state!.purchaseOrderId}`)}>Return to the count</a></div>
        {:else}
          <section class="receipt-summary" aria-labelledby="summary-title"><div class="section-heading"><h2 id="summary-title">Recorded result</h2><span>{state.receivedBy} · 28 Aug 2026, 09:40</span></div><div class="count-strip"><div><span>Lines</span><strong>3</strong></div><div><span>Short</span><strong>2 each</strong></div><div><span>Damaged</span><strong>1 each</strong></div><div class="difference"><span>State</span><strong>Partial delivery</strong></div></div><div class="sheet-actions"><a class="button secondary" href="/demo/discrepancies/{state.discrepancyId}" onclick={(event) => navigate(event, `/demo/discrepancies/${state!.discrepancyId}`)}>Open discrepancy</a><button class="button secondary" type="button" onclick={downloadCsv}>Export receipt CSV</button><button class="text-button" type="button" onclick={(event) => openDialog(correctionDialog, event.currentTarget)}>Record a correction</button></div></section>
          <section class="timeline" aria-labelledby="timeline-title"><div class="section-heading"><h2 id="timeline-title">Receipt history</h2><span>{chainVerified ? 'Hash chain verified' : 'History needs review'}</span></div><ol>{#each state.events as event}<li id={event.id}><div><span class="event-sequence">{String(event.sequence).padStart(2, '0')}</span><strong>{event.type === 'finalized' ? 'Receipt finalized' : 'Correction recorded'}</strong></div><p>{event.summary}</p><dl><div><dt>Actor</dt><dd>{event.actor}</dd></div><div><dt>Time</dt><dd>{event.at.slice(11, 16)}</dd></div><div><dt>Hash</dt><dd><code>{event.hash.slice(0, 12)}…</code></dd></div></dl></li>{/each}</ol></section>
        {/if}
      {:else if pathname.startsWith('/demo/discrepancies/')}
        <section class="app-heading split-heading" aria-labelledby="page-title"><div><div class="route-mark">DISCREPANCY / {state.poNumber}</div><h1 id="page-title" tabindex="-1">Northline delivery exception.</h1><p class="lead">{state.supplier} · Receipt {state.poNumber}</p></div><span class="status-stamp danger-plate">Partial</span></section>
        <section class="discrepancy-summary" aria-labelledby="discrepancy-title"><h2 id="discrepancy-title">2 each short · 1 seal damaged</h2><p>This sample record keeps both issues together for the supplier follow-up.</p><div class="issue-list"><article><span class="eyebrow">SHORT / BLT-A42</span><h3>Two belts did not arrive</h3><dl><div><dt>Expected</dt><dd>48 each</dd></div><div><dt>Received</dt><dd>46 each</dd></div></dl><p>Two belts missing from one opened case.</p></article><article><span class="eyebrow">DAMAGED / SEAL-28</span><h3>One seal is damaged</h3><dl><div><dt>Received</dt><dd>24 each</dd></div><div><dt>Damaged</dt><dd>1 each</dd></div></dl><p>One seal has a split outer lip.</p></article></div></section>
        <section class="evidence-record" aria-labelledby="record-evidence-title"><h2 id="record-evidence-title">Evidence kept with the receipt</h2><p>Sample dock note: Sam checked the opened belt case and marked the split seal before finalizing.</p><p class="muted">Supplier messages are not sent from the M1 demo.</p><a href="/demo/receipts/{state.receiptId}" onclick={(event) => navigate(event, `/demo/receipts/${state!.receiptId}`)}>Back to receipt history</a></section>
      {:else}
        <section class="state-panel" aria-labelledby="page-title"><div class="route-mark">DEMO / 404</div><h1 id="page-title" tabindex="-1">This demo record is not here.</h1><p>Reset the sample or return to its purchase order.</p><a class="button primary" href="/demo" onclick={(event) => navigate(event, '/demo')}>Open the demo</a></section>
      {/if}
    {/if}
  {:else if pathname === '/privacy'}
    <article class="legal" aria-labelledby="page-title"><div class="route-mark">LEGAL / PRIVACY</div><h1 id="page-title" tabindex="-1">Your demo changes stay here.</h1><p class="lead">The M1 sample stores changes only in this browser. It does not send receiving data to Intake Desk or another service.</p><h2>What the demo stores</h2><p>The demo keeps the West Yard sample, your changed counts, and its receipt history in IndexedDB named <code>intake-desk:demo:v1</code>. Reset demo deletes and rebuilds that sample.</p><h2>What leaves this device</h2><p>The demo loads only same-origin app files. It has no analytics, advertising, account, billing, email, or AI request.</p><h2>Production accounts</h2><p>Production accounts and site storage are not available in M1. This notice will change before those features open.</p><p>Questions: <a href="mailto:privacy@sociobot.in">privacy@sociobot.in</a>.</p></article>
  {:else if pathname === '/terms'}
    <article class="legal" aria-labelledby="page-title"><div class="route-mark">LEGAL / TERMS</div><h1 id="page-title" tabindex="-1">Terms for the sample desk.</h1><p class="lead">Use this M1 demo to evaluate the receiving flow. Do not enter confidential or real supplier data.</p><h2>Sample use</h2><p>The sample is provided as-is for evaluation. Reset removes your local changes and restores NB-1047.</p><h2>No production service yet</h2><p>The Dock plan is not open in M1. No payment or subscription starts from this site.</p><h2>Data and exports</h2><p>You may download the sample CSV. You remain responsible for any information you enter or export.</p><p>Questions: <a href="mailto:support@sociobot.in">support@sociobot.in</a>.</p></article>
  {:else}
    <section class="state-panel not-found" aria-labelledby="page-title"><div class="route-mark">ROUTE / 404</div><h1 id="page-title" tabindex="-1">This page missed the dock.</h1><p>The address does not match an Intake Desk page.</p><div class="sheet-actions"><a class="button primary" href="/" onclick={(event) => navigate(event, '/')}>Return home</a><a href="/demo" onclick={(event) => navigate(event, '/demo')}>Open the sample</a></div></section>
  {/if}
</main>

<footer>
  <div><strong>Intake Desk</strong><span>Intake Desk records supplier deliveries and their exceptions.</span></div>
  <nav aria-label="Footer navigation"><a href="/privacy" onclick={(event) => navigate(event, '/privacy')}>Privacy</a><a href="/terms" onclick={(event) => navigate(event, '/terms')}>Terms</a><a href="https://sociobot.in">Built by Param Factory <span class="visually-hidden">(external site)</span></a></nav>
  <span class="build-id">Build {buildSha.slice(0, 12)} · M1</span>
</footer>

<dialog bind:this={finalizeDialog} onclose={() => lastDialogTrigger?.focus()} aria-labelledby="finalize-dialog-title">
  <form method="dialog" onsubmit={(event) => event.preventDefault()}><div class="dialog-mark">FINAL RECEIPT</div><h2 id="finalize-dialog-title">Finalize NB-1047?</h2><p>This records 2 each short and 1 seal damaged. Later changes become correction events.</p><div class="dialog-actions"><button class="button secondary" type="button" onclick={() => closeDialog(finalizeDialog)}>Keep counting</button><button class="button primary" type="button" onclick={finalizeReceipt} disabled={busy}>{busy ? 'Finalizing…' : 'Finalize receipt'}</button></div></form>
</dialog>

<dialog bind:this={correctionDialog} onclose={() => lastDialogTrigger?.focus()} aria-labelledby="correction-dialog-title">
  <form method="dialog" onsubmit={(event) => { event.preventDefault(); void recordCorrection(); }}><div class="dialog-mark">NEW EVENT</div><h2 id="correction-dialog-title">Record a correction</h2><p>The finalized event stays unchanged. This reason is added after it.</p><label for="correction-reason">Correction reason</label><textarea id="correction-reason" rows="3" bind:value={correctionReason} required placeholder="Example: Supplier confirmed the missing belts"></textarea><div class="dialog-actions"><button class="button secondary" type="button" onclick={() => closeDialog(correctionDialog)}>Cancel</button><button class="button primary" type="submit" disabled={!correctionReason.trim() || busy}>{busy ? 'Recording…' : 'Record correction'}</button></div></form>
</dialog>

<dialog bind:this={resetDialog} onclose={() => lastDialogTrigger?.focus()} aria-labelledby="reset-dialog-title">
  <form method="dialog" onsubmit={(event) => { event.preventDefault(); void resetDemo(); }}><div class="dialog-mark">RESET SAMPLE</div><h2 id="reset-dialog-title">Reset NB-1047?</h2><p>This discards all demo changes on this device and restores the three original lines.</p><div class="dialog-actions"><button class="button secondary" type="button" onclick={() => closeDialog(resetDialog)}>Keep changes</button><button class="button primary" type="submit" disabled={busy}>{busy ? 'Resetting…' : 'Reset demo'}</button></div></form>
</dialog>
