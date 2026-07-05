<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import * as QRCode from "qrcode";
  import { onMount } from "svelte";
  import "$lib/fluent";
  import { accentBaseColor, SwatchRGB } from "@fluentui/web-components";
  import { parseColorHexRGB } from "@microsoft/fast-colors";

  interface ServerInfo {
    ip: string;
    port: number;
    url: string;
  }

  interface Settings {
    needs_auth: boolean;
    himetric: boolean;
    touch_input: boolean;
    auto_fullscreen: boolean;
    monitor_name: string;
  }

  const APP_VERSION = "v0.1.0";

  let serverInfo: ServerInfo | null = $state(null);
  let settings: Settings | null = $state(null);
  let pin: number | null = $state(null);
  let qrDataUrl = $state("");
  let isLoading = $state(true);
  let copyNotice = $state("");
  let fullscreenNotice = $state("");
  let pollInterval: ReturnType<typeof setInterval> | undefined;

  function getConnectionUrl() {
    if (!serverInfo) return "";
    return settings?.needs_auth && pin != null
      ? `${serverInfo.url}?pin=${pin}`
      : serverInfo.url;
  }

  async function loadState(isBackground = false) {
    if (!isBackground) {
      isLoading = true;
    }

    try {
      const [nextServerInfo, nextSettings, nextPin] = await Promise.all([
        invoke<ServerInfo>("get_server_info"),
        invoke<Settings>("get_settings"),
        invoke<number | null>("get_pin"),
      ]);

      serverInfo = nextServerInfo;
      settings = nextSettings;
      pin = nextPin;
    } finally {
      if (!isBackground) {
        isLoading = false;
      }
    }
  }

  async function loadSystemAccentColor() {
    try {
      const color = await invoke<string>("get_system_accent_color");
      const hexWithoutHash = color.startsWith("#") ? color.slice(1) : color;
      const rgbColor = parseColorHexRGB(hexWithoutHash);
      if (rgbColor) {
        accentBaseColor.withDefault(SwatchRGB.from(rgbColor));
        accentBaseColor.setValueFor(document.documentElement, SwatchRGB.from(rgbColor));
      }
    } catch {
      const rgbColor = parseColorHexRGB("0f6cbd");
      if (rgbColor) {
        accentBaseColor.withDefault(SwatchRGB.from(rgbColor));
        
        accentBaseColor.setValueFor(document.documentElement, SwatchRGB.from(rgbColor));
        
      }
    }
  }

  $effect(() => {
    if (!serverInfo) {
      qrDataUrl = "";
      return;
    }

    QRCode.toDataURL(getConnectionUrl(), {
      width: 220,
      margin: 1,
      color: { dark: "#0f172a", light: "#ffffff" },
    })
      .then((dataUrl) => {
        qrDataUrl = dataUrl;
      })
      .catch(() => {
        qrDataUrl = "";
      });
  });

  async function toggleAuth() {
    const newVal = await invoke<boolean>("toggle_auth");
    if (settings) settings = { ...settings, needs_auth: newVal };
  }

  async function toggleHimetric() {
    const newVal = await invoke<boolean>("toggle_himetric");
    if (settings) settings = { ...settings, himetric: newVal };
  }

  async function toggleTouch() {
    const newVal = await invoke<boolean>("toggle_touch");
    if (settings) settings = { ...settings, touch_input: newVal };
  }

  async function toggleAutoFullscreen() {
    const newVal = await invoke<boolean>("toggle_auto_fullscreen");
    if (settings) settings = { ...settings, auto_fullscreen: newVal };
  }

  async function exitClientFullscreen() {
    try {
      await invoke("exit_client_fullscreen");
      fullscreenNotice = "Exit full screen requested for connected devices.";
    } catch {
      fullscreenNotice = "Could not send the exit full screen request.";
    }

    setTimeout(() => {
      fullscreenNotice = "";
    }, 1800);
  }

  async function switchMonitor() {
    const monitorName = await invoke<string>("switch_monitor");
    if (settings) settings = { ...settings, monitor_name: monitorName };
  }

  async function copyConnectionUrl() {
    const url = getConnectionUrl();
    if (!url) return;

    try {
      await navigator.clipboard.writeText(url);
      copyNotice = "Connection URL copied.";
    } catch {
      copyNotice = "Copy failed. Please copy the URL manually.";
    }

    setTimeout(() => {
      copyNotice = "";
    }, 1800);
  }

  onMount(() => {
    void loadSystemAccentColor();
    void loadState();
    pollInterval = setInterval(() => {
      void loadState(true);
    }, 5000);

    return () => {
      if (pollInterval) clearInterval(pollInterval);
    };
  });
