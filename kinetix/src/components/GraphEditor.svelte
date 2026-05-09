<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { Diamond, ChevronDown, ChevronRight, X, Trash2 } from 'lucide-svelte';

  export let selectedClip: any = null;
  export let properties: any = null; // Current properties of selected clip
  export let currentTime = 0;
  export let duration = 0;

  const dispatch = createEventDispatcher();

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let containerWidth = 0;
  let containerHeight = 0;

  let zoomX = 1;
  let scrollX = 0;
  let zoomY = 1;
  let scrollY = 0;

  let isDraggingKeyframe = false;
  let dragKeyframe: { property: string; index: number } | null = null;
  let dragStartX = 0;
  let dragStartY = 0;
  let dragBaseTime = 0;
  let dragBaseValue = 0;

  const propertyConfigs: Record<string, { label: string; color: string; min: number; max: number; default: number }> = {
    x: { label: 'Position X', color: '#ff4444', min: -1000, max: 1000, default: 0 },
    y: { label: 'Position Y', color: '#44ff44', min: -1000, max: 1000, default: 0 },
    scale: { label: 'Scale', color: '#4444ff', min: 0.1, max: 10, default: 1 },
    scaleX: { label: 'Scale X', color: '#ff44ff', min: 0.1, max: 10, default: 1 },
    scaleY: { label: 'Scale Y', color: '#ffff44', min: 0.1, max: 10, default: 1 },
    rotation: { label: 'Rotation', color: '#44ffff', min: -360, max: 360, default: 0 },
    opacity: { label: 'Opacity', color: '#ffffff', min: 0, max: 1, default: 1 }
  };

  let activeProperties = ['x', 'y', 'scale', 'rotation', 'opacity'];
  let visibleProperties: Set<string> = new Set(['x', 'y']);

  $: keyframes = properties?.transform?.keyframes || {};
  $: relTime = currentTime - (selectedClip?.startTime || 0);

  function timeToX(time: number) {
    return (time * 100 * zoomX) - scrollX + 50;
  }

  function xToTime(x: number) {
    return (x + scrollX - 50) / (100 * zoomX);
  }

  function valueToY(value: number, property: string) {
    const config = propertyConfigs[property] || { min: 0, max: 100 };
    const range = config.max - config.min;
    const normalized = (value - config.min) / range;
    return containerHeight - (normalized * (containerHeight - 100) * zoomY + 50) + scrollY;
  }

  function yToValue(y: number, property: string) {
    const config = propertyConfigs[property] || { min: 0, max: 100 };
    const range = config.max - config.min;
    const normalized = (containerHeight - y + scrollY - 50) / ((containerHeight - 100) * zoomY);
    return normalized * range + config.min;
  }

  function render() {
    if (!canvas || !ctx) return;
    ctx.clearRect(0, 0, containerWidth, containerHeight);

    // Draw Grid
    ctx.strokeStyle = '#222';
    ctx.lineWidth = 1;
    ctx.beginPath();
    const gridDuration = Math.max(duration, 20);
    for (let i = 0; i < gridDuration; i++) {
      const x = timeToX(i);
      ctx.moveTo(x, 0);
      ctx.lineTo(x, containerHeight);
    }
    ctx.stroke();

    // Draw Playhead
    const playheadX = timeToX(relTime);
    ctx.strokeStyle = '#3b82f6';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(playheadX, 0);
    ctx.lineTo(playheadX, containerHeight);
    ctx.stroke();

    // Draw Curves
    Object.keys(keyframes).forEach(prop => {
      if (!visibleProperties.has(prop)) return;
      const kfs = keyframes[prop];
      if (!kfs || kfs.length === 0) return;

      const config = propertyConfigs[prop];
      ctx.strokeStyle = config?.color || '#fff';
      ctx.lineWidth = 2;
      ctx.beginPath();

      for (let i = 0; i < kfs.length; i++) {
        const x = timeToX(kfs[i].time);
        const y = valueToY(kfs[i].value, prop);
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();

      // Draw Keyframe Points
      kfs.forEach((kf: any, index: number) => {
        const x = timeToX(kf.time);
        const y = valueToY(kf.value, prop);
        const isSelected = dragKeyframe?.property === prop && dragKeyframe?.index === index;
        
        ctx.fillStyle = isSelected ? '#fff' : (config?.color || '#fff');
        ctx.beginPath();
        ctx.arc(x, y, isSelected ? 6 : 4, 0, Math.PI * 2);
        ctx.fill();
        ctx.strokeStyle = '#000';
        ctx.lineWidth = 1;
        ctx.stroke();
      });
    });
  }

  $: if (containerWidth || containerHeight || keyframes || relTime || zoomX || scrollX || zoomY || scrollY || visibleProperties) {
    render();
  }

  function handleMouseDown(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    // Check for keyframe selection
    for (const prop of Array.from(visibleProperties)) {
      const kfs = keyframes[prop];
      if (!kfs) continue;

      for (let i = 0; i < kfs.length; i++) {
        const kf = kfs[i];
        const kfX = timeToX(kf.time);
        const kfY = valueToY(kf.value, prop);

        const dist = Math.sqrt((mouseX - kfX) ** 2 + (mouseY - kfY) ** 2);
        if (dist < 10) {
          isDraggingKeyframe = true;
          dragKeyframe = { property: prop, index: i };
          dragStartX = e.clientX;
          dragStartY = e.clientY;
          dragBaseTime = kf.time;
          dragBaseValue = kf.value;
          return;
        }
      }
    }

    // Default: seek or pan
  }

  function handleMouseMove(e: MouseEvent) {
    if (isDraggingKeyframe && dragKeyframe) {
      const dx = e.clientX - dragStartX;
      const dy = e.clientY - dragStartY;

      const deltaTime = dx / (100 * zoomX);
      const config = propertyConfigs[dragKeyframe.property];
      const range = config.max - config.min;
      const deltaValue = -(dy / ((containerHeight - 100) * zoomY)) * range;

      const newTime = Math.max(0, Math.min(selectedClip.duration, dragBaseTime + deltaTime));
      const newValue = Math.max(config.min, Math.min(config.max, dragBaseValue + deltaValue));

      dispatch('updatekeyframe', {
        property: dragKeyframe.property,
        index: dragKeyframe.index,
        time: newTime,
        value: newValue
      });
    }
  }

  function handleMouseUp() {
    isDraggingKeyframe = false;
  }

  function handleWheel(e: WheelEvent) {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      const zoomSpeed = 0.001;
      const delta = -e.deltaY;
      zoomX = Math.max(0.1, Math.min(10, zoomX * (1 + delta * zoomSpeed)));
    } else {
      scrollX = Math.max(0, scrollX + e.deltaX);
      scrollY = scrollY + e.deltaY;
    }
  }

  function togglePropertyVisibility(prop: string) {
    if (visibleProperties.has(prop)) {
      visibleProperties.delete(prop);
    } else {
      visibleProperties.add(prop);
    }
    visibleProperties = new Set(visibleProperties);
  }

  onMount(() => {
    ctx = canvas.getContext('2d')!;
    render();
  });
