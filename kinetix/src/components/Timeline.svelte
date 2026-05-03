<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Diamond, MousePointer2, Scissors, Trash2, Sparkles } from 'lucide-svelte';
  
  export let currentTime = 0;
  export let projectDuration = 60;
  export let tracks: any[][] = [[], [], []];
  export let activeTool: 'select' | 'razor' = 'select';
  export let clipProperties: Record<string, any> = {};

  const dispatch = createEventDispatcher();
  let container: HTMLDivElement;
  let containerWidth = 0;
  let isDragging = false;

  let resizingClip: { id: string, edge: 'left' | 'right', startX: number, startVal: number, startDur: number } | null = null;
  let resizingTransition: { clipId: string, position: 'start' | 'end', startX: number, startDuration: number } | null = null;
  const TRACK_GUTTER_PX = 48;

  $: trackWidth = Math.max(1, containerWidth - TRACK_GUTTER_PX);
  $: playheadPx = TRACK_GUTTER_PX + (currentTime / projectDuration) * trackWidth;

  function clampToTrack(clientX: number, rect: DOMRect) {
    return Math.max(0, Math.min(clientX - rect.left - TRACK_GUTTER_PX, trackWidth));
  }

  function clientXToTime(clientX: number, rect: DOMRect) {
    const x = clampToTrack(clientX, rect);
    return (x / trackWidth) * projectDuration;
  }

  function getClipKeyframeTimes(clip: any): number[] {
    const props = clipProperties?.[clip.id];
    if (!props) return [];

    const times: number[] = [];

    const transformKeyframes = props.transform?.keyframes;
    if (transformKeyframes) {
      Object.values(transformKeyframes).forEach((kfs: any) => {
        if (Array.isArray(kfs)) {
          kfs.forEach((kf: any) => {
            if (typeof kf?.time === 'number') times.push(kf.time);
          });
        }
      });
    }

    if (Array.isArray(props.effects)) {
      props.effects.forEach((effect: any) => {
        if (Array.isArray(effect?.params)) {
          effect.params.forEach((param: any) => {
            if (Array.isArray(param?.keyframes)) {
              param.keyframes.forEach((kf: any) => {
                if (typeof kf?.time === 'number') times.push(kf.time);
              });
            }
          });
        }
      });
    }

    return [...new Set(times.map((t) => Number(t.toFixed(3))))]
      .filter((time) => time >= 0 && time <= clip.duration)
      .sort((a, b) => a - b);
  }

  function onMouseDown(e: MouseEvent) {
    const target = e.target as HTMLElement;
    
    // Handle razor tool
    if (activeTool === 'razor') {
      const clipBtn = target.closest('.clip-btn') as HTMLElement;
      if (clipBtn) {
        const rect = container.getBoundingClientRect();
        const clickTime = clientXToTime(e.clientX, rect);
        // Find clip info
        const name = clipBtn.dataset.name;
        for (const track of tracks) {
          const clip = track.find(c => c.name === name);
          if (clip) {
            dispatch('clipcut', { clipId: clip.id, cutTime: clickTime });
            return;
          }
        }
      }
    }

    if (target.closest('.resize-handle')) return;
    if (target.closest('.delete-btn')) return;
    // Note: clip-btn clicks are handled by the clip's own on:click handler
    
    isDragging = true;
    handleMouseMove(e);
  }

  function startResize(e: MouseEvent, clip: any, edge: 'left' | 'right') {
    if (activeTool !== 'select') return;
    e.stopPropagation();
    e.preventDefault();
    resizingClip = { 
      id: clip.id, 
      edge, 
      startX: e.clientX, 
      startVal: clip.startTime, 
      startDur: clip.duration 
    };
  }

  function startTransitionResize(e: MouseEvent, clipId: string, position: 'start' | 'end', duration: number) {
    if (activeTool !== 'select') return;
    e.stopPropagation();
    e.preventDefault();
    resizingTransition = {
      clipId,
      position,
      startX: e.clientX,
      startDuration: duration
    };
  }

  function handleMouseMove(e: MouseEvent) {
    if (isDragging) {
      const rect = container.getBoundingClientRect();
      dispatch('seek', clientXToTime(e.clientX, rect));
    } else if (resizingClip) {
      const rect = container.getBoundingClientRect();
      const dx = ((e.clientX - resizingClip.startX) / trackWidth) * projectDuration;
      
      dispatch('clipresize', { 
        id: resizingClip.id, 
        edge: resizingClip.edge, 
        delta: dx,
        initialStart: resizingClip.startVal,
        initialDuration: resizingClip.startDur
      });
    } else if (resizingTransition) {
      const dx = ((e.clientX - resizingTransition.startX) / trackWidth) * projectDuration;
      // 'start' position means moving the right edge of the IN transition
      // 'end' position means moving the left edge of the OUT transition
      const delta = resizingTransition.position === 'start' ? dx : -dx;
      const newDuration = Math.max(0.1, Math.min(5, resizingTransition.startDuration + delta));
      
      dispatch('updatetransitionduration', {
        clipId: resizingTransition.clipId,
        position: resizingTransition.position,
        duration: newDuration
      });
    }
  }

  function onMouseUp() {
    isDragging = false;
    resizingClip = null;
    resizingTransition = null;
  }
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'copy';
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    const effectId = e.dataTransfer?.getData('text/effect');
    const transitionId = e.dataTransfer?.getData('text/transition');
    
    const target = e.target as HTMLElement;
    const clipBtn = target.closest('.clip-btn') as HTMLElement;
    
    if (clipBtn) {
      const name = clipBtn.dataset.name;
      let targetClip: any = null;
      for (const track of tracks) {
        targetClip = track.find(c => c.name === name);
        if (targetClip) break;
      }

      if (targetClip) {
        if (effectId) {
          dispatch('filterdrop', { filterId: effectId, clipId: targetClip.id });
        } else if (transitionId) {
          // Detect if drop is at start or end of clip
          const rect = clipBtn.getBoundingClientRect();
          const relativeX = e.clientX - rect.left;
          const dropPosition = relativeX / rect.width > 0.5 ? 'end' : 'start';
          
          dispatch('transitiondrop', { 
            transitionId, 
            clipId: targetClip.id, 
            position: dropPosition 
          });
        }
      }
    }
  }
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={onMouseUp} />

