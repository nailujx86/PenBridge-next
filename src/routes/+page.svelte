<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import QRCode from "qrcode";
  import { onMount } from "svelte";

  interface ServerInfo {
    ip: string;
    port: number;
    url: string;
  }

  interface Settings {
    needs_auth: boolean;
    himetric: boolean;
    touch_input: boolean;
    monitor_name: string;
  }

  let serverInfo: ServerInfo | null = $state(null);
  let settings: Settings | null = $state(null);
  let pin: number | null = $state(null);
  let qrDataUrl: string = $state("");
  let pollInterval: ReturnType<typeof setInterval>;

  async function loadState() {
    serverInfo = await invoke<ServerInfo>("get_server_info");
    settings = await invoke<Settings>("get_settings");
    pin = await invoke<number | null>("get_pin");

    if (serverInfo) {
      qrDataUrl = await QRCode.toDataURL(serverInfo.url, {
        width: 200,
        margin: 2,
        color: { dark: "#000000", light: "#ffffff" },
      });
    }
  }

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

  async function switchMonitor() {
    const monitorName = await invoke<string>("switch_monitor");
    if (settings) settings = { ...settings, monitor_name: monitorName };
  }

  async function refreshPin() {
    pin = await invoke<number | null>("get_pin");
  }

  onMount(() => {
    loadState();
    pollInterval = setInterval(refreshPin, 2000);
    return () => clearInterval(pollInterval);
  });
</script>

<main>
  <div class="header">
    <h1>PenBridge</h1>
    <span class="version">v0.1.0</span>
  </div>

  {#if serverInfo}
    <div class="connection-section">
      <div class="qr-container">
        {#if qrDataUrl}
          <img src={qrDataUrl} alt="QR Code" class="qr-code" />
        {/if}
      </div>

      <div class="connection-info">
        <p class="url">{serverInfo.url}</p>
        <p class="hint">Scan the QR code or navigate to the URL above on your pen-enabled device.</p>
      </div>
    </div>

    {#if settings}
      <div class="settings-section">
        <h2>Settings</h2>

        <div class="toggle-row">
          <div class="toggle-info">
            <span class="toggle-label">Authentication</span>
            <span class="toggle-desc">Require PIN to connect</span>
          </div>
          <button
            class="toggle-btn"
            class:active={settings.needs_auth}
            onclick={toggleAuth}
          >
            {settings.needs_auth ? "ON" : "OFF"}
          </button>
        </div>

        {#if settings.needs_auth && pin}
          <div class="pin-display">
            Current PIN: <span class="pin-value">{pin}</span>
          </div>
        {/if}

        <div class="toggle-row">
          <div class="toggle-info">
            <span class="toggle-label">HiMetric Input</span>
            <span class="toggle-desc">Higher precision on supported devices</span>
          </div>
          <button
            class="toggle-btn"
            class:active={settings.himetric}
            onclick={toggleHimetric}
          >
            {settings.himetric ? "ON" : "OFF"}
          </button>
        </div>

        <div class="toggle-row">
          <div class="toggle-info">
            <span class="toggle-label">Touch Input</span>
            <span class="toggle-desc">Accept multi-touch from clients</span>
          </div>
          <button
            class="toggle-btn"
            class:active={settings.touch_input}
            onclick={toggleTouch}
          >
            {settings.touch_input ? "ON" : "OFF"}
          </button>
        </div>

        <div class="toggle-row">
          <div class="toggle-info">
            <span class="toggle-label">Target Monitor</span>
            <span class="toggle-desc">{settings.monitor_name}</span>
          </div>
          <button class="action-btn" onclick={switchMonitor}>
            Switch to Current
          </button>
        </div>
      </div>
    {/if}
  {:else}
    <div class="loading">Starting server...</div>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    background-color: #1a1a2e;
    color: #eee;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  main {
    max-width: 480px;
    margin: 0 auto;
    padding: 24px 20px;
  }

  .header {
    display: flex;
    align-items: baseline;
    gap: 10px;
    margin-bottom: 24px;
  }

  h1 {
    margin: 0;
    font-size: 28px;
    font-weight: 700;
    color: #e0e0ff;
  }

  .version {
    font-size: 14px;
    color: #666;
  }

  .connection-section {
    background: #16213e;
    border-radius: 12px;
    padding: 24px;
    text-align: center;
    margin-bottom: 20px;
  }

  .qr-container {
    margin-bottom: 16px;
  }

  .qr-code {
    border-radius: 8px;
    image-rendering: pixelated;
  }

  .url {
    font-size: 18px;
    font-weight: 600;
    color: #4a9eff;
    margin: 0 0 8px;
    word-break: break-all;
  }

  .hint {
    font-size: 13px;
    color: #888;
    margin: 0;
    line-height: 1.4;
  }

  .settings-section {
    background: #16213e;
    border-radius: 12px;
    padding: 20px;
  }

  h2 {
    margin: 0 0 16px;
    font-size: 18px;
    font-weight: 600;
    color: #c0c0ff;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 0;
    border-bottom: 1px solid #1f2b47;
  }

  .toggle-row:last-child {
    border-bottom: none;
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .toggle-label {
    font-size: 15px;
    font-weight: 500;
  }

  .toggle-desc {
    font-size: 12px;
    color: #777;
  }

  .toggle-btn {
    min-width: 56px;
    padding: 6px 14px;
    border: 2px solid #333;
    border-radius: 6px;
    background: #2a2a3e;
    color: #999;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .toggle-btn.active {
    background: #1a4a1a;
    border-color: #2d7a2d;
    color: #8f8;
  }

  .toggle-btn:hover {
    border-color: #555;
  }

  .action-btn {
    padding: 6px 14px;
    border: 2px solid #333;
    border-radius: 6px;
    background: #2a2a3e;
    color: #aaa;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .action-btn:hover {
    background: #3a3a5e;
    border-color: #555;
    color: #ddd;
  }

  .pin-display {
    background: #1a2a4e;
    border-radius: 8px;
    padding: 10px 16px;
    margin: 8px 0 4px;
    font-size: 14px;
    color: #aaa;
  }

  .pin-value {
    font-size: 20px;
    font-weight: 700;
    color: #ffa;
    letter-spacing: 4px;
  }

  .loading {
    text-align: center;
    padding: 60px 0;
    color: #666;
    font-size: 16px;
  }
</style>
