<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open as openFileDialog } from '@tauri-apps/plugin-dialog';
  import { listen } from '@tauri-apps/api/event';

  type StoredLogTemplate = {
    name: string;
    pattern: string;
    field_mapping: Record<string, string>;
  };

  type TemplateCatalog = {
    built_in: StoredLogTemplate[];
    user_defined: StoredLogTemplate[];
  };

  type TemplatePreviewResponse = {
    matched: boolean;
    template_name: string;
    event: Record<string, unknown> | null;
    error: string | null;
  };

  type TemplateMatchSummary = {
    template_name: string;
    matched_lines: number;
    total_lines: number;
    success_rate: number;
  };

  type ParseProgress = {
    percentage: number;
    phase: string;
    entries_parsed: number;
  };

  type FileInfo = {
    id: number;
    path: string;
    name: string;
    size: number;
    entry_count: number;
  };

  type LogEntryView = {
    id: number;
    timestamp: number;
    level: string;
    logger: string;
    summary: string;
    raw: string;
    template_name: string;
    extra: Record<string, string>;
  };

  type ParseSessionInfo = {
    mode: string;
    active_template: string;
    detection: TemplateMatchSummary[];
  };

  const defaultTemplate = (): StoredLogTemplate => ({
    name: '',
    pattern: '^(?P<timestamp>\\d{4}-\\d{2}-\\d{2} \\d{2}:\\d{2}:\\d{2}) (?P<level>\\w+) (?P<source>\\S+) - (?P<message>.+)$',
    field_mapping: {
      timestamp: 'timestamp',
      level: 'level',
      source: 'source',
      message: 'message'
    }
  });

  let catalog: TemplateCatalog = { built_in: [], user_defined: [] };
  let currentFile: FileInfo | null = null;
  let logs: LogEntryView[] = [];
  let selectedLog: LogEntryView | null = null;
  let parseSession: ParseSessionInfo | null = null;
  let loading = false;
  let loadingProgress = 0;
  let loadingPhase = '';
  let statusMessage = '请选择日志文件';
  let showMenu = false;
  let showTemplateModal = false;
  let showTemplateLibrary = false;
  let preview: TemplatePreviewResponse | null = null;
  let previewLoading = false;
  let previewTimer: ReturnType<typeof setTimeout> | null = null;
  let draftTemplate: StoredLogTemplate = defaultTemplate();
  let previewSample = '2024-01-02 03:04:05 INFO auth.service - login ok';
  let selectedTemplateName = 'auto-detect';
  let importInput: HTMLInputElement | null = null;

  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

  const formatTime = (timestamp: number) => {
    if (!timestamp) return '—';
    return new Date(timestamp).toLocaleString();
  };

  async function refreshTemplates() {
    if (!isTauri) return;
    catalog = await invoke<TemplateCatalog>('get_templates');
  }

  async function refreshParseSession() {
    if (!isTauri) return;
    parseSession = await invoke<ParseSessionInfo | null>('get_parse_session');
  }

  async function loadEntries() {
    if (!currentFile || !isTauri) return;
    logs = await invoke<LogEntryView[]>('get_entries', {
      fileId: currentFile.id,
      offset: 0,
      limit: 200
    });
    selectedLog = logs[0] ?? null;
  }

  async function openLogFile() {
    if (!isTauri) {
      statusMessage = '模板预览与文件解析需要在 Tauri 桌面环境中运行。';
      return;
    }

    const selected = await openFileDialog({
      multiple: false,
      filters: [{ name: 'Logs', extensions: ['log', 'txt', 'json', '1'] }]
    });

    if (!selected || Array.isArray(selected)) return;

    currentFile = await invoke<FileInfo>('open_file', { path: selected });
    statusMessage = `已打开 ${currentFile.name}`;
    loading = true;
    logs = [];
    selectedLog = null;

    await invoke<number>('parse_log', {
      fileId: currentFile.id,
      templateName: selectedTemplateName
    });

    loading = false;
    await refreshParseSession();
    await loadEntries();
    statusMessage = `解析完成，已载入 ${logs.length} 条日志预览`;
  }

  async function saveCurrentTemplate() {
    if (!isTauri) return;
    await invoke('save_template', { template: draftTemplate });
    showTemplateModal = false;
    showTemplateLibrary = true;
    draftTemplate = defaultTemplate();
    await refreshTemplates();
    statusMessage = '模板已保存到 sled';
  }

  async function deleteTemplate(name: string) {
    if (!isTauri) return;
    await invoke('delete_template', { templateName: name });
    if (selectedTemplateName === name) {
      selectedTemplateName = 'auto-detect';
    }
    await refreshTemplates();
    statusMessage = `已删除模板 ${name}`;
  }

  function triggerImport() {
    importInput?.click();
    showMenu = false;
  }

  async function handleImportTemplates(event: Event) {
    if (!isTauri) return;
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    const content = await file.text();
    const payload = JSON.parse(content) as StoredLogTemplate[];
    for (const template of payload) {
      await invoke('save_template', { template });
    }
    await refreshTemplates();
    statusMessage = `已导入 ${payload.length} 个模板`;
    input.value = '';
  }

  function exportTemplates() {
    const blob = new Blob([JSON.stringify(catalog.user_defined, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'log-templates.json';
    anchor.click();
    URL.revokeObjectURL(url);
    showMenu = false;
  }

  function applyTemplateSelection(name: string) {
    selectedTemplateName = name;
    showMenu = false;
    statusMessage = name === 'auto-detect' ? '已切换到自动检测' : `已选择模板 ${name}`;
  }

  function createTemplateFrom(existing?: StoredLogTemplate) {
    draftTemplate = existing
      ? JSON.parse(JSON.stringify(existing))
      : defaultTemplate();
    preview = null;
    showTemplateModal = true;
    showTemplateLibrary = false;
    showMenu = false;
  }

  $: if (showTemplateModal && isTauri) {
    if (previewTimer) clearTimeout(previewTimer);
    previewTimer = setTimeout(async () => {
      previewLoading = true;
      try {
        preview = await invoke<TemplatePreviewResponse>('preview_template', {
          template: draftTemplate,
          sampleLine: previewSample
        });
      } catch (error) {
        preview = {
          matched: false,
          template_name: draftTemplate.name || 'draft',
          event: null,
          error: error instanceof Error ? error.message : String(error)
        };
      } finally {
        previewLoading = false;
      }
    }, 250);
  }

  onMount(() => {
    if (!isTauri) return;

    let unlisten = () => {};

    (async () => {
      await refreshTemplates();
      unlisten = await listen<ParseProgress>('parse_progress', (event) => {
        loadingProgress = event.payload.percentage;
        loadingPhase = `${event.payload.phase} · ${event.payload.entries_parsed} 条`;
        if (event.payload.percentage >= 100) {
          loading = false;
        }
      });
    })();

    return () => {
      unlisten();
      if (previewTimer) clearTimeout(previewTimer);
    };
  });
</script>

<svelte:head>
  <title>Large Log Viewer</title>
</svelte:head>

<div class="page">
  <input bind:this={importInput} class="hidden-input" type="file" accept="application/json" onchange={handleImportTemplates} />

  <header class="toolbar">
    <div>
      <h1>日志解析模板系统</h1>
      <p>Regex 命名捕获组 + 自动检测 + 流式解析</p>
    </div>
    <div class="toolbar-actions">
      <button class="primary" onclick={openLogFile}>打开日志</button>
      <div class="menu-wrap">
        <button class="menu-button" onclick={() => (showMenu = !showMenu)}>[日志格式 ▼]</button>
        {#if showMenu}
          <div class="menu">
            <button onclick={() => applyTemplateSelection('auto-detect')}>自动检测</button>
            <div class="menu-section-title">内置模板</div>
            {#each catalog.built_in as template}
              <button onclick={() => applyTemplateSelection(template.name)}>{template.name}</button>
            {/each}
            <div class="menu-section-title">我的模板</div>
            {#if catalog.user_defined.length === 0}
              <div class="menu-empty">暂无自定义模板</div>
            {/if}
            {#each catalog.user_defined as template}
              <button onclick={() => applyTemplateSelection(template.name)}>{template.name}</button>
            {/each}
            <button onclick={() => createTemplateFrom()}>新建模板</button>
            <button onclick={() => { showTemplateLibrary = true; showMenu = false; }}>我的模板</button>
            <button onclick={triggerImport}>导入</button>
            <button onclick={exportTemplates}>导出</button>
          </div>
        {/if}
      </div>
    </div>
  </header>

  <section class="status-grid">
    <div class="card">
      <span class="label">当前模板</span>
      <strong>{selectedTemplateName}</strong>
    </div>
    <div class="card">
      <span class="label">解析状态</span>
      <strong>{loading ? `${loadingProgress.toFixed(1)}%` : '待命'}</strong>
      <small>{loadingPhase || statusMessage}</small>
    </div>
    <div class="card">
      <span class="label">当前文件</span>
      <strong>{currentFile?.name ?? '未打开'}</strong>
      <small>{currentFile ? `${currentFile.entry_count} 条 / ${(currentFile.size / 1024).toFixed(1)} KB` : '支持大文件流式读取'}</small>
    </div>
  </section>

  {#if parseSession}
    <section class="panel">
      <div class="panel-header">
        <h2>自动检测结果</h2>
        <span>{parseSession.mode} / active: {parseSession.active_template}</span>
      </div>
      {#if parseSession.detection.length > 0}
        <div class="detection-list">
          {#each parseSession.detection as item}
            <div class="detection-item">
              <strong>{item.template_name}</strong>
              <span>{item.matched_lines}/{item.total_lines} 命中</span>
              <span>{(item.success_rate * 100).toFixed(1)}%</span>
            </div>
          {/each}
        </div>
      {:else}
        <div class="empty">手动模板模式下不显示检测统计。</div>
      {/if}
    </section>
  {/if}

  <section class="content-grid">
    <div class="panel">
      <div class="panel-header">
        <h2>日志预览</h2>
        <span>仅加载前 200 条结果，避免一次性渲染大文件</span>
      </div>
      {#if logs.length === 0}
        <div class="empty">打开日志后将在此显示解析结果。</div>
      {:else}
        <div class="log-list">
          {#each logs as log}
            <button class:selected={selectedLog?.id === log.id} class="log-row" onclick={() => (selectedLog = log)}>
              <div>
                <strong>{log.level}</strong>
                <span>{formatTime(log.timestamp)}</span>
              </div>
              <div class="muted">{log.logger} · {log.template_name}</div>
              <div class="summary">{log.summary}</div>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="panel detail-panel">
      <div class="panel-header">
        <h2>日志详情</h2>
        <span>标准字段 + extra 字段</span>
      </div>
      {#if selectedLog}
        <div class="detail-grid">
          <div><span class="label">时间</span><strong>{formatTime(selectedLog.timestamp)}</strong></div>
          <div><span class="label">级别</span><strong>{selectedLog.level}</strong></div>
          <div><span class="label">来源</span><strong>{selectedLog.logger}</strong></div>
          <div><span class="label">模板</span><strong>{selectedLog.template_name}</strong></div>
          <div class="detail-block full"><span class="label">消息</span><pre>{selectedLog.summary}</pre></div>
          <div class="detail-block full"><span class="label">extra</span><pre>{JSON.stringify(selectedLog.extra, null, 2)}</pre></div>
        </div>
      {:else}
        <div class="empty">请选择左侧日志记录。</div>
      {/if}
    </div>
  </section>

  {#if showTemplateLibrary}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="overlay" role="presentation" onclick={() => (showTemplateLibrary = false)} onkeydown={(event) => event.key === 'Escape' && (showTemplateLibrary = false)}>
      <div class="modal library-modal" role="dialog" aria-modal="true" tabindex="-1" onclick={(event) => event.stopPropagation()}>
        <div class="panel-header">
          <h2>我的模板</h2>
          <button class="ghost" onclick={() => (showTemplateLibrary = false)}>关闭</button>
        </div>
        {#if catalog.user_defined.length === 0}
          <div class="empty">还没有自定义模板，点击“新建模板”开始。</div>
        {:else}
          <div class="template-library">
            {#each catalog.user_defined as template}
              <div class="template-card">
                <div>
                  <strong>{template.name}</strong>
                  <pre>{template.pattern}</pre>
                </div>
                <div class="card-actions">
                  <button class="ghost" onclick={() => createTemplateFrom(template)}>编辑</button>
                  <button class="danger" onclick={() => deleteTemplate(template.name)}>删除</button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}

  {#if showTemplateModal}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="overlay" role="presentation" onclick={() => (showTemplateModal = false)} onkeydown={(event) => event.key === 'Escape' && (showTemplateModal = false)}>
      <div class="modal" role="dialog" aria-modal="true" tabindex="-1" onclick={(event) => event.stopPropagation()}>
        <div class="panel-header">
          <h2>新建模板</h2>
          <button class="ghost" onclick={() => (showTemplateModal = false)}>关闭</button>
        </div>

        <label>
          模板名称
          <input bind:value={draftTemplate.name} placeholder="例如：nginx-access" />
        </label>

        <label>
          正则输入框（支持 ?P&lt;field&gt; 命名捕获）
          <textarea bind:value={draftTemplate.pattern} rows="6"></textarea>
        </label>

        <div class="mapping-grid">
          <label>timestamp<input bind:value={draftTemplate.field_mapping.timestamp} /></label>
          <label>level<input bind:value={draftTemplate.field_mapping.level} /></label>
          <label>source<input bind:value={draftTemplate.field_mapping.source} /></label>
          <label>message<input bind:value={draftTemplate.field_mapping.message} /></label>
        </div>

        <label>
          测试日志输入框
          <textarea bind:value={previewSample} rows="4"></textarea>
        </label>

        <section class="preview-box">
          <div class="panel-header">
            <h3>实时解析结果</h3>
            <span>{previewLoading ? '调用 Rust backend...' : 'JSON 预览'}</span>
          </div>
          <pre>{JSON.stringify(preview, null, 2)}</pre>
        </section>

        <div class="actions-row">
          <button class="ghost" onclick={() => (showTemplateModal = false)}>取消</button>
          <button class="primary" onclick={saveCurrentTemplate}>保存模板</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  :global(body) {
    margin: 0;
    font-family: Inter, system-ui, sans-serif;
    background: #0f172a;
    color: #e2e8f0;
  }
  .page {
    min-height: 100vh;
    padding: 24px;
    background: linear-gradient(180deg, #0f172a 0%, #111827 100%);
  }
  .toolbar, .panel-header, .toolbar-actions, .status-grid, .content-grid, .actions-row, .card-actions {
    display: flex;
    gap: 12px;
  }
  .toolbar, .panel-header {
    justify-content: space-between;
    align-items: center;
  }
  .toolbar { margin-bottom: 18px; }
  h1, h2, h3, p { margin: 0; }
  .status-grid, .content-grid { align-items: stretch; }
  .status-grid { margin-bottom: 18px; }
  .content-grid { display: grid; grid-template-columns: 1.2fr 0.8fr; }
  .card, .panel, .modal {
    background: rgba(15, 23, 42, 0.88);
    border: 1px solid rgba(148, 163, 184, 0.2);
    border-radius: 16px;
    padding: 16px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.25);
  }
  .card { flex: 1; display: flex; flex-direction: column; gap: 6px; }
  .label, .muted, small, .menu-section-title { color: #94a3b8; font-size: 0.85rem; }
  button, input, textarea {
    font: inherit;
    border-radius: 10px;
    border: 1px solid rgba(148, 163, 184, 0.25);
    background: rgba(30, 41, 59, 0.95);
    color: inherit;
  }
  button { padding: 10px 14px; cursor: pointer; }
  input, textarea { padding: 10px 12px; width: 100%; }
  .primary { background: #2563eb; }
  .danger { background: #b91c1c; }
  .ghost { background: transparent; }
  .menu-wrap { position: relative; }
  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + 8px);
    min-width: 220px;
    display: flex;
    flex-direction: column;
    padding: 10px;
    border-radius: 14px;
    background: #111827;
    border: 1px solid rgba(148, 163, 184, 0.25);
    z-index: 5;
  }
  .menu button { text-align: left; background: transparent; }
  .menu-empty, .empty { color: #94a3b8; padding: 14px 0; }
  .detection-list, .template-library, .log-list { display: flex; flex-direction: column; gap: 10px; }
  .detection-item, .template-card {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: 10px;
    padding: 12px;
    border-radius: 12px;
    background: rgba(30, 41, 59, 0.65);
  }
  .log-row {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    text-align: left;
    padding: 12px;
  }
  .log-row.selected { outline: 2px solid #38bdf8; }
  .summary, pre { white-space: pre-wrap; word-break: break-word; }
  .detail-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
  .detail-block.full { grid-column: 1 / -1; }
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.7);
    display: grid;
    place-items: center;
    padding: 24px;
    z-index: 10;
  }
  .modal { width: min(860px, 92vw); max-height: 90vh; overflow: auto; }
  .library-modal { width: min(900px, 92vw); }
  label { display: flex; flex-direction: column; gap: 8px; margin-top: 14px; }
  .mapping-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
  .preview-box { margin-top: 16px; padding: 12px; border-radius: 12px; background: rgba(30, 41, 59, 0.6); }
  .hidden-input { display: none; }
  @media (max-width: 900px) {
    .content-grid { grid-template-columns: 1fr; }
    .status-grid { flex-direction: column; }
    .mapping-grid, .detail-grid { grid-template-columns: 1fr; }
  }
</style>
