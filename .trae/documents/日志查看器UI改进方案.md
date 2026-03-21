# 日志查看器 UI 改进方案（修订版）

## 需求概述

根据用户需求，对日志查看器界面进行以下改进：

1. 去除右上角提示文字，改为设置和帮助按钮
2. 底部状态栏动态显示状态信息
3. 级别筛选增加"级别"标签
4. 搜索框后增加搜索按钮
5. 增加搜索范围下拉框
6. 增加搜索模式下拉框（模糊/精确/正则）
7. 新增设置界面（主题颜色、内容编码）
8. 后端支持文件编码检测和设置持久化
9. 更新 README

---

## UI 布局变化预览

### 修改后:
```
┌──────────────────────────────────────────────────────────────────────┐
│ LogViewer V1.0.0                                      [⚙️设置] [❓帮助] │
├──────────────────────────────────────────────────────────────────────┤
│ 筛选 [全部时间 ▼]  [全部范围 ▼] [模糊 ▼]  │  [搜索条件...    ] [搜索] │
├──────────────────────────────────────────────────────────────────────┤
│ 级别 [所有] [致命] [错误] [警告] [信息] [调试] [跟踪] [其它]          │
├──────────────────────────────────────────────────────────────────────┤
│ ...                                                                  │
├──────────────────────────────────────────────────────────────────────┤
│ 已加载 1234 条日志，用时 567ms  [清理缓存] [刷新] 列表数: 1234       │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 详细修改方案

### 1. TitleBar.svelte - 标题栏改造

**文件**: [src/lib/components/TitleBar.svelte](src/lib/components/TitleBar.svelte)

**修改内容**:
- 移除右上角提示文字
- 增加"设置"和"帮助"按钮
- 新增 Props 用于打开设置弹窗

**修改后代码**:
```svelte
<script lang="ts">
  const appVersion = '1.0.0';
  
  interface Props {
    onOpenSettings?: () => void;
    onOpenHelp?: () => void;
  }
  
  let { onOpenSettings, onOpenHelp }: Props = $props();
</script>

<header class="title-bar">
  <div class="title">
    <span class="app-name">LogViewer</span>
    <span class="version">V{appVersion}</span>
  </div>
  <div class="window-controls">
    <button class="control-btn" onclick={onOpenSettings} title="设置">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
      </svg>
      设置
    </button>
    <button class="control-btn" onclick={onOpenHelp} title="帮助">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/>
        <line x1="12" y1="17" x2="12.01" y2="17"/>
      </svg>
      帮助
    </button>
  </div>
</header>

<style>
  /* ... 原有样式 ... */
  
  .window-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .control-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    font-size: 12px;
    color: var(--color-text-secondary);
    background-color: transparent;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  
  .control-btn:hover {
    color: var(--color-text-primary);
    background-color: var(--color-bg-tertiary);
    border-color: var(--color-accent);
  }
</style>
```

---

### 2. SettingsModal.svelte - 设置弹窗（新增）

**文件**: [src/lib/components/SettingsModal.svelte](src/lib/components/SettingsModal.svelte)（新建）

**功能**:
- 主题颜色选择：深色/浅色
- 内容编码选择：自动检测/UTF-8/ANSI/Unicode LE/Unicode BE
- 恢复默认/取消/应用按钮

**完整代码**:
```svelte
<script lang="ts">
  import { settingsStore, type AppSettings } from '$lib/stores/settingsStore';
  
  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }
  
  let { isOpen, onClose }: Props = $props();
  
  let localSettings = $state<AppSettings>({
    theme: 'dark',
    encoding: 'auto'
  });
  
  $effect(() => {
    if (isOpen) {
      localSettings = { ...settingsStore.getSettings() };
    }
  });
  
  const themeOptions = [
    { value: 'dark', label: '深色' },
    { value: 'light', label: '浅色' }
  ];
  
  const encodingOptions = [
    { value: 'auto', label: '自动检测文件编码' },
    { value: 'utf-8', label: 'UTF-8编码' },
    { value: 'ansi', label: 'ANSI编码' },
    { value: 'utf-16le', label: 'Unicode LE编码' },
    { value: 'utf-16be', label: 'Unicode BE编码' }
  ];
  
  function handleReset() {
    localSettings = {
      theme: 'dark',
      encoding: 'auto'
    };
  }
  
  function handleCancel() {
    onClose();
  }
  
  function handleApply() {
    settingsStore.updateSettings(localSettings);
    onClose();
  }
</script>