</script>

<div class="flex h-full bg-[#0d0d0d] overflow-hidden" bind:clientWidth={containerWidth} bind:clientHeight={containerHeight}>
  <!-- Sidebar -->
  <div class="w-48 border-r border-zinc-800 bg-[#111] flex flex-col shrink-0">
    <div class="p-3 border-b border-zinc-800">
      <span class="text-[10px] font-black text-zinc-500 uppercase tracking-widest">Properties</span>
    </div>
    <div class="flex-1 overflow-y-auto custom-scrollbar">
      {#each activeProperties as prop}
        <button 
          on:click={() => togglePropertyVisibility(prop)}
          class="w-full px-3 py-2 flex items-center justify-between group transition-colors {visibleProperties.has(prop) ? 'bg-blue-600/10' : 'hover:bg-zinc-800'}"
        >
          <div class="flex items-center gap-2">
            <div class="w-2 h-2 rounded-full" style="background: {propertyConfigs[prop].color}"></div>
            <span class="text-[11px] font-medium {visibleProperties.has(prop) ? 'text-zinc-200' : 'text-zinc-500'}">{propertyConfigs[prop].label}</span>
          </div>
          {#if keyframes[prop]?.length > 0}
            <Diamond size={8} class="fill-current text-blue-500" />
          {/if}
        </button>
      {/each}
    </div>
  </div>

  <!-- Graph -->
  <div class="flex-1 relative overflow-hidden flex flex-col">
    <!-- Toolbar -->
    <div class="h-10 bg-[#111] border-b border-zinc-800 flex items-center px-4 gap-4 shrink-0">
      <div class="flex items-center gap-2">
        <span class="text-[10px] text-zinc-500 uppercase font-bold">Zoom</span>
        <input type="range" min="0.1" max="5" step="0.1" bind:value={zoomX} class="w-24 h-1 accent-blue-500 bg-zinc-800 rounded-lg appearance-none" />
      </div>
      <div class="flex-1"></div>
      {#if dragKeyframe}
        <div class="flex items-center gap-3">
          <div class="flex flex-col items-end">
            <span class="text-[9px] text-zinc-500 uppercase font-black">Time</span>
            <span class="text-[10px] text-blue-400 font-mono">{dragKeyframe ? (keyframes[dragKeyframe.property][dragKeyframe.index].time).toFixed(2) : '0.00'}s</span>
          </div>
          <div class="flex flex-col items-end">
            <span class="text-[9px] text-zinc-500 uppercase font-black">Value</span>
            <span class="text-[10px] text-blue-400 font-mono">{dragKeyframe ? (keyframes[dragKeyframe.property][dragKeyframe.index].value).toFixed(2) : '0.00'}</span>
          </div>
          <button 
            on:click={() => {
              if (dragKeyframe) {
                dispatch('deletekeyframe', { property: dragKeyframe.property, index: dragKeyframe.index });
                dragKeyframe = null;
              }
            }}
            class="p-2 text-zinc-500 hover:text-red-400 transition-colors"
          >
            <Trash2 size={14} />
          </button>
        </div>
      {/if}
    </div>

    <canvas
      bind:this={canvas}
      width={containerWidth - 192}
      height={containerHeight - 40}
      on:mousedown={handleMouseDown}
      on:mousemove={handleMouseMove}
      on:mouseup={handleMouseUp}
      on:mouseleave={handleMouseUp}
      on:wheel|passive={handleWheel}
      class="flex-1 cursor-crosshair"
    ></canvas>

    <!-- Bottom Legend / Info -->
    <div class="h-8 bg-[#111] border-t border-zinc-800 flex items-center px-4 justify-between shrink-0">
      <div class="text-[10px] text-zinc-600 font-mono italic">
        Select a property on the left to view and edit its animation curve.
      </div>
    </div>
  </div>
</div>

<style>
  canvas {
    image-rendering: pixelated;
  }
  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #333;
    border-radius: 10px;
  }
</style>