<div class="h-full flex flex-col bg-[#0a0a0a] overflow-hidden">
  <!-- Toolbar -->
  <div class="h-9 border-b border-zinc-800/50 flex items-center px-4 gap-2 bg-[#111] shrink-0">
    <div class="flex items-center gap-1 bg-zinc-800/50 rounded-lg p-0.5">
      <button 
        type="button"
        class="p-1.5 rounded-md transition-all {activeTool === 'select' ? 'bg-blue-600 text-white shadow-lg shadow-blue-500/20' : 'text-zinc-500 hover:text-zinc-300'}"
        on:click={() => activeTool = 'select'}
        title="Selection Tool (V)"
      >
        <MousePointer2 size={13} />
      </button>
      <button 
        type="button"
        class="p-1.5 rounded-md transition-all {activeTool === 'razor' ? 'bg-red-600 text-white shadow-lg shadow-red-500/20' : 'text-zinc-500 hover:text-zinc-300'}"
        on:click={() => activeTool = 'razor'}
        title="Razor Tool (C)"
      >
        <Scissors size={13} />
      </button>
    </div>
    <div class="w-px h-4 bg-zinc-800/50 mx-1"></div>
    <div class="flex items-center gap-3">
      <span class="text-[10px] text-zinc-400 font-mono bg-zinc-800/30 px-2 py-0.5 rounded border border-zinc-700/30 uppercase tracking-widest">{currentTime.toFixed(2)}s</span>
    </div>
    <span class="text-[9px] text-zinc-600 font-black uppercase ml-auto tracking-[0.2em]">Timeline</span>
  </div>

  <div 
    bind:this={container}
    bind:clientWidth={containerWidth}
    role="application"
    aria-label="Timeline editor"
    class="flex-1 relative overflow-x-auto overflow-y-hidden custom-scrollbar bg-grid-pattern"
    on:mousedown={onMouseDown}
    on:dragover={handleDragOver}
    on:drop={onDrop}
  >
    <!-- Time Ruler -->
    <div class="sticky top-0 left-0 right-0 h-7 border-b border-zinc-800/50 flex items-end bg-[#111]/90 backdrop-blur-md z-30 pointer-events-none">
      {#each Array(Math.ceil(projectDuration / 5) + 1) as _, i}
        {@const time = i * 5}
        <div class="absolute h-full flex flex-col justify-end" style="left: {(time / projectDuration) * 100}%">
          <span class="text-[8px] text-zinc-600 font-mono mb-1.5 -translate-x-1/2">{time}s</span>
          <div class="h-2 w-px bg-zinc-800"></div>
        </div>
        {#each Array(4) as _, j}
          {@const subTime = time + (j + 1)}
          {#if subTime < projectDuration}
            <div class="absolute h-1 w-px bg-zinc-800/50" style="left: {(subTime / projectDuration) * 100}%; bottom: 0"></div>
          {/if}
        {/each}
      {/each}
    </div>

    <!-- Track Lanes -->
    <div class="absolute inset-0 top-7 flex flex-col p-0">
      {#each tracks as track, i}
        <div class="flex-1 group/lane relative border-b border-zinc-800/30 hover:bg-white/[0.01] transition-colors min-h-[40px]">
          <!-- Track Info Sidebar (Sticky) -->
          <div class="sticky left-0 top-0 bottom-0 w-12 bg-[#0a0a0a]/90 backdrop-blur-sm border-r border-zinc-800/50 flex flex-col items-center justify-center z-20 pointer-events-none shadow-[4px_0_10px_rgba(0,0,0,0.3)]">
            <span class="text-[10px] text-zinc-600 font-black tracking-tighter opacity-50 mb-1">{i === 0 ? 'V1' : i === 1 ? 'V2' : 'V3'}</span>
            <div class="w-1.5 h-1.5 rounded-full {i === 0 ? 'bg-blue-500/40' : i === 1 ? 'bg-purple-500/40' : 'bg-green-500/40'} shadow-[0_0_8px_rgba(0,0,0,0.5)]"></div>
          </div>
          
          <div class="absolute inset-0 left-12 flex items-center">
            {#each track as clip}
              <div 
                role="button"
                tabindex="0"
                class="clip-btn absolute top-1.5 bottom-1.5 rounded-md shadow-2xl flex items-center px-0 text-[10px] overflow-hidden whitespace-nowrap group transition-all
                  {activeTool === 'razor' ? 'cursor-crosshair ring-2 ring-red-500/50' : 'cursor-default'}
                  {clip.type?.startsWith('image') ? 'bg-[#2a1b3d] border-purple-500/30 hover:border-purple-400/50 shadow-purple-500/5' : 
                   clip.type?.startsWith('text') ? 'bg-[#3d2b1b] border-amber-500/30 hover:border-amber-400/50 shadow-amber-500/5' : 
                   'bg-[#1b2a3d] border-blue-500/30 hover:border-blue-400/50 shadow-blue-500/5'} border-l-[3px] border-y border-r"
                style="left: {(clip.startTime / projectDuration) * 100}%; width: {(clip.duration / projectDuration) * 100}%"
                data-name={clip.name}
                on:click|stopPropagation={() => {
                  if (activeTool === 'select') {
                    dispatch('clipselect', clip);
                  }
                }}
                on:keydown|stopPropagation={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    dispatch('clipselect', clip);
                  }
                }}
              >
                <!-- Clip Progress/Background Pattern -->
                <div class="absolute inset-0 opacity-10 pointer-events-none bg-stripe-pattern"></div>

                <!-- Resize Handles -->
                {#if activeTool === 'select'}
                  <div 
                    role="slider"
                    aria-label="Resize clip left"
                    aria-valuenow={clip.startTime}
                    tabindex="0"
                    class="resize-handle absolute left-0 top-0 bottom-0 w-3 cursor-ew-resize hover:bg-white/10 z-20 group-hover:bg-white/5 transition-colors"
                    on:mousedown={(e) => startResize(e, clip, 'left')}
                  ></div>
                {/if}
                
                <div class="flex-1 flex items-center justify-between min-w-0 h-full relative px-2">
                  <!-- Transition Indicators (Stylized) -->
                  {#if clipProperties[clip.id]?.transitions?.in?.type && clipProperties[clip.id]?.transitions?.in?.type !== 'none'}
                    {@const transIn = clipProperties[clip.id].transitions.in}
                    <div 
                      class="absolute left-0 top-0 bottom-0 bg-gradient-to-r from-white/10 to-transparent border-r border-white/10 transition-all overflow-hidden group/trans z-10"
                      style="width: {(transIn.duration / clip.duration) * 100}%"
                    >
                      <svg class="w-full h-full opacity-20" preserveAspectRatio="none" viewBox="0 0 100 100">
                        <path d="M 0 100 L 100 0 L 100 100 Z" fill="currentColor" />
                      </svg>
                      
                      {#if activeTool === 'select'}
                        <div 
                          role="slider"
                          aria-label="Resize in transition"
                          aria-valuenow={transIn.duration}
                          tabindex="0"
                          class="absolute right-0 top-0 bottom-0 w-2 cursor-ew-resize hover:bg-white/40 z-20"
                          on:mousedown|stopPropagation={(e) => startTransitionResize(e, clip.id, 'start', transIn.duration)}
                        ></div>
                        <button 
                          type="button"
                          class="absolute top-1 left-1 p-0.5 bg-black/60 text-white/40 hover:text-red-400 opacity-0 group-hover/trans:opacity-100 transition-all rounded-md backdrop-blur-sm z-30"
                          on:click|stopPropagation={() => dispatch('updatetransitionduration', { clipId: clip.id, position: 'start', type: 'none', duration: 0 })}
                          title="Remove transition"
                        >
                          <Trash2 size={9} />
                        </button>
                      {/if}
                    </div>
                  {/if}

                  {#if clipProperties[clip.id]?.transitions?.out?.type && clipProperties[clip.id]?.transitions?.out?.type !== 'none'}
                    {@const transOut = clipProperties[clip.id].transitions.out}
                    <div 
                      class="absolute right-0 top-0 bottom-0 bg-gradient-to-l from-white/10 to-transparent border-l border-white/10 transition-all overflow-hidden group/trans z-10"
                      style="width: {(transOut.duration / clip.duration) * 100}%"
                    >
                      <svg class="w-full h-full opacity-20" preserveAspectRatio="none" viewBox="0 0 100 100">
                        <path d="M 0 0 L 100 100 L 0 100 Z" fill="currentColor" />
                      </svg>

                      {#if activeTool === 'select'}
                        <div 
                          role="slider"
                          aria-label="Resize out transition"
                          aria-valuenow={transOut.duration}
                          tabindex="0"
                          class="absolute left-0 top-0 bottom-0 w-2 cursor-ew-resize hover:bg-white/40 z-20"
                          on:mousedown|stopPropagation={(e) => startTransitionResize(e, clip.id, 'end', transOut.duration)}
                        ></div>
                        <button 
                          type="button"
                          class="absolute top-1 right-1 p-0.5 bg-black/60 text-white/40 hover:text-red-400 opacity-0 group-hover/trans:opacity-100 transition-all rounded-md backdrop-blur-sm z-30"
                          on:click|stopPropagation={() => dispatch('updatetransitionduration', { clipId: clip.id, position: 'end', type: 'none', duration: 0 })}
                          title="Remove transition"
                        >
                          <Trash2 size={9} />
                        </button>
                      {/if}
                    </div>
                  {/if}

                  <div class="flex items-center gap-2 min-w-0 relative z-20">
                    <span class="truncate font-bold text-zinc-100/90 tracking-tight">{clip.name}</span>
                    {#if clipProperties[clip.id]?.effects?.length > 0}
                      <div class="flex items-center gap-1 px-1.5 py-0.5 rounded-full bg-blue-500/20 text-blue-400 border border-blue-500/20 scale-[0.85] origin-left backdrop-blur-sm">
                        <Sparkles size={8} />
                        <span class="text-[7px] font-black uppercase">FX</span>
                      </div>
                    {/if}
                  </div>

                  {#if getClipKeyframeTimes(clip).length > 0}
                    <div class="absolute inset-x-0 bottom-0.5 h-3 pointer-events-none z-20">
                      {#each getClipKeyframeTimes(clip) as keyframeTime}
                        <div
                          class="absolute -translate-x-1/2 text-amber-300/90 drop-shadow-[0_0_4px_rgba(251,191,36,0.45)]"
                          style="left: {(keyframeTime / clip.duration) * 100}%"
                          title={`Keyframe at ${keyframeTime.toFixed(2)}s`}
                        >
                          <Diamond size={9} fill="currentColor" strokeWidth={1.5} />
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>

                <!-- Delete Button -->
                {#if activeTool === 'select'}
                  <button 
                    type="button"
                    class="delete-btn p-1.5 rounded-md hover:bg-red-500/20 text-white/20 hover:text-red-400 transition-all pointer-events-auto ml-1 opacity-0 group-hover:opacity-100 mr-1 backdrop-blur-md"
                    on:click|stopPropagation={() => dispatch('clipdelete', { clipId: clip.id })}
                  >
                    <Trash2 size={11} />
                  </button>
                  <div 
                    role="slider"
                    aria-label="Resize clip right"
                    aria-valuenow={clip.duration}
                    tabindex="0"
                    class="resize-handle absolute right-0 top-0 bottom-0 w-3 cursor-ew-resize hover:bg-white/10 z-20 group-hover:bg-white/5 transition-colors"
                    on:mousedown={(e) => startResize(e, clip, 'right')}
                  ></div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/each}
      
      <!-- Placeholder for more tracks -->
      <div class="flex-1 bg-zinc-900/10 pointer-events-none"></div>
    </div>

    <!-- Playhead (Professional look) -->
    <div 
      class="absolute top-0 bottom-0 w-[2px] bg-red-500 z-50 pointer-events-none shadow-[0_0_15px_rgba(239,68,68,0.4)]"
      style="left: {playheadPx}px"
    >
      <div class="absolute top-0 left-1/2 -translate-x-1/2 w-4 h-5 bg-red-500 clip-playhead shadow-lg flex items-center justify-center">
        <div class="w-0.5 h-2 bg-white/40 rounded-full"></div>
      </div>
    </div>
  </div>
</div>

<style>
  .clip-playhead {
    clip-path: polygon(0% 0%, 100% 0%, 100% 70%, 50% 100%, 0% 70%);
  }
  
  .custom-scrollbar::-webkit-scrollbar {
    height: 6px;
    width: 6px;
  }
  
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #27272a;
    border-radius: 10px;
  }
  
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: #3f3f46;
  }

  .bg-grid-pattern {
    background-image: linear-gradient(to right, #ffffff03 1px, transparent 1px);
    background-size: 10% 100%;
  }

  .bg-stripe-pattern {
    background: repeating-linear-gradient(
      45deg,
      transparent,
      transparent 10px,
      rgba(255, 255, 255, 0.03) 10px,
      rgba(255, 255, 255, 0.03) 20px
    );
  }
</style>
