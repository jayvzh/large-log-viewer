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

  let selectedLogId = $state<number | null>(null);
  let activeCategory = $state<string>('all');
  let logCount = $state<number>(0);
  let isDragging = $state(false);
  let showSettings = $state(false);
  let showHelp = $state(false);
  let statusMessage = $state('');
  let loadingProgress = $state(0);
  let loadingPhase = $state('');
  let loadStartTime = $state(0);
  let loadTime = $state(0);
  let totalEntries = $state(0);

  $effect(() => {
    const unsubscribe = logStore.subscribe(() => {
      logCount = logStore.getFilteredLogs().length;
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

  function handleLogSelect(id: number) {
    selectedLogId = id;
  }

  function handleCategoryChange(category: string) {
    activeCategory = category;
    logStore.setActiveCategory(category);
  }

  function handleSearch(query: string, scope: string, mode: string) {
    logStore.setSearchQuery(query, scope, mode);
  }

  function handleTimeFilter(filter: string) {
    logStore.setTimeFilter(filter);
  }

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
      if (file.name.endsWith('.log') || file.name.endsWith('.txt') || file.name.endsWith('.json')) {
        statusMessage = '正在加载文件...';
        loadStartTime = Date.now();
        try {
          const path = (file as any).path || file.name;
          await logStore.loadFile(path);
          loadTime = Date.now() - loadStartTime;
          totalEntries = logStore.getStats().all;
          statusMessage = '';
        } catch (err) {
          console.error('Failed to load file:', err);
          statusMessage = '加载文件失败';
        }
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'F5') {
      e.preventDefault();
      handleRefresh();
    } else if (e.ctrlKey && e.key === 'o') {
      e.preventDefault();
      handleOpenFile();
    } else if (e.ctrlKey && e.key === 'f') {
      e.preventDefault();
      const searchInput = document.querySelector('.search-box input') as HTMLInputElement;
      if (searchInput) {
        searchInput.focus();
      }
    } else if (e.key === 'Escape') {
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
    <div class="log-detail-panel">
      <LogDetail logId={selectedLogId} />
    </div>
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
    --filebar-height: 36px;
    --control-height: 50px;
    --tabs-height: 40px;
    --status-height: 30px;
  }

  :root[data-theme="light"] {
    --color-bg-primary: #ffffff;
    --color-bg-secondary: #f3f3f3;
    --color-bg-tertiary: #e5e5e5;
    --color-text-primary: #333333;
    --color-text-secondary: #666666;
    --color-border: #d4d4d4;
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
    flex: 1;
    overflow: hidden;
    border-top: 1px solid var(--color-border);
    border-bottom: 1px solid var(--color-border);
  }

  .log-list-panel {
    flex: 1;
    min-width: 300px;
    overflow: hidden;
    border-right: 1px solid var(--color-border);
  }

  .log-detail-panel {
    flex: 1;
    min-width: 300px;
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