{#if isOpen}
  <div class="modal-overlay" onclick={onClose}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>设置</h2>
        <button class="close-btn" onclick={onClose}>×</button>
      </div>
      
      <div class="modal-body">
        <div class="setting-group">
          <label class="setting-label">主题颜色:</label>
          <div class="radio-group">
            {#each themeOptions as option}
              <label class="radio-item">
                <input 
                  type="radio" 
                  name="theme" 
                  value={option.value}
                  bind:group={localSettings.theme}
                />
                <span class="radio-label">{option.label}</span>
              </label>
            {/each}
          </div>
        </div>
        
        <div class="setting-group">
          <label class="setting-label">内容编码:</label>
          <div class="radio-group">
            {#each encodingOptions as option}
              <label class="radio-item">
                <input 
                  type="radio" 
                  name="encoding" 
                  value={option.value}
                  bind:group={localSettings.encoding}
                />
                <span class="radio-label">{option.label}</span>
              </label>
            {/each}
          </div>
        </div>
      </div>
      
      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={handleReset}>恢复默认</button>
        <button class="btn btn-secondary" onclick={handleCancel}>取消</button>
        <button class="btn btn-primary" onclick={handleApply}>应用</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
  }
  
  .modal-content {
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    min-width: 400px;
    max-width: 500px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  }
  
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border);
  }
  
  .modal-header h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 20px;
    color: var(--color-text-secondary);
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }
  
  .close-btn:hover {
    color: var(--color-text-primary);
  }
  
  .modal-body {
    padding: 20px;
  }
  
  .setting-group {
    margin-bottom: 20px;
  }
  
  .setting-group:last-child {
    margin-bottom: 0;
  }
  
  .setting-label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
    margin-bottom: 10px;
  }
  
  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  
  .radio-item {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }
  
  .radio-item input[type="radio"] {
    accent-color: var(--color-accent);
  }
  
  .radio-label {
    font-size: 13px;
    color: var(--color-text-primary);
  }
  
  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px 20px;
    border-top: 1px solid var(--color-border);
  }
  
  .btn {
    padding: 6px 16px;
    font-size: 13px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  
  .btn-primary {
    background-color: var(--color-accent);
    color: white;
    border: 1px solid var(--color-accent);
  }
  
  .btn-primary:hover {
    background-color: #005a9e;
  }
  
  .btn-secondary {
    background-color: var(--color-bg-tertiary);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
  }
  
  .btn-secondary:hover {
    background-color: var(--color-accent);
    border-color: var(--color-accent);
  }
</style>
```

---

### 3. settingsStore.ts - 设置状态管理（新增）

**文件**: [src/lib/stores/settingsStore.ts](src/lib/stores/settingsStore.ts)（新建）

**完整代码**:
```typescript
import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  theme: 'dark' | 'light';
  encoding: 'auto' | 'utf-8' | 'ansi' | 'utf-16le' | 'utf-16be';
}

class SettingsStore {
  private settings: AppSettings = {
    theme: 'dark',
    encoding: 'auto'
  };
  private subscribers: Set<() => void> = new Set();

  subscribe(callback: () => void): () => void {
    this.subscribers.add(callback);
    return () => this.subscribers.delete(callback);
  }

  private notify() {
    this.subscribers.forEach(callback => callback());
  }

  getSettings(): AppSettings {
    return { ...this.settings };
  }

  async loadSettings(): Promise<void> {
    try {
      const settings = await invoke<AppSettings>('get_settings');
      this.settings = settings;
      this.notify();
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  async updateSettings(settings: Partial<AppSettings>): Promise<void> {
    this.settings = { ...this.settings, ...settings };
    this.notify();
    
    try {
      await invoke('save_settings', { settings: this.settings });
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  }

  getTheme(): 'dark' | 'light' {
    return this.settings.theme;
  }

  getEncoding(): string {
    return this.settings.encoding;
  }
}

export const settingsStore = new SettingsStore();
```

---

### 4. HelpModal.svelte - 帮助弹窗（新增）

**文件**: [src/lib/components/HelpModal.svelte](src/lib/components/HelpModal.svelte)（新建）

**完整代码**:
```svelte
<script lang="ts">
  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }
  
  let { isOpen, onClose }: Props = $props();
  
  const shortcuts = [
    { key: 'F5', description: '刷新日志' },
    { key: 'Ctrl+O', description: '打开文件' },
    { key: 'Ctrl+F', description: '聚焦搜索框' },
    { key: 'Escape', description: '重置筛选条件' },
    { key: 'Ctrl+C', description: '复制选中日志内容' }
  ];
</script>

{#if isOpen}
  <div class="modal-overlay" onclick={onClose}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>帮助</h2>
        <button class="close-btn" onclick={onClose}>×</button>
      </div>
      
      <div class="modal-body">
        <h3>快捷键</h3>
        <table class="shortcuts-table">
          <tbody>
            {#each shortcuts as shortcut}
              <tr>
                <td class="key">{shortcut.key}</td>
                <td class="description">{shortcut.description}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        
        <h3>使用说明</h3>
        <ul class="help-list">
          <li>拖拽日志文件到窗口即可加载</li>
          <li>使用级别标签快速筛选日志级别</li>
          <li>搜索支持模糊、精确、正则三种模式</li>
          <li>可在设置中切换主题和文件编码</li>
        </ul>
      </div>
      
      <div class="modal-footer">
        <button class="btn btn-primary" onclick={onClose}>确定</button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* 样式与 SettingsModal 类似 */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
  }
  
  .modal-content {
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    min-width: 400px;
    max-width: 500px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  }
  
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border);
  }
  
