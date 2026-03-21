<script lang="ts">
  import TitleBar from '$lib/components/TitleBar.svelte';
  import ControlBar from '$lib/components/ControlBar.svelte';
  import CategoryTabs from '$lib/components/CategoryTabs.svelte';
  import LogList from '$lib/components/LogList.svelte';
  import LogDetail from '$lib/components/LogDetail.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import { logStore } from '$lib/stores/logStore';

  let selectedLogId = $state<number | null>(null);
  let activeCategory = $state<string>('all');
  let logCount = $state<number>(0);
  let isDragging = $state(false);

  $effect(() => {
    const unsubscribe = logStore.subscribe(() => {
      logCount = logStore.getFilteredLogs().length;
    });
    return unsubscribe;
  });

  function handleLogSelect(id: number) {
    selectedLogId = id;
  }

  function handleCategoryChange(category: string) {
    activeCategory = category;
    logStore.setActiveCategory(category);
  }

  function handleSearch(query: string) {
    logStore.setSearchQuery(query);
  }

  function handleTimeFilter(filter: string) {
    logStore.setTimeFilter(filter);
  }

  function handleRefresh() {
    selectedLogId = null;
    logStore.clear();
  }

  async function handleOpenFile() {
    try {
      await logStore.openFileDialog();
    } catch (err) {
      console.error('Failed to open file:', err);
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
        try {
          const path = (file as any).path || file.name;
          await logStore.loadFile(path);
        } catch (err) {
          console.error('Failed to load file:', err);
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
      logStore.setSearchQuery('');
      logStore.setTimeFilter('all');
    }
  }
</script>

<svelte:head>
  <title>LogViewer</title>
</svelte:head>

<svelte:window onkeydown={handleKeydown} />

<div 
  class="app-container"
  class:dragging={isDragging}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
>
  <TitleBar />
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
    --control-height: 50px;
    --tabs-height: 40px;
    --status-height: 30px;
  }

  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  html, body {
    height: 100%;
    overflow: hidden;
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    font-size: 13px;
    color: var(--color-text-primary);
    background-color: var(--color-bg-primary);
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
