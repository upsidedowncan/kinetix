<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Plus, Film, Music, Image, Palette, Sparkles, Type } from 'lucide-svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { convertFileSrc } from '@tauri-apps/api/core';

  export let activeTab: 'media' | 'filters' = 'media';
  export let mediaFiles: { id: string; name: string; src: string; type: string; duration: number; thumbnail?: string }[] = [];
  export let onMediaSelect: (mediaItem: any) => void = () => {};

  const dispatch = createEventDispatcher();
  
  // Fallback: Get video duration using browser's built-in parser
  function getDurationFromBrowser(videoSrc: string): Promise<number> {
    return new Promise((resolve) => {
      const video = document.createElement('video');
      video.preload = 'metadata';
      
      video.onloadedmetadata = () => {
        console.log('Browser fallback duration:', video.duration);
        resolve(video.duration || 0);
        URL.revokeObjectURL(video.src);
      };
      
      video.onerror = () => {
        console.error('Browser fallback failed');
        resolve(0);
        URL.revokeObjectURL(video.src);
      };
      
      // Load via blob to ensure it works with tauri convertFileSrc
      fetch(videoSrc)
        .then(r => r.blob())
        .then(blob => {
          video.src = URL.createObjectURL(blob);
        })
        .catch(() => {
          video.src = videoSrc;
        });
    });
  }

  const filters = [
    { id: 'brightness', name: 'Brightness', category: 'Basic' },
    { id: 'contrast', name: 'Contrast', category: 'Basic' },
    { id: 'saturation', name: 'Saturation', category: 'Color' },
    { id: 'sepia', name: 'Sepia', category: 'Color Grading' },
    { id: 'vintage', name: 'Vintage', category: 'Color Grading' },
    { id: 'cinematic', name: 'Cinematic', category: 'Color Grading' },
    { id: 'chromakey', name: 'Chroma Key', category: 'Effects' },
    { id: 'blur', name: 'Gaussian Blur', category: 'Effects' },
    { id: 'invert', name: 'Invert', category: 'Effects' },
    { id: 'distortion', name: 'Distortion', category: 'Effects' },
    { id: 'rgbsplit', name: 'RGB Split', category: 'Effects' },
    { id: 'pixelate', name: 'Pixelate', category: 'Effects' },
    { id: 'vignette', name: 'Vignette', category: 'Effects' },
    { id: 'hueshift', name: 'Hue Shift', category: 'Color' },
  ];

  const transitions = [
    { id: 'fade', name: 'Fade', category: 'Transitions' },
    { id: 'scale', name: 'Scale Pop', category: 'Transitions' },
    { id: 'rotate', name: 'Rotate Pop', category: 'Transitions' },
    { id: 'slide-up', name: 'Slide Up', category: 'Transitions' },
    { id: 'slide-down', name: 'Slide Down', category: 'Transitions' },
    { id: 'slide-left', name: 'Slide Left', category: 'Transitions' },
    { id: 'slide-right', name: 'Slide Right', category: 'Transitions' },
  ];

  function getTypeIcon(type: string) {
    if (type.startsWith('video')) return Film;
    if (type.startsWith('audio')) return Music;
    if (type.startsWith('image')) return Image;
    return Film;
  }

  function formatDuration(seconds: number): string {
    if (!isFinite(seconds) || seconds < 0) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
</script>

<div class="h-full flex flex-col bg-[#141414] border-r border-zinc-800 overflow-hidden">
  <!-- Tab Navigation -->
  <div class="flex border-b border-zinc-800 shrink-0 bg-zinc-900/50">
    <button
      class="flex-1 py-3 text-[10px] font-bold uppercase tracking-wider transition-colors flex items-center justify-center gap-1.5 {activeTab === 'media'
        ? 'text-blue-400 bg-zinc-800/50 border-b-2 border-blue-500'
        : 'text-zinc-500 hover:text-zinc-300'}"
      on:click={() => (activeTab = 'media')}
    >
      <Film size={12} />
      Project
    </button>
    <button
      class="flex-1 py-3 text-[10px] font-bold uppercase tracking-wider transition-colors flex items-center justify-center gap-1.5 {activeTab === 'filters'
        ? 'text-blue-400 bg-zinc-800/50 border-b-2 border-blue-500'
        : 'text-zinc-500 hover:text-zinc-300'}"
      on:click={() => (activeTab = 'filters')}
    >
      <Sparkles size={12} />
      Effects
    </button>
  </div>

  <!-- Tab Content -->
  <div class="flex-1 overflow-y-auto custom-scrollbar">
    {#if activeTab === 'media'}
      <div class="p-3 space-y-4">
        <div class="flex items-center justify-between">
          <span class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest">Media Pool</span>
          <div class="flex items-center gap-1.5">
            <button
              on:click={() => dispatch('addtext')}
              class="p-1.5 bg-zinc-700 hover:bg-zinc-600 text-white rounded-md transition-colors"
              title="Add Text"
            >
              <Type size={14} />
            </button>
            <button
              on:click={async () => {
              try {
                const selected = await open({
                  multiple: false,
                  filters: [
                    { name: 'Video', extensions: ['mp4', 'mov', 'avi', 'mkv', 'webm'] },
                    { name: 'Image', extensions: ['jpg', 'jpeg', 'png', 'gif', 'webp'] },
                    { name: 'All Files', extensions: ['*'] }
                  ]
                });
                
                if (!selected || typeof selected !== 'string') return;
                
                const fileName = selected.split('/').pop() || selected.split('\\').pop() || 'Unknown';
                const ext = fileName.split('.').pop()?.toLowerCase() || '';
                const extToMime: Record<string, string> = {
                  'mp4': 'video/mp4', 'mov': 'video/quicktime', 'avi': 'video/x-msvideo',
                  'mkv': 'video/x-matroska', 'webm': 'video/webm',
                  'jpg': 'image/jpeg', 'jpeg': 'image/jpeg', 'png': 'image/png',
                  'gif': 'image/gif', 'webp': 'image/webp'
                };
                const mimeType = extToMime[ext] || 'video/mp4';
                
                // Use convertFileSrc for all files - asset protocol is now enabled
                let src = convertFileSrc(selected);
                let originalPath = selected;
                
                let finalDuration = mimeType.startsWith('image') ? 5 : 0;
                
                // For videos, generate proxy and get metadata
                if (mimeType.startsWith('video')) {
                  try {
                    console.log('Generating proxy and thumbnail for:', fileName);
                    const proxyData: any = await invoke('generate_proxy_video', { filePath: selected });
                    console.log('Proxy/Thumb generated:', proxyData);
                    
                    // CRITICAL FIX: Read proxy as blob to bypass asset:// protocol issues on Linux
                    console.log('Sidebar: Reading proxy into memory for reliable playback...');
                    const bytes: number[] = await invoke('read_video_file', { filePath: proxyData.proxy_path });
                    const blob = new Blob([new Uint8Array(bytes)], { type: 'video/webm' });
                    src = URL.createObjectURL(blob);
                    
                    const thumbnail = convertFileSrc(proxyData.thumbnail_path);

                    if (proxyData.duration_secs > 0) {
                      finalDuration = proxyData.duration_secs;
                    }

                    dispatch('import', { 
                      name: fileName, 
                      src, 
                      type: mimeType, 
                      duration: finalDuration,
                      filePath: originalPath,
                      thumbnail
                    });
                    return; // Early return since we dispatched
                  } catch (e) {
                    console.error('Proxy generation error, falling back to original:', e);
                    // Fallback to original metadata if proxy fails
                    try {
                      const meta: any = await invoke('get_video_metadata', { filePath: selected });
                      if (meta.duration_secs > 0) finalDuration = meta.duration_secs;
                    } catch (innerE) {
                      // Final fallback: duration extraction will happen in addMedia via VideoPreview if needed
                    }
                  }
                }
                
                dispatch('import', { 
                  name: fileName, 
                  src, 
                  type: mimeType, 
                  duration: finalDuration,
                  filePath: originalPath
                });
              } catch (error) {
                console.error('Failed to open file dialog:', error);
              }
            }}
              class="p-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-md transition-colors"
              title="Import Media"
            >
              <Plus size={14} />
            </button>
          </div>
        </div>

        {#if mediaFiles.length === 0}
          <div class="py-12 flex flex-col items-center justify-center border-2 border-dashed border-zinc-800/50 rounded-xl space-y-2 opacity-50">
            <Film size={32} strokeWidth={1} />
            <p class="text-[10px] font-bold uppercase">No Media</p>
          </div>
        {:else}
          <div class="grid grid-cols-1 gap-2">
            {#each mediaFiles as file}
              {@const Icon = getTypeIcon(file.type)}
              <div
                class="group bg-zinc-800/30 border border-zinc-800 rounded-lg p-2 hover:bg-zinc-800/60 transition-all cursor-grab active:cursor-grabbing"
                draggable="true"
                on:dragstart={(e) => {
                  e.dataTransfer?.setData('text/plain', file.id);
                  e.dataTransfer!.effectAllowed = 'copy';
                }}
                on:click={() => onMediaSelect(file)}
                on:keypress={(e) => e.key === 'Enter' && onMediaSelect(file)}
                role="button"
                tabindex="0"
              >
                <div class="flex gap-3 items-center">
                  <div class="w-16 aspect-video bg-zinc-900 rounded-md flex items-center justify-center shrink-0 border border-zinc-800 group-hover:border-zinc-700 transition-colors overflow-hidden">
                    {#if file.thumbnail}
                      <img src={file.thumbnail} alt="" class="w-full h-full object-cover" />
                    {:else}
                      <Icon size={16} class="text-zinc-600 group-hover:text-zinc-400 transition-colors" strokeWidth={1.5} />
                    {/if}
                  </div>
                  <div class="min-w-0 flex-1">
                    <p class="text-[11px] font-medium text-zinc-300 truncate">{file.name}</p>
                    <p class="text-[9px] font-bold text-zinc-600 uppercase tabular-nums">{formatDuration(file.duration)}</p>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {:else}
      <div class="p-3">
        {#each ['Basic', 'Color', 'Color Grading', 'Effects', 'Transitions'] as category}
          <div class="mb-6 last:mb-0">
            <h4 class="text-[10px] font-bold text-zinc-600 mb-3 uppercase tracking-widest px-1">{category}</h4>
            <div class="grid grid-cols-1 gap-1.5">
              {#if category === 'Transitions'}
                {#each transitions as transition}
                  <div
                    class="flex items-center gap-3 px-3 py-2.5 bg-blue-500/5 border border-blue-500/10 rounded-lg text-xs font-medium text-blue-300/80 hover:bg-blue-500/15 hover:text-blue-200 transition-all cursor-grab active:cursor-grabbing group"
                    draggable="true"
                    on:dragstart={(e) => {
                      e.dataTransfer?.setData('text/transition', transition.id);
                      e.dataTransfer!.effectAllowed = 'copy';
                    }}
                  >
                    <Sparkles size={12} class="text-blue-600 group-hover:text-blue-400 transition-colors" />
                    <span class="tracking-tight truncate">{transition.name}</span>
                  </div>
                {/each}
              {:else}
                {#each filters.filter(f => f.category === category) as filter}
                  <div
                    class="flex items-center gap-3 px-3 py-2.5 bg-zinc-800/30 border border-zinc-800/50 rounded-lg text-xs font-medium text-zinc-400 hover:bg-zinc-800/80 hover:text-zinc-200 transition-all cursor-grab active:cursor-grabbing group"
                    draggable="true"
                    on:dragstart={(e) => {
                      e.dataTransfer?.setData('text/effect', filter.id);
                      e.dataTransfer!.effectAllowed = 'copy';
                    }}
                  >
                    <Palette size={12} class="text-zinc-600 group-hover:text-blue-500 transition-colors" />
                    <span class="tracking-tight truncate">{filter.name}</span>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #27272a;
    border-radius: 10px;
  }
</style>
