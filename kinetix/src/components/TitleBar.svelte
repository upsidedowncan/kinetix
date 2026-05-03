<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { goto } from '$app/navigation';
  import { ArrowLeft, Minus, Square, X } from 'lucide-svelte';

  import { Settings, FileVideo, ChevronDown } from 'lucide-svelte';
  
  export let showBackButton: boolean = false;
  export let breadcrumbs: { label: string; onClick?: () => void }[] = [];
  export let onOpenSequenceSettings: () => void = () => {};

  let currentWindow = getCurrentWindow();
  let showFileMenu = false;
  let showEditMenu = false;
  let showViewMenu = false;

  const menus = ['File', 'Edit', 'View', 'Project', 'Window', 'Help'];
  let activeMenu: string | null = null;


  async function minimize() {
    await currentWindow.minimize();
  }

  async function maximize() {
    if (await currentWindow.isMaximized()) {
      await currentWindow.unmaximize();
    } else {
      await currentWindow.maximize();
    }
  }

  async function close() {
    await currentWindow.close();
  }

  function goBack() {
    goto('/');
  }
</script>

<div class="title-bar bg-[#141414] h-10 flex items-center justify-between px-3 select-none border-b border-zinc-800" style="-webkit-app-region: drag" data-tauri-drag-region>
  <div class="flex items-center gap-2">
    {#if showBackButton}
      <button
        on:click={goBack}
        style="-webkit-app-region: no-drag"
        class="flex items-center gap-1.5 px-2 py-1 rounded-md hover:bg-zinc-800 transition-colors text-zinc-400 hover:text-white"
        aria-label="Back to dashboard"
      >
        <ArrowLeft size={12} />
        <span class="text-[11px] font-medium">Dashboard</span>
      </button>
      <div class="w-px h-4 bg-zinc-700"></div>
    {/if}
    <div class="w-6 h-6 bg-blue-600 rounded-md flex items-center justify-center ml-1">
      <span class="text-white font-bold text-[10px]">K</span>
    </div>
    
    <div class="flex items-center h-full ml-2" style="-webkit-app-region: no-drag">
      {#each menus as menu}
        <button
          class="px-3 h-7 text-[11px] text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200 rounded-md transition-colors"
          on:click={() => {
            if (menu === 'Project') {
              // Open project settings directly or show a menu
            }
          }}
        >
          {menu}
        </button>
      {/each}
    </div>

    <div class="w-px h-4 bg-zinc-800 mx-2"></div>

    <button
      on:click={onOpenSequenceSettings}
      style="-webkit-app-region: no-drag"
      class="flex items-center gap-1.5 px-2 py-1 rounded-md hover:bg-zinc-800 transition-colors text-zinc-400 hover:text-blue-400"
    >
      <FileVideo size={12} />
      <span class="text-[11px] font-medium tracking-tight">Sequence Settings</span>
    </button>

    {#if breadcrumbs.length > 0}
      <div class="flex items-center gap-2 ml-4">
        {#each breadcrumbs as crumb, i}
          <div class="flex items-center gap-2">
            <span class="text-zinc-600 text-[10px]">/</span>
            {#if crumb.onClick}
              <button 
                on:click={crumb.onClick}
                class="text-[11px] font-medium text-zinc-400 hover:text-white transition-colors"
                style="-webkit-app-region: no-drag"
              >
                {crumb.label}
              </button>
            {:else}
              <span class="text-[11px] font-medium text-zinc-200">
                {crumb.label}
              </span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="flex items-center gap-0.5">
    <button
      on:click={minimize}
      style="-webkit-app-region: no-drag"
      class="w-8 h-7 flex items-center justify-center rounded-md hover:bg-zinc-800 transition-colors text-zinc-400 hover:text-white"
      aria-label="Minimize"
    >
      <Minus size={12} />
    </button>
    <button
      on:click={maximize}
      style="-webkit-app-region: no-drag"
      class="w-8 h-7 flex items-center justify-center rounded-md hover:bg-zinc-800 transition-colors text-zinc-400 hover:text-white"
      aria-label="Maximize"
    >
      <Square size={10} />
    </button>
    <button
      on:click={close}
      style="-webkit-app-region: no-drag"
      class="w-8 h-7 flex items-center justify-center rounded-md hover:bg-red-600 transition-colors text-zinc-400 hover:text-white"
      aria-label="Close"
    >
      <X size={12} />
    </button>
  </div>
</div>