</script>

<svelte:head>
  <title>PenBridge</title>
  <meta name="color-scheme" content="light" />
</svelte:head>

<main class="shell">
  <header class="app-header">
    <div class="title-row">
      <h1>PenBridge</h1>
      <span class="version-chip">{APP_VERSION}</span>
    </div>
  </header>

  {#if serverInfo && settings}
    <div class="dashboard-grid">
      <fluent-card class="panel connection-panel">
        <div class="panel-heading">
          <div>
            <h2>Connection</h2>
            <p>Scan the QR code or open the session URL on your client device.</p>
          </div>
          <fluent-badge appearance={settings.needs_auth ? "accent" : "lightweight"}>
            {settings.needs_auth ? "PIN required" : "Open access"}
          </fluent-badge>
        </div>

        <div class="qr-layout">
          <div class="qr-frame">
            {#if qrDataUrl}
              <img src={qrDataUrl} alt="QR code for PenBridge session" class="qr-code" />
            {:else}
              <div class="qr-placeholder">Generating QR…</div>
            {/if}
          </div>

          <div class="connection-meta">
            <div>
              <div class="eyebrow">Session URL</div>
              <div class="url">{serverInfo.url}</div>
              <p class="muted">Host {serverInfo.ip}:{serverInfo.port}</p>
            </div>

            <div class="button-row">
              <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
              <fluent-button role="button" tabindex="0" appearance="accent" onclick={copyConnectionUrl}>
                Copy URL
              </fluent-button>
            </div>

            {#if copyNotice}
              <p class="inline-note">{copyNotice}</p>
            {/if}
          </div>
        </div>

        {#if settings.needs_auth}
          <div class="pin-strip">
            <span>Current PIN</span>
            <strong>{pin ?? "— — — —"}</strong>
          </div>
        {/if}
      </fluent-card>

      <fluent-card class="panel settings-panel">
        <div class="panel-heading">
          <div>
            <h2>Preferences</h2>
          </div>
        </div>

        <div class="setting-list">
          <div class="setting-row">
            <div class="setting-copy">
              <div class="setting-title">Authentication</div>
              <div class="setting-description">
                Require a short PIN before clients can join the session.
              </div>
            </div>
            <fluent-switch checked={settings.needs_auth} onchange={toggleAuth}>
              Authentication
            </fluent-switch>
          </div>

          <fluent-divider></fluent-divider>

          <div class="setting-row">
            <div class="setting-copy">
              <div class="setting-title">HiMetric input</div>
              <div class="setting-description">
                Use higher-precision coordinate input on supported hardware.
              </div>
            </div>
            <fluent-switch checked={settings.himetric} onchange={toggleHimetric}>
              HiMetric input
            </fluent-switch>
          </div>

          <fluent-divider></fluent-divider>

          <div class="setting-row">
            <div class="setting-copy">
              <div class="setting-title">Touch input</div>
              <div class="setting-description">
                Allow multi-touch gestures from connected client devices.
              </div>
            </div>
            <fluent-switch checked={settings.touch_input} onchange={toggleTouch}>
              Touch input
            </fluent-switch>
          </div>

          <fluent-divider></fluent-divider>

          <div class="setting-row">
            <div class="setting-copy">
              <div class="setting-title">Auto full screen</div>
              <div class="setting-description">
                Ask the web client to enter full screen after connecting or on the first tap.
              </div>
              {#if fullscreenNotice}
                <p class="inline-note">{fullscreenNotice}</p>
              {/if}
            </div>
            <div class="setting-actions">
              <fluent-switch checked={settings.auto_fullscreen} onchange={toggleAutoFullscreen}>
                Auto full screen
              </fluent-switch>
              <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
              <fluent-button role="button" tabindex="0" appearance="outline" onclick={exitClientFullscreen}>
                Exit full screen
              </fluent-button>
            </div>
          </div>

          <fluent-divider></fluent-divider>

          <div class="setting-row monitor-row">
            <div class="setting-copy">
              <div class="setting-title">Target monitor</div>
              <div class="setting-description">{settings.monitor_name}</div>
            </div>
            <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
            <fluent-button role="button" tabindex="0" appearance="outline" onclick={switchMonitor}>
              Use current monitor
            </fluent-button>
          </div>
        </div>
      </fluent-card>
    </div>
  {:else}
    <fluent-card class="loading-panel">
      <fluent-progress-ring></fluent-progress-ring>
      <div>
        <h2>Starting PenBridge</h2>
        <p>{isLoading ? "Loading bridge state…" : "Waiting for the server to respond…"}</p>
      </div>
    </fluent-card>
  {/if}
</main>

<style>
  :global(html, body) {
    margin: 0;
    min-height: 100%;
    background: linear-gradient(180deg, #f7f7f8 0%, #eef1f5 100%);
    color: #1b1a19;
    font-family: "Segoe UI Variable Text", "Segoe UI", sans-serif;
    --system-accent: #0f6cbd;
    --accent-base-color: #0f6cbd;
  }

  :global(body) {
    min-height: 100vh;
  }

  :global(fluent-card),
  :global(fluent-button),
  :global(fluent-switch),
  :global(fluent-badge) {
    font-family: inherit;
  }

  :global(fluent-badge) {
    width: fit-content;
  }

  .shell {
    max-width: 1040px;
    margin: 0 auto;
    padding: 24px 18px 32px;
  }

  .app-header {
    margin-bottom: 16px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  h1,
  h2,
  p {
    margin: 0;
  }

  h1 {
    font-size: clamp(2rem, 3.5vw, 2.4rem);
    line-height: 1.1;
    letter-spacing: -0.02em;
    font-weight: 700;
    color: #1b1a19;
  }

  .version-chip {
    padding: 3px 10px;
    border-radius: 999px;
    background: rgba(15, 108, 189, 0.12);
    color: var(--system-accent);
    font-size: 0.85rem;
    font-weight: 600;
  }

  .url {
    font-size: 1.02rem;
    font-weight: 600;
    word-break: break-all;
    color: var(--system-accent);
  }

  .dashboard-grid {
    display: grid;
    grid-template-columns: minmax(0, 1.15fr) minmax(320px, 0.85fr);
    gap: 16px;
  }

  .panel,
  .loading-panel {
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.86), rgba(248, 250, 252, 0.96));
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 16px;
    padding: 18px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
    backdrop-filter: blur(18px);
    color: #1b1a19;
  }

  .panel-heading {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: flex-start;
    margin-bottom: 16px;
  }

  .panel-heading h2 {
    font-size: 1.08rem;
    margin-bottom: 4px;
    color: #201f1e;
  }

  .panel-heading p,
  .setting-description,
  .muted,
  .status-note,
  .inline-note,
  .loading-panel p {
    color: #605e5c;
  }

  .qr-layout {
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    gap: 18px;
    align-items: center;
  }

  .qr-frame {
    background: #ffffff;
    border: 1px solid #e1dfdd;
    border-radius: 14px;
    min-height: 220px;
    display: grid;
    place-items: center;
    padding: 12px;
  }

  .qr-code {
    width: 100%;
    max-width: 220px;
    border-radius: 10px;
    image-rendering: pixelated;
  }

  .qr-placeholder {
    color: #605e5c;
    font-size: 0.95rem;
  }

  .connection-meta {
    display: grid;
    gap: 14px;
  }

  .eyebrow {
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #605e5c;
    margin-bottom: 6px;
  }

  .muted,
  .status-note,
  .inline-note {
    font-size: 0.9rem;
  }

  .status-note {
    margin-top: 6px;
  }

  .button-row {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    align-items: center;
  }

  .pin-strip {
    margin-top: 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 16px;
    border-radius: 12px;
    background: #eff6fc;
    border: 1px solid #c7e0f4;
  }

  .pin-strip span {
    color: #605e5c;
  }

  .pin-strip strong {
    font-size: 1.25rem;
    letter-spacing: 0.28em;
    color: #0f6cbd;
  }

  .setting-list {
    display: grid;
    gap: 12px;
  }

  .setting-row {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: center;
  }

  .setting-copy {
    display: grid;
    gap: 4px;
  }

  .setting-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
    justify-content: flex-end;
  }

  .setting-title {
    font-weight: 600;
    color: #201f1e;
  }

  .setting-description {
    font-size: 0.92rem;
    line-height: 1.45;
  }

  .monitor-row {
    align-items: flex-start;
  }

  .loading-panel {
    min-height: 160px;
    display: flex;
    gap: 14px;
    align-items: center;
    justify-content: center;
    text-align: left;
  }

  @media (max-width: 820px) {
    .dashboard-grid,
    .qr-layout {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 640px) {
    .shell {
      padding-inline: 12px;
    }

    .panel-heading,
    .setting-row {
      flex-direction: column;
      align-items: stretch;
    }

    .button-row,
    .setting-actions {
      flex-direction: column;
      align-items: stretch;
    }

    .pin-strip {
      gap: 8px;
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
