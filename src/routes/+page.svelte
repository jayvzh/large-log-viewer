<script lang="ts">
  import TitleBar from '$lib/components/TitleBar.svelte';
  import FileBar from '$lib/components/FileBar.svelte';
  import ControlBar from '$lib/components/ControlBar.svelte';
  import CategoryTabs from '$lib/components/CategoryTabs.svelte';
  import FilterPanel from '$lib/components/FilterPanel.svelte';
  import LogList from '$lib/components/LogList.svelte';
  import LogDetail from '$lib/components/LogDetail.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import HelpModal from '$lib/components/HelpModal.svelte';
  import { logStore } from '$lib/stores/logStore';
  import { templateStore, type FilterCondition } from '$lib/stores/templateStore';
  import { settingsStore } from '$lib/stores/settingsStore';
  import { onParseProgress, isTauriSync } from '$lib/api';
  import { listen } from '@tauri-apps/api/event';

  let selectedLogId = $state<number | null>(null);
  let activeCategory = $state<string>('all');
  let logCount = $state<number>(0);
  let isDragging = $state(false);
  let showSettings = $state(false);
  let showHelp = $state(false);
  let statusMessage = $state('');
  let loadingProgress = $state(0);
  let loadingPhase = $state('');
  let loadTime = $state(0);
  let totalEntries = $state(0);
  let fileSize = $state(0);
  let fileName = $state('');
  let detailHeight = $state(180);
  let isResizing = $state(false);
  let extraFields = $state<string[]>([]);
  let filterConditions = $state<FilterCondition[]>([]);
  let filterCombineMode = $state<'and' | 'or'>('and');

  $effect(() => {
    const unsubscribe = logStore.subscribe(() => {
      logCount = logStore.getFilteredLogs().length;
      const currentFile = logStore.getCurrentFile();
      if (currentFile && !logStore.isLoading()) {
        totalEntries = logStore.getStats().all;
        fileSize = currentFile.size;
        fileName = currentFile.name;
        loadTime = logStore.getLoadTime();
      }
    });
    return unsubscribe;
  });

  $effect(() => {
    const unsubscribe = templateStore.subscribe(() => {
      extraFields = templateStore.getCurrentFields();
      filterConditions = templateStore.getFilterConditions();
      filterCombineMode = templateStore.getFilterCombineMode();
    });
    return unsubscribe;
  });

  $effect(() => {
    const unsubscribe = onParseProgress((progress) => {
      loadingProgress = progress.percentage;
      loadingPhase = progress.phase;
    });
    return unsubscribe;
  });

  $effect(() => {
    settingsStore.loadSettings();
  });

  $effect(() => {
    if (!isTauriSync()) return;
    
    let unlisten: (() => void) | null = null;
    
    listen<string>('open-file-argument', async (event) => {
      const filePath = event.payload;
      if (filePath) {
        statusMessage = '正在加载文件...';
        try {
          await logStore.loadFile(filePath);
          statusMessage = '';
        } catch (err) {
          console.error('Failed to load file from argument:', err);
          statusMessage = '加载文件失败';
        }
      }
    }).then((fn) => {
      unlisten = fn;
    });
    
    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  });

  function handleLogSelect(id: number) {
    selectedLogId = id;
  }

  async function handleCategoryChange(category: string) {
    activeCategory = category;
    await logStore.setActiveCategory(category);
  }

  async function handleSearch(query: string, scope: string, mode: string) {
    await logStore.setSearchQuery(query, scope, mode);
  }

  async function handleTimeFilter(filter: string) {
    await logStore.setTimeFilter(filter);
  }

  function handleAddFilterCondition(condition: FilterCondition) {
    templateStore.addFilterCondition(condition);
    logStore.refreshFilters();
  }

  function handleRemoveFilterCondition(id: string) {
    templateStore.removeFilterCondition(id);
    logStore.refreshFilters();
  }

  function handleUpdateFilterCondition(condition: FilterCondition) {
    templateStore.updateFilterCondition(condition);
    logStore.refreshFilters();
  }

  function handleFilterCombineModeChange(mode: 'and' | 'or') {
    templateStore.setFilterCombineMode(mode);
    logStore.refreshFilters();
  }

  function handleClearAllFilterConditions() {
    templateStore.clearFilterConditions();
    logStore.refreshFilters();
  }

  async function handleOpenFile() {
    statusMessage = '正在打开文件...';
    try {
      await logStore.openFileDialog();
      statusMessage = '';
    } catch (err) {
      console.error('Failed to open file:', err);
      statusMessage = '打开文件失败';
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
    
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0];
      const isValidFile = file.name.endsWith('.log') || 
                          file.name.endsWith('.txt') || 
                          file.name.endsWith('.json') ||
                          file.name.endsWith('.1') ||
                          !file.name.includes('.');
      
      if (isValidFile) {
        statusMessage = '正在加载文件...';
        try {
          if (isTauriSync()) {
            const path = (file as any).path;
            if (path) {
              await logStore.loadFile(path);
              statusMessage = '';
            } else {
              statusMessage = '无法获取文件路径';
            }
          } else {
            statusMessage = '拖拽功能需要在 Tauri 桌面应用中使用。\n请运行 pnpm tauri dev 启动桌面应用。';
            console.warn('Drag and drop file loading is only supported in Tauri environment');
          }
        } catch (err) {
          console.error('Failed to load file:', err);
          statusMessage = '加载文件失败';
        }
      } else {
        statusMessage = '不支持的文件类型，请拖拽 .log, .txt, .json 文件';
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'o') {
      e.preventDefault();
      handleOpenFile();
    } else if (e.ctrlKey && e.key === 'f') {
      e.preventDefault();
      const searchInput = document.querySelector('.search-box input') as HTMLInputElement;
      if (searchInput) {
        searchInput.focus();
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      activeCategory = 'all';
      logStore.setActiveCategory('all');
      logStore.setSearchQuery('', 'all', 'fuzzy');
      logStore.setTimeFilter('all');
    }
  }

  function handleOpenSettings() {
    showSettings = true;
  }

  function handleCloseSettings() {
    showSettings = false;
  }

  function handleOpenHelp() {
    showHelp = true;
  }

  function handleCloseHelp() {
    showHelp = false;
  }

  function handleResizeStart(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;
    document.addEventListener('mousemove', handleResizeMove);
    document.addEventListener('mouseup', handleResizeEnd);
  }

  function handleResizeMove(e: MouseEvent) {
    if (!isResizing) return;
    const mainContent = document.querySelector('.main-content');
    if (!mainContent) return;
    const rect = mainContent.getBoundingClientRect();
    const newHeight = rect.bottom - e.clientY;
    detailHeight = Math.min(400, Math.max(100, newHeight));
  }

  function handleResizeEnd() {
    isResizing = false;
    document.removeEventListener('mousemove', handleResizeMove);
    document.removeEventListener('mouseup', handleResizeEnd);
  }
</script>

<svelte:head>
  <title>LogViewer</title>
</svelte:head>

<svelte:window onkeydown={handleKeydown} />

<div 
  class="app-container"
  class:dragging={isDragging}
  role="application"
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
>
  <TitleBar 
    onOpenSettings={handleOpenSettings}
    onOpenHelp={handleOpenHelp}
  />
  <FileBar />
  <ControlBar 
    onSearch={handleSearch} 
    onTimeFilter={handleTimeFilter} 
  />
  <FilterPanel 
    fields={extraFields}
    conditions={filterConditions}
    combineMode={filterCombineMode}
    onAdd={handleAddFilterCondition}
    onRemove={handleRemoveFilterCondition}
    onUpdate={handleUpdateFilterCondition}
    onCombineModeChange={handleFilterCombineModeChange}
    onClearAll={handleClearAllFilterConditions}
  />
  <CategoryTabs 
    activeCategory={activeCategory} 
    onCategoryChange={handleCategoryChange} 
  />
  <div class="main-content">
    <div class="log-list-panel">
      <LogList 
        selectedId={selectedLogId}
        onSelect={handleLogSelect}
      />
    </div>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div 
      class="resize-handle"
      class:active={isResizing}
      onmousedown={handleResizeStart}
      role="separator"
      aria-orientation="horizontal"
    ></div>
    <div class="log-detail-panel" style="height: {detailHeight}px;">
      <LogDetail logId={selectedLogId} />
    </div>
  </div>
  <StatusBar 
    logCount={logCount} 
    statusMessage={statusMessage}
    loadingProgress={loadingProgress}
    loadingPhase={loadingPhase}
    loadTime={loadTime}
    totalEntries={totalEntries}
    fileSize={fileSize}
    fileName={fileName}
  />
  
  {#if isDragging}
    <div class="drop-overlay">
      <div class="drop-hint">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
          <polyline points="17 8 12 3 7 8"></polyline>
          <line x1="12" y1="3" x2="12" y2="15"></line>
        </svg>
        <span>拖放日志文件到此处</span>
      </div>
    </div>
  {/if}
</div>

<SettingsModal 
  isOpen={showSettings} 
  onClose={handleCloseSettings} 
/>

<HelpModal 
  isOpen={showHelp} 
  onClose={handleCloseHelp} 
/>

<style>
  :root {
    --color-bg-primary: #1e1e1e;
    --color-bg-secondary: #2d2d30;
    --color-bg-tertiary: #3c3c3c;
    --color-text-primary: #cccccc;
    --color-text-secondary: #808080;
    --color-accent: #007acc;
    --color-border: #404040;
    --color-fatal: #f44747;
    --color-error: #f44747;
    --color-warn: #d7ba7d;
    --color-info: #569cd6;
    --color-debug: #808080;
    --color-trace: #606060;
    --color-other: #cccccc;
    --header-height: 40px;
    --filebar-height: 44px;
    --control-height: 50px;
    --tabs-height: 40px;
    --status-height: 30px;
  }

  :root[data-theme="light"] {
    --color-bg-primary: #ffffff;
    --color-bg-secondary: #f3f3f3;
    --color-bg-tertiary: #e5e5e5;
    --color-text-primary: #1a1a1a;
    --color-text-secondary: #505050;
    --color-border: #d4d4d4;
    --color-fatal: #c42b2b;
    --color-error: #c42b2b;
    --color-warn: #8a6d00;
    --color-info: #0066cc;
    --color-debug: #666666;
    --color-trace: #888888;
    --color-other: #333333;
  }

  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: var(--color-bg-primary);
    position: relative;
  }

  .main-content {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    border-top: 1px solid var(--color-border);
    border-bottom: 1px solid var(--color-border);
  }

  .log-list-panel {
    flex: 1;
    min-height: 100px;
    overflow: hidden;
  }

  .resize-handle {
    height: 6px;
    background-color: var(--color-bg-secondary);
    cursor: ns-resize;
    flex-shrink: 0;
    position: relative;
    transition: background-color 0.15s ease;
  }

  .resize-handle::after {
    content: '';
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 40px;
    height: 3px;
    background-color: var(--color-border);
    border-radius: 2px;
    transition: background-color 0.15s ease;
  }

  .resize-handle:hover,
  .resize-handle.active {
    background-color: var(--color-accent);
  }

  .resize-handle:hover::after,
  .resize-handle.active::after {
    background-color: var(--color-text-primary);
  }

  .log-detail-panel {
    flex-shrink: 0;
    min-height: 100px;
    overflow: hidden;
  }

  .drop-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 122, 204, 0.2);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    pointer-events: none;
  }

  .drop-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 32px 64px;
    background-color: var(--color-bg-secondary);
    border: 2px dashed var(--color-accent);
    border-radius: 8px;
    color: var(--color-text-primary);
  }

  .app-container.dragging {
    background-color: rgba(0, 122, 204, 0.05);
  }
</style>