  .modal-header h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 20px;
    color: var(--color-text-secondary);
    cursor: pointer;
  }
  
  .modal-body {
    padding: 20px;
  }
  
  .modal-body h3 {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    margin: 0 0 12px 0;
  }
  
  .modal-body h3:not(:first-child) {
    margin-top: 20px;
  }
  
  .shortcuts-table {
    width: 100%;
    border-collapse: collapse;
  }
  
  .shortcuts-table td {
    padding: 8px 0;
    font-size: 13px;
  }
  
  .shortcuts-table .key {
    color: var(--color-accent);
    font-family: monospace;
    width: 100px;
  }
  
  .shortcuts-table .description {
    color: var(--color-text-primary);
  }
  
  .help-list {
    margin: 0;
    padding-left: 20px;
  }
  
  .help-list li {
    font-size: 13px;
    color: var(--color-text-primary);
    margin-bottom: 8px;
  }
  
  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: 16px 20px;
    border-top: 1px solid var(--color-border);
  }
  
  .btn {
    padding: 6px 16px;
    font-size: 13px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .btn-primary {
    background-color: var(--color-accent);
    color: white;
    border: 1px solid var(--color-accent);
  }
</style>
```

---

### 5. StatusBar.svelte - 状态栏动态显示

**文件**: [src/lib/components/StatusBar.svelte](src/lib/components/StatusBar.svelte)

**修改内容**:
- 新增状态 Props
- 根据不同状态显示不同内容

**修改后代码**:
```svelte
<script lang="ts">
  import { logStore } from '$lib/stores/logStore';
  import { clearCache, getCacheInfo, type CacheInfo } from '$lib/api';

  interface Props {
    logCount?: number;
    onRefresh?: () => void;
    statusMessage?: string;
    loadingProgress?: number;
    loadingPhase?: string;
    loadTime?: number;
    totalEntries?: number;
  }

  let { 
    logCount = 0, 
    onRefresh,
    statusMessage = '',
    loadingProgress = 0,
    loadingPhase = '',
    loadTime = 0,
    totalEntries = 0
  }: Props = $props();
  
  let cacheInfo = $state<CacheInfo | null>(null);
  let clearing = $state(false);

  async function handleClearCache() {
    if (clearing) return;
    clearing = true;
    try {
      const info = await clearCache();
      cacheInfo = info;
      logStore.clear();
      setTimeout(() => {
        cacheInfo = null;
      }, 3000);
    } catch (e) {
      console.error('Failed to clear cache:', e);
    } finally {
      clearing = false;
    }
  }
  
  function getStatusText(): string {
    if (statusMessage) return statusMessage;
    if (loadingProgress > 0 && loadingProgress < 100) {
      return `加载中: ${loadingPhase} (${loadingProgress.toFixed(0)}%)`;
    }
    if (totalEntries > 0 && loadTime > 0) {
      return `已加载 ${totalEntries.toLocaleString()} 条日志，用时 ${loadTime}ms`;
    }
    return '打开文件或将日志文件拖入窗口即可加载，按F5可刷新';
  }
</script>

<footer class="status-bar">
  <div class="status-left">
    {#if cacheInfo}
      <span class="cache-info">已清理 {cacheInfo.entries_cleared} 条缓存</span>
    {:else}
      <span class="status-text" class:loading={loadingProgress > 0 && loadingProgress < 100}>
        {getStatusText()}
      </span>
    {/if}
  </div>
  <div class="status-right">
    <button class="action-btn" onclick={handleClearCache} disabled={clearing} title="清理缓存">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
        <line x1="10" y1="11" x2="10" y2="17"/>
        <line x1="14" y1="11" x2="14" y2="17"/>
      </svg>
      {clearing ? '清理中...' : '清理缓存'}
    </button>
    <button class="action-btn" onclick={onRefresh} title="刷新 (F5)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M23 4v6h-6M1 20v-6h6"/>
        <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
      </svg>
      刷新
    </button>
    <span class="count">列表数: {logCount.toLocaleString()}</span>
  </div>
</footer>

<style>
  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: var(--status-height);
    padding: 0 12px;
    background-color: var(--color-bg-secondary);
    border-top: 1px solid var(--color-border);
    font-size: 12px;
  }

  .status-left {
    display: flex;
    align-items: center;
  }

  .status-text {
    color: var(--color-text-secondary);
  }
  
  .status-text.loading {
    color: var(--color-accent);
  }

  .status-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    font-size: 12px;
    color: var(--color-text-primary);
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover:not(:disabled) {
    background-color: var(--color-accent);
    border-color: var(--color-accent);
  }

  .action-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .count {
    color: var(--color-text-secondary);
    margin-left: 8px;
  }

  .cache-info {
    color: #4ec9b0;
    font-size: 12px;
  }
</style>
```

---

### 6. CategoryTabs.svelte - 级别筛选增加标签

**文件**: [src/lib/components/CategoryTabs.svelte](src/lib/components/CategoryTabs.svelte)

**修改内容**:
- 在所有筛选标签前增加"级别"标签

**修改后代码**:
```svelte
<script lang="ts">
  import { logStore, type LogStats } from '$lib/stores/logStore';

  interface Props {
    activeCategory?: string;
    onCategoryChange?: (category: string) => void;
  }

  let { activeCategory = 'all', onCategoryChange }: Props = $props();

  const categories = [
    { id: 'all', label: '所有', color: null },
    { id: 'fatal', label: '致命', color: 'var(--color-fatal)' },
    { id: 'error', label: '错误', color: 'var(--color-error)' },
    { id: 'warn', label: '警告', color: 'var(--color-warn)' },
    { id: 'info', label: '信息', color: 'var(--color-info)' },
    { id: 'debug', label: '调试', color: 'var(--color-debug)' },
    { id: 'trace', label: '跟踪', color: 'var(--color-trace)' },
    { id: 'other', label: '其它', color: 'var(--color-other)' }
  ];

  let stats = $state<LogStats>({
    all: 0,
    fatal: 0,
    error: 0,
    warn: 0,
    info: 0,
    debug: 0,
    trace: 0,
    other: 0
  });

  $effect(() => {
    const unsubscribe = logStore.subscribe(() => {
      stats = logStore.getStats();
    });
    return unsubscribe;
  });

  function handleClick(categoryId: string) {
    onCategoryChange?.(categoryId);
  }
</script>

<div class="category-tabs">
  <span class="category-label">级别</span>
  {#each categories as category}
    <button 
      class="tab-button"
      class:active={activeCategory === category.id}
      style:--category-color={category.color}
      onclick={() => handleClick(category.id)}
    >
      <span class="label">{category.label}</span>
      <span class="count">({stats[category.id as keyof LogStats] || 0})</span>
    </button>
  {/each}
</div>

<style>
  .category-tabs {
    display: flex;
    align-items: center;
    height: var(--tabs-height);
    padding: 0 8px;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    gap: 2px;
    overflow-x: auto;
  }

  .category-label {
    font-size: 13px;
    color: var(--color-text-secondary);
    padding: 6px 8px;
    white-space: nowrap;
  }

  .tab-button {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    font-size: 13px;
    color: var(--color-text-primary);
    background-color: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.15s ease;
    white-space: nowrap;
  }

  .tab-button:hover {
    background-color: var(--color-bg-tertiary);
  }

  .tab-button.active {
    background-color: var(--color-bg-tertiary);
    color: var(--category-color, var(--color-accent));
  }

  .tab-button .label {
    font-weight: 500;
  }

  .tab-button .count {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .tab-button.active .count {
    color: var(--category-color, var(--color-text-secondary));
  }
</style>
```

---

### 7. ControlBar.svelte - 搜索控制栏重构

**文件**: [src/lib/components/ControlBar.svelte](src/lib/components/ControlBar.svelte)

**修改内容**:
- 时间筛选标签改为"筛选"
- 增加搜索范围下拉框
- 增加搜索模式下拉框
- 增加搜索按钮
- 修改搜索触发逻辑

**修改后代码**:
```svelte
<script lang="ts">
  interface Props {
    onSearch?: (query: string, scope: string, mode: string) => void;
    onTimeFilter?: (filter: string) => void;
  }

  let { onSearch, onTimeFilter }: Props = $props();

  let searchValue = $state('');
  let selectedTimeFilter = $state('all');
  let searchScope = $state('all');
  let searchMode = $state('fuzzy');

  const timeOptions = [
    { value: 'all', label: '全部时间' },
    { value: 'today', label: '今天' },
    { value: 'yesterday', label: '昨天' },
    { value: 'week', label: '最近7天' },
    { value: 'month', label: '最近30天' },
    { value: 'custom', label: '自定义...' }
  ];

  const scopeOptions = [
    { value: 'all', label: '全部范围' },
    { value: 'logger', label: '记录者' },
    { value: 'content', label: '内容' }
  ];

  const modeOptions = [
    { value: 'fuzzy', label: '模糊' },
    { value: 'exact', label: '精确' },
    { value: 'regex', label: '正则' }
  ];

  function handleSearch() {
    onSearch?.(searchValue, searchScope, searchMode);
  }

  function handleTimeChange() {
    onTimeFilter?.(selectedTimeFilter);
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleSearch();
    }
  }
</script>

<div class="control-bar">
  <div class="filter-section">
    <div class="filter-group">
      <label for="time-select">筛选</label>
      <select id="time-select" bind:value={selectedTimeFilter} onchange={handleTimeChange}>
        {#each timeOptions as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
    
    <div class="filter-group">
      <select id="scope-select" bind:value={searchScope}>
        {#each scopeOptions as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
    
    <div class="filter-group">
      <select id="mode-select" bind:value={searchMode}>
        {#each modeOptions as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
  </div>
  
  <div class="search-section">
    <div class="search-box">
      <input 
        type="text" 
        placeholder="输入搜索条件..." 
        bind:value={searchValue}
        onkeydown={handleSearchKeydown}
      />
    </div>
    <button class="search-btn" onclick={handleSearch}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      搜索
    </button>
  </div>
</div>

<style>
  .control-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    height: var(--control-height);
    padding: 0 12px;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .filter-section {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .filter-group label {
    font-size: 13px;
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .filter-group select {
    padding: 4px 8px;
    font-size: 13px;
    color: var(--color-text-primary);
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    cursor: pointer;
    min-width: 90px;
  }

  .filter-group select:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .search-section {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    max-width: 500px;
  }

  .search-box {
    flex: 1;
  }

  .search-box input {
    width: 100%;
    padding: 6px 12px;
    font-size: 13px;
    color: var(--color-text-primary);
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
  }

  .search-box input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .search-box input::placeholder {
    color: var(--color-text-secondary);
  }

  .search-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    font-size: 13px;
    color: var(--color-text-primary);
    background-color: var(--color-bg-tertiary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
  }

  .search-btn:hover {
    background-color: var(--color-accent);
    border-color: var(--color-accent);
  }
</style>
```

---

### 8. logStore.ts - 搜索逻辑扩展

**文件**: [src/lib/stores/logStore.ts](src/lib/stores/logStore.ts)

**修改内容**:
- 增加搜索范围和搜索模式参数
- 实现三种搜索模式的逻辑

**关键修改**:
```typescript
class LogStore {
  // ... 现有属性 ...
  private searchScope = 'all';
  private searchMode = 'fuzzy';

  setSearchQuery(query: string, scope: string = 'all', mode: string = 'fuzzy'): void {
    this.searchQuery = query;
    this.searchScope = scope;
    this.searchMode = mode;
    this.applyFilters();
  }

  private applyFilters(): void {
    let result = [...this.logs];

    // 级别筛选（保持不变）
    if (this.activeCategory !== 'all') {
      // ... 现有逻辑 ...
    }

    // 搜索逻辑（重构）
    if (this.searchQuery) {
      result = result.filter(log => {
        let searchFields: string[] = [];
        
        switch (this.searchScope) {
          case 'logger':
            searchFields = [log.logger];
            break;
          case 'content':
            searchFields = [log.summary, log.raw];
            break;
          case 'all':
          default:
            searchFields = [log.raw, log.summary, log.logger];
        }

        switch (this.searchMode) {
          case 'exact':
            return searchFields.some(field => field.includes(this.searchQuery));
          
          case 'regex':
            try {
              const regex = new RegExp(this.searchQuery);
              return searchFields.some(field => regex.test(field));
            } catch {
              return false;
            }
          
          case 'fuzzy':
          default:
            const query = this.searchQuery.toLowerCase();
            return searchFields.some(field => 
              field.toLowerCase().includes(query)
            );
        }
      });
    }

    // 时间筛选（保持不变）
    // ... 现有逻辑 ...

    this.filteredLogs = result;
    this.notify();
  }
}
```

---

### 9. +page.svelte - 主页面整合

**文件**: [src/routes/+page.svelte](src/routes/+page.svelte)

**修改内容**:
- 导入新组件
- 增加状态管理
- 处理加载进度和状态显示

**关键修改**:
```svelte
<script lang="ts">
  import TitleBar from '$lib/components/TitleBar.svelte';
  import FileBar from '$lib/components/FileBar.svelte';
  import ControlBar from '$lib/components/ControlBar.svelte';
  import CategoryTabs from '$lib/components/CategoryTabs.svelte';
  import LogList from '$lib/components/LogList.svelte';
  import LogDetail from '$lib/components/LogDetail.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import HelpModal from '$lib/components/HelpModal.svelte';
  import { logStore } from '$lib/stores/logStore';
  import { settingsStore } from '$lib/stores/settingsStore';
  import { onParseProgress } from '$lib/api';

  // 现有状态
  let selectedLogId = $state<number | null>(null);
  let activeCategory = $state<string>('all');
  let logCount = $state<number>(0);
  let isDragging = $state(false);

  // 新增状态
  let showSettings = $state(false);
  let showHelp = $state(false);
  let statusMessage = $state('');
  let loadingProgress = $state(0);
  let loadingPhase = $state('');
  let loadStartTime = $state(0);
  let loadTime = $state(0);
  let totalEntries = $state(0);

  // 加载进度监听
  $effect(() => {
    const unsubscribe = onParseProgress((progress) => {
      loadingProgress = progress.percentage;
      loadingPhase = `已解析 ${progress.entries_parsed.toLocaleString()} 条`;
    });
    return unsubscribe;
  });

  // 搜索处理（修改）
  function handleSearch(query: string, scope: string, mode: string) {
    logStore.setSearchQuery(query, scope, mode);
  }

  // 打开文件处理（修改）
  async function handleOpenFile() {
    statusMessage = '正在打开文件...';
    loadStartTime = Date.now();
    try {
      await logStore.openFileDialog();
      loadTime = Date.now() - loadStartTime;
      totalEntries = logStore.getStats().all;
      statusMessage = '';
    } catch (err) {
      console.error('Failed to open file:', err);
      statusMessage = '打开文件失败';
    }
  }

  // 刷新处理（修改）
  function handleRefresh() {
    statusMessage = '正在刷新...';
    selectedLogId = null;
    logStore.clear();
    totalEntries = 0;
    loadTime = 0;
    setTimeout(() => {
      statusMessage = '';
    }, 500);
  }

  // ... 其他处理函数 ...
</script>

<!-- 模板修改 -->
<div class="app-container" ...>
  <TitleBar 
    onOpenSettings={() => showSettings = true}
    onOpenHelp={() => showHelp = true}
  />
  <FileBar />
  <ControlBar 
    onSearch={handleSearch} 
    onTimeFilter={handleTimeFilter} 
  />
  <CategoryTabs 
    activeCategory={activeCategory} 
    onCategoryChange={handleCategoryChange} 
  />
  <div class="main-content">
    <!-- ... -->
  </div>
  <StatusBar 
    logCount={logCount} 
    onRefresh={handleRefresh}
    statusMessage={statusMessage}
    loadingProgress={loadingProgress}
    loadingPhase={loadingPhase}
    loadTime={loadTime}
    totalEntries={totalEntries}
  />
  
  <SettingsModal 
    isOpen={showSettings} 
    onClose={() => showSettings = false} 
  />
  <HelpModal 
    isOpen={showHelp} 
    onClose={() => showHelp = false} 
  />
</div>
```

---

### 10. 后端修改 - 设置持久化和编码支持

#### 10.1 models/mod.rs - 新增设置模型

**文件**: [src-tauri/src/models/mod.rs](src-tauri/src/models/mod.rs)

**新增内容**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub encoding: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            encoding: "auto".to_string(),
        }
    }
}
```

#### 10.2 commands/mod.rs - 新增设置命令

**文件**: [src-tauri/src/commands/mod.rs](src-tauri/src/commands/mod.rs)

**新增内容**:
```rust
use std::fs;
use std::path::PathBuf;
use crate::models::AppSettings;

fn get_settings_path() -> PathBuf {
    let data_dir = Database::get_data_dir();
    data_dir.join("settings.json")
}

#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    let path = get_settings_path();
    
    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        let settings: AppSettings = serde_json::from_str(&content)
            .unwrap_or_default();
        Ok(settings)
    } else {
        Ok(AppSettings::default())
    }
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path();
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;
    
    Ok(())
}
```

#### 10.3 reader/mod.rs - 编码支持

**文件**: [src-tauri/src/reader/mod.rs](src-tauri/src/reader/mod.rs)

**新增编码检测和转换函数**:
```rust
use encoding_rs::{UTF_8, UTF_16LE, UTF_16BE};

pub enum FileEncoding {
    Auto,
    Utf8,
    Ansi,
    Utf16LE,
    Utf16BE,
}

impl FileEncoding {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "utf-8" => FileEncoding::Utf8,
            "ansi" => FileEncoding::Ansi,
            "utf-16le" => FileEncoding::Utf16LE,
            "utf-16be" => FileEncoding::Utf16BE,
            _ => FileEncoding::Auto,
        }
    }
}

impl LogFileReader {
    pub fn read_lines_with_encoding(&self, encoding: FileEncoding) -> Result<MmapLineIterator, String> {
        // 实现带编码的读取逻辑
        // ...
    }
    
    fn detect_encoding(data: &[u8]) -> FileEncoding {
        // BOM 检测
        if data.len() >= 3 && &data[0..3] == b"\xEF\xBB\xBF" {
            return FileEncoding::Utf8;
        }
        if data.len() >= 2 {
            if &data[0..2] == b"\xFF\xFE" {
                return FileEncoding::Utf16LE;
            }
            if &data[0..2] == b"\xFE\xFF" {
                return FileEncoding::Utf16BE;
            }
        }
        FileEncoding::Utf8
    }
}
```

#### 10.4 main.rs - 注册新命令

**文件**: [src-tauri/src/main.rs](src-tauri/src/main.rs)

**修改内容**:
```rust
fn main() {
    tauri::Builder::default()
        .manage(AppState::new(db))
        .invoke_handler(tauri::generate_handler![
            // 现有命令
            commands::open_file,
            commands::parse_log,
            commands::get_entries,
            commands::get_entry_detail,
            commands::get_stats,
            commands::filter_by_level,
            commands::search,
            commands::get_current_file,
            commands::clear_cache,
            commands::get_cache_info,
            // 新增命令
            commands::get_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### 11. README.md 更新

**文件**: [README.md](README.md)

**新增内容**:

```markdown
## 功能特性

- **大文件支持**：使用内存映射文件（mmap）零拷贝技术，轻松处理GB级日志文件
- **快速解析**：多线程并行解析，1GB文件解析时间 < 15秒
- **智能分类**：自动识别日志级别（FATAL/ERROR/WARN/INFO/DEBUG/TRACE），支持快速过滤
- **高效搜索**：
  - 支持模糊搜索、精确匹配、正则表达式三种模式
  - 支持全部范围、记录者、内容三种搜索范围
- **虚拟滚动**：支持千万级日志条目的流畅滚动和快速跳转
- **多格式查看**：支持文本、HTML、JSON、XML 四种查看模式
- **时间筛选**：按时间范围快速筛选日志
- **主题切换**：支持深色/浅色主题
- **编码支持**：支持自动检测、UTF-8、ANSI、Unicode LE/BE 编码
- **拖拽加载**：支持拖拽文件到窗口直接打开

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `F5` | 刷新日志 |
| `Ctrl+O` | 打开文件 |
| `Ctrl+F` | 聚焦搜索框 |
| `Escape` | 重置筛选条件 |
| `Ctrl+C` | 复制选中日志内容 |

## 搜索功能

### 搜索模式
- **模糊搜索**：不区分大小写的包含匹配
- **精确匹配**：区分大小写的精确匹配
- **正则表达式**：支持标准正则表达式语法

### 搜索范围
- **全部范围**：搜索日志原始内容、摘要和记录者
- **记录者**：仅搜索记录者字段
- **内容**：仅搜索日志摘要和原始内容
```

---

## 文件修改清单

| 序号 | 文件 | 修改类型 | 说明 |
|------|------|----------|------|
| 1 | TitleBar.svelte | 重构 | 移除提示，增加设置/帮助按钮 |
| 2 | SettingsModal.svelte | 新增 | 设置弹窗组件 |
| 3 | HelpModal.svelte | 新增 | 帮助弹窗组件 |
| 4 | settingsStore.ts | 新增 | 设置状态管理 |
| 5 | StatusBar.svelte | 重构 | 动态状态显示 |
| 6 | CategoryTabs.svelte | 修改 | 添加"级别"标签 |
| 7 | ControlBar.svelte | 重构 | 搜索范围/模式/按钮 |
| 8 | logStore.ts | 修改 | 扩展搜索功能 |
| 9 | +page.svelte | 修改 | 整合新组件和状态 |
| 10 | models/mod.rs | 修改 | 新增 AppSettings 模型 |
| 11 | commands/mod.rs | 修改 | 新增设置命令 |
| 12 | reader/mod.rs | 修改 | 编码支持 |
| 13 | main.rs | 修改 | 注册新命令 |
| 14 | README.md | 更新 | 更新功能说明 |

---

## 与现有实现差距的关联分析

根据 README.md 中的"当前实现差距分析"，本次 UI 改进方案涉及以下需要并行修改的部分：

### 本次方案涉及的差距项

| 差距项 | 严重程度 | 本次方案关联 | 处理方式 |
|--------|----------|--------------|----------|
| UTF-8 处理不当 | 中等 | 设置界面增加编码选择 | **本次一并解决** |
| 进度更新粗糙 | 轻微 | 状态栏显示加载进度 | **本次一并优化** |
| 时间过滤在前端 | 中等 | 搜索功能扩展 | 本次仅前端实现，后端优化列入 TODO |

### 本次需要并行修改的内容

#### 1. 编码处理改进（关联设置界面的编码选择）

**文件**: [src-tauri/src/reader/mod.rs](src-tauri/src/reader/mod.rs)

**新增内容**:
```rust
use encoding_rs::{UTF_8, UTF_16LE, UTF_16BE, WINDOWS_1252};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileEncoding {
    Auto,
    Utf8,
    Ansi,      // Windows-1252
    Utf16LE,
    Utf16BE,
}

impl FileEncoding {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "utf-8" => FileEncoding::Utf8,
            "ansi" => FileEncoding::Ansi,
            "utf-16le" => FileEncoding::Utf16LE,
            "utf-16be" => FileEncoding::Utf16BE,
            _ => FileEncoding::Auto,
        }
    }
}

impl LogFileReader {
    pub fn detect_encoding(data: &[u8]) -> FileEncoding {
        if data.len() >= 3 && &data[0..3] == b"\xEF\xBB\xBF" {
            return FileEncoding::Utf8;
        }
        if data.len() >= 2 {
            if &data[0..2] == b"\xFF\xFE" {
                return FileEncoding::Utf16LE;
            }
            if &data[0..2] == b"\xFE\xFF" {
                return FileEncoding::Utf16BE;
            }
        }
        FileEncoding::Utf8
    }
    
    pub fn decode_line(data: &[u8], encoding: FileEncoding) -> String {
        match encoding {
            FileEncoding::Utf8 | FileEncoding::Auto => {
                String::from_utf8_lossy(data).into_owned()
            }
            FileEncoding::Utf16LE => {
                UTF_16LE.decode(data).0.into_owned()
            }
            FileEncoding::Utf16BE => {
                UTF_16BE.decode(data).0.into_owned()
            }
            FileEncoding::Ansi => {
                WINDOWS_1252.decode(data).0.into_owned()
            }
        }
    }
}
```

#### 2. 进度反馈优化（关联状态栏显示）

**文件**: [src-tauri/src/commands/mod.rs](src-tauri/src/commands/mod.rs)

**修改内容**:
```rust
let mut last_update = std::time::Instant::now();
const UPDATE_INTERVAL_MS: u64 = 100;
const UPDATE_ENTRY_INTERVAL: u64 = 5000;

for line in lines {
    // ... 解析逻辑 ...
    
    let now = std::time::Instant::now();
    let elapsed = now.duration_since(last_update).as_millis() as u64;
    
    if elapsed >= UPDATE_INTERVAL_MS || entry_count % UPDATE_ENTRY_INTERVAL == 0 {
        last_update = now;
        
        let phase = format!(
            "解析中: 已处理 {} 条",
            entry_count.toLocaleString()
        );
        
        let _ = app.emit("parse_progress", ParseProgress {
            file_id,
            total_bytes: file_size,
            processed_bytes: line.offset,
            entries_parsed: entry_count,
            percentage: progress,
            phase,
            is_complete: false,
        });
    }
}
```

**文件**: [src-tauri/src/models/mod.rs](src-tauri/src/models/mod.rs)

**修改 ParseProgress 结构体**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseProgress {
    pub file_id: u64,
    pub total_bytes: u64,
    pub processed_bytes: u64,
    pub entries_parsed: u64,
    pub percentage: f32,
    pub phase: String,  // 新增：阶段描述
    pub is_complete: bool,
}
```

---

## 实施步骤

### 第一阶段：后端修改

1. **models/mod.rs**
   - 新增 AppSettings 模型
   - 修改 ParseProgress 增加 phase 字段

2. **commands/mod.rs**
   - 新增 get_settings、save_settings 命令
   - 优化进度反馈（时间+数量双重策略）
   - 修改 parse_log 支持编码参数

3. **reader/mod.rs**
   - 新增 FileEncoding 枚举
   - 新增 detect_encoding 函数
   - 新增 decode_line 函数

4. **main.rs**
   - 注册新命令

### 第二阶段：前端状态管理

1. **settingsStore.ts** - 新建
   - 设置状态管理
   - 编码设置持久化

### 第三阶段：前端组件修改

1. **TitleBar.svelte** - 改造标题栏
2. **CategoryTabs.svelte** - 添加"级别"标签
3. **ControlBar.svelte** - 重构搜索控制栏
4. **StatusBar.svelte** - 动态状态显示

### 第四阶段：新增组件

1. **SettingsModal.svelte** - 设置弹窗
2. **HelpModal.svelte** - 帮助弹窗

### 第五阶段：主页面整合

1. **+page.svelte** - 整合所有修改

### 第六阶段：文档更新

1. **README.md** - 更新功能说明

### 第七阶段：测试验证

1. 编译检查
2. 功能测试
