<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { Settings2, Move, Scale, RotateCw, Eye, CircleDot, Trash2, ChevronDown, ChevronRight, Pipette, Search, Check, AlignLeft, AlignCenter, AlignRight, Sparkles, Link, Unlink, Diamond } from 'lucide-svelte';
  import { scale, fade } from 'svelte/transition';
  import { invoke } from '@tauri-apps/api/core';

  export let selectedClip: any = null;
  export let properties: any = null; // Basic transforms
  export let textData: any = null;
  export let effects: any[] = []; // List of applied effect objects

  const dispatch = createEventDispatcher();

  let activeTab = 'transform';
  let expandedEffects: Record<string, boolean> = {};

  let openDropdown: string | null = null;
  let fontSearch = '';
  let systemFonts: { label: string, value: string }[] = [];

  onMount(async () => {
    try {
      const fonts: string[] = await invoke('get_system_fonts');
      systemFonts = [
        { label: 'System Default', value: 'system-ui, sans-serif' },
        ...fonts.map(f => ({ label: f, value: f }))
      ];
    } catch (e) {
      console.error('Failed to load system fonts:', e);
      systemFonts = [
        { label: 'System Default', value: 'system-ui, sans-serif' },
        { label: 'Arial', value: 'Arial, sans-serif' },
        { label: 'Helvetica', value: 'Helvetica, Arial, sans-serif' },
        { label: 'Georgia', value: 'Georgia, serif' },
        { label: 'Times New Roman', value: '"Times New Roman", Times, serif' },
        { label: 'Verdana', value: 'Verdana, Geneva, sans-serif' },
        { label: 'Monospace', value: 'monospace' }
      ];
    }
  });

  const typeOptions = [
    { label: 'Normal', value: 'normal' },
    { label: 'Italic', value: 'italic' },
    { label: 'Oblique', value: 'oblique' }
  ];

  const weightOptions = [
    { label: 'Thin (300)', value: 300 },
    { label: 'Regular (400)', value: 400 },
    { label: 'Medium (500)', value: 500 },
    { label: 'Semi Bold (600)', value: 600 },
    { label: 'Bold (700)', value: 700 },
    { label: 'Extra Bold (800)', value: 800 },
    { label: 'Black (900)', value: 900 }
  ];

  $: filteredFonts = systemFonts.filter(f => 
    f.label.toLowerCase().includes(fontSearch.toLowerCase())
  );

  function toggleDropdown(id: string) {
    if (openDropdown === id) openDropdown = null;
    else {
      openDropdown = id;
      fontSearch = '';
    }
  }

  function closeDropdowns() {
    openDropdown = null;
  }

  const transformFields = [
    { id: 'x', label: 'X Position', min: -1000, max: 1000, step: 1, icon: Move },
    { id: 'y', label: 'Y Position', min: -1000, max: 1000, step: 1, icon: Move },
    { id: 'scaleX', label: 'Scale X', min: 0.1, max: 5, step: 0.1, icon: Scale },
    { id: 'scaleY', label: 'Scale Y', min: 0.1, max: 5, step: 0.1, icon: Scale },
    { id: 'rotation', label: 'Rotation', min: -360, max: 360, step: 1, icon: RotateCw },
    { id: 'opacity', label: 'Opacity', min: 0, max: 1, step: 0.01, icon: Eye },
  ];
  const transformRows = [
    ['scaleX', 'scaleY'],
    ['x', 'y'],
    ['rotation'],
    ['opacity']
  ];

  const tabItems = [
    { id: 'transform', icon: Move, title: 'Transform' },
    { id: 'effects', icon: Sparkles, title: 'Effects' },
    { id: 'transitions', icon: CircleDot, title: 'Transitions' }
  ];

  function updateTransform(prop: string, value: number | boolean) {
    dispatch('updateTransform', { prop, value });
  }

  function getTransformField(id: string) {
    return transformFields.find((field) => field.id === id);
  }

  function getTransformLabel(id: string) {
    if (id === 'scaleX') return 'Width';
    if (id === 'scaleY') return 'Height';
    if (id === 'x') return 'X';
    if (id === 'y') return 'Y';
    if (id === 'rotation') return 'Rotation';
    if (id === 'opacity') return 'Opacity';
    return id;
  }

  function getTransformPrefix(id: string) {
    if (id === 'scaleX') return 'W';
    if (id === 'scaleY') return 'H';
    if (id === 'x') return 'X';
    if (id === 'y') return 'Y';
    if (id === 'rotation') return '↻';
    if (id === 'opacity') return 'O';
    return '';
  }

  function getTransformDefault(id: string) {
    if (id === 'scaleX' || id === 'scaleY') return 1;
    if (id === 'opacity') return 1;
    return 0;
  }

  function updateText(prop: string, value: string | number) {
    dispatch('updateText', { prop, value });
  }

  function updateEffectParam(effectId: string, paramId: string, value: any) {
    dispatch('updateEffect', { effectId, paramId, value });
  }

  function startEyedropper(effectId: string, paramId: string) {
    dispatch('startEyedropper', { effectId, paramId });
  }

  function toggleKeyframe(effectId: string, paramId: string) {
    dispatch('toggleKeyframe', { effectId, paramId });
  }

  function deleteEffect(effectId: string) {
    dispatch('deleteEffect', { effectId });
  }

  function updateTransition(type: string, duration: number, position?: 'start' | 'end') {
    dispatch('updateTransition', { type, duration, position });
  }

  function toggleExpand(id: string) {
    expandedEffects[id] = !expandedEffects[id];
  }

  function clickOutside(node: HTMLElement, onOut: () => void) {
    const handleClick = (event: MouseEvent) => {
      if (node && !node.contains(event.target as Node) && !event.defaultPrevented) {
        onOut();
      }
    };
    document.addEventListener('click', handleClick, true);
    return {
      destroy() {
        document.removeEventListener('click', handleClick, true);
      }
    };
  }

  $: if (effects) {
    effects.forEach(eff => {
      if (expandedEffects[eff.id] === undefined) expandedEffects[eff.id] = true;
    });
  }
</script>

<div class="h-full flex flex-col bg-[#141414] overflow-hidden border-l border-zinc-800">
  <div class="p-3 border-b border-zinc-800 flex items-center justify-between shrink-0 bg-zinc-900/50">
    <div class="flex items-center gap-2">
      <Settings2 size={14} class="text-blue-400" />
      <span class="text-xs font-bold uppercase tracking-wider text-zinc-400">Properties</span>
    </div>
    {#if selectedClip}
      <span class="text-[10px] text-zinc-500 font-medium truncate max-w-[100px]">{selectedClip.name}</span>
    {/if}
  </div>

  {#if selectedClip && properties}
    <div class="flex flex-1 min-h-0">
      <div class="w-12 border-r border-zinc-800 bg-zinc-900/30 flex flex-col items-center py-2 gap-1 shrink-0">
        {#each tabItems as tab}
          <button
            type="button"
            class="w-9 h-9 rounded-lg flex items-center justify-center transition-colors {activeTab === tab.id ? 'bg-blue-600/20 text-blue-400 border border-blue-500/40' : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/70 border border-transparent'}"
            on:click={() => activeTab = tab.id}
            title={tab.title}
            aria-label={tab.title}
          >
            <svelte:component this={tab.icon} size={14} />
          </button>
        {/each}
      </div>

      <div class="flex-1 overflow-y-auto p-3 space-y-4 custom-scrollbar">
      {#if activeTab === 'transform'}
        <div class="space-y-4">
          {#if selectedClip.type?.startsWith('text') && textData}
            <div class="space-y-4 p-3 bg-zinc-900/20 rounded-lg border border-zinc-800/30">
              <div class="space-y-2">
                <div class="flex justify-between items-center">
                  <span class="text-[10px] font-semibold uppercase tracking-tight text-zinc-400">Text Content</span>
                </div>
                <textarea
                  rows="2"
                  value={textData.content}
                  on:input={(e) => updateText('content', e.currentTarget.value)}
                  class="w-full bg-zinc-900 border border-zinc-800 rounded px-2 py-1 text-xs text-zinc-200 outline-none focus:border-blue-500/50"
                ></textarea>
              </div>

              <div class="grid grid-cols-2 gap-3 relative">
                <div class="space-y-1.5 col-span-2">
                  <span class="text-[9px] font-medium text-zinc-500 uppercase tracking-wider">Font Family</span>
                  <div class="relative">
                    <button
                      on:click|stopPropagation={() => toggleDropdown('font')}
                      class="w-full h-7 bg-zinc-900 text-zinc-200 border border-zinc-800 rounded px-2 text-[10px] flex items-center justify-between hover:bg-zinc-800 transition-colors"
                    >
                      <span class="truncate">{systemFonts.find(f => f.value === textData.fontFamily)?.label || textData.fontFamily || 'Select Font'}</span>
                      <ChevronDown size={10} class="text-zinc-500 transition-transform {openDropdown === 'font' ? 'rotate-180' : ''}" />
                    </button>

                    {#if openDropdown === 'font'}
                      <div 
                        use:clickOutside={closeDropdowns}
                        transition:scale={{ duration: 150, start: 0.95 }}
                        class="absolute top-full left-0 right-0 mt-1 bg-zinc-900 border border-zinc-800 rounded shadow-xl z-[60] overflow-hidden flex flex-col"
                        style="max-height: 200px;"
                      >
                        <div class="p-1.5 border-b border-zinc-800 flex items-center gap-1.5 sticky top-0 bg-zinc-900">
                          <Search size={10} class="text-zinc-500" />
                          <input
                            type="text"
                            bind:value={fontSearch}
                            placeholder="Search fonts..."
                            class="bg-transparent border-none outline-none text-[10px] text-zinc-200 w-full placeholder:text-zinc-600"
                            on:click|stopPropagation
                          />
                        </div>
                        <div class="overflow-y-auto custom-scrollbar flex-1">
                          {#each filteredFonts as font}
                            <button
                              on:click={() => { updateText('fontFamily', font.value); closeDropdowns(); }}
                              class="w-full px-2 py-1.5 text-[10px] text-left hover:bg-blue-600/20 hover:text-blue-400 transition-colors flex items-center justify-between group"
                              style="font-family: {font.value}"
                            >
                              <span class="truncate">{font.label}</span>
                              {#if textData.fontFamily === font.value}
                                <Check size={10} class="text-blue-400" />
                              {/if}
                            </button>
                          {/each}
                        </div>
                      </div>
                    {/if}
                  </div>
                </div>

                <div class="space-y-1.5">
                  <span class="text-[9px] font-medium text-zinc-500 uppercase tracking-wider">Style</span>
                  <div class="relative">
                    <button
                      on:click|stopPropagation={() => toggleDropdown('type')}
                      class="w-full h-7 bg-zinc-900 text-zinc-200 border border-zinc-800 rounded px-2 text-[10px] flex items-center justify-between hover:bg-zinc-800 transition-colors"
                    >
                      <span class="truncate uppercase">{textData.type || 'Normal'}</span>
                      <ChevronDown size={10} class="text-zinc-500 transition-transform {openDropdown === 'type' ? 'rotate-180' : ''}" />
                    </button>
                    {#if openDropdown === 'type'}
                      <div use:clickOutside={closeDropdowns} transition:scale={{ duration: 150, start: 0.95 }} class="absolute top-full left-0 right-0 mt-1 bg-zinc-900 border border-zinc-800 rounded shadow-xl z-[60] overflow-hidden">
                        {#each typeOptions as type}
                          <button on:click={() => { updateText('type', type.value); closeDropdowns(); }} class="w-full px-2 py-1.5 text-[10px] text-left hover:bg-blue-600/20 hover:text-blue-400 transition-colors flex items-center justify-between">
                            <span>{type.label}</span>
                            {#if (textData.type || 'normal') === type.value} <Check size={10} class="text-blue-400" /> {/if}
                          </button>
                        {/each}
                      </div>
                    {/if}
                  </div>
                </div>

                <div class="space-y-1.5">
                  <span class="text-[9px] font-medium text-zinc-500 uppercase tracking-wider">Weight</span>
                  <div class="relative">
                    <button
                      on:click|stopPropagation={() => toggleDropdown('weight')}
                      class="w-full h-7 bg-zinc-900 text-zinc-200 border border-zinc-800 rounded px-2 text-[10px] flex items-center justify-between hover:bg-zinc-800 transition-colors"
                    >
                      <span>{textData.weight}</span>
                      <ChevronDown size={10} class="text-zinc-500 transition-transform {openDropdown === 'weight' ? 'rotate-180' : ''}" />
                    </button>
                    {#if openDropdown === 'weight'}
                      <div use:clickOutside={closeDropdowns} transition:scale={{ duration: 150, start: 0.95 }} class="absolute top-full left-0 right-0 mt-1 bg-zinc-900 border border-zinc-800 rounded shadow-xl z-[60] overflow-hidden">
                        <div class="overflow-y-auto custom-scrollbar" style="max-height: 150px;">
                          {#each weightOptions as weight}
                            <button on:click={() => { updateText('weight', weight.value); closeDropdowns(); }} class="w-full px-2 py-1.5 text-[10px] text-left hover:bg-blue-600/20 hover:text-blue-400 transition-colors flex items-center justify-between" style="font-weight: {weight.value}">
                              <span>{weight.label}</span>
                              {#if textData.weight === weight.value} <Check size={10} class="text-blue-400" /> {/if}
                            </button>
                          {/each}
                        </div>
                      </div>
                    {/if}
                  </div>
                </div>
              </div>

              <div class="grid grid-cols-2 gap-3">
                <div class="space-y-1.5">
                  <span class="text-[9px] font-medium text-zinc-500 uppercase tracking-wider">Alignment</span>
                  <div class="flex bg-zinc-900 border border-zinc-800 rounded p-0.5">
                    <button on:click={() => updateText('align', 'left')} class="flex-1 h-6 flex items-center justify-center rounded transition-colors {textData.align === 'left' ? 'bg-zinc-700 text-blue-400' : 'text-zinc-500 hover:text-zinc-300'}"><AlignLeft size={12} /></button>
                    <button on:click={() => updateText('align', 'center')} class="flex-1 h-6 flex items-center justify-center rounded transition-colors {textData.align === 'center' || !textData.align ? 'bg-zinc-700 text-blue-400' : 'text-zinc-500 hover:text-zinc-300'}"><AlignCenter size={12} /></button>
                    <button on:click={() => updateText('align', 'right')} class="flex-1 h-6 flex items-center justify-center rounded transition-colors {textData.align === 'right' ? 'bg-zinc-700 text-blue-400' : 'text-zinc-500 hover:text-zinc-300'}"><AlignRight size={12} /></button>
                  </div>
                </div>
                <div class="space-y-1.5">
                  <span class="text-[9px] font-medium text-zinc-500 uppercase tracking-wider">Fill Color</span>
                  <input type="color" value={textData.color} on:input={(e) => updateText('color', e.currentTarget.value)} class="w-full h-7 bg-zinc-800 rounded border border-zinc-700 cursor-pointer p-0.5" />
                </div>
              </div>
            </div>
          {/if}

          <div class="pt-1">
            <div class="h-9 px-1 border-b border-zinc-800/70 flex items-center justify-between">
              <span class="text-[11px] font-medium text-zinc-200">Transform</span>
              <ChevronDown size={12} class="text-zinc-500" />
            </div>

            <div class="pt-3 space-y-2.5">
              {#each transformRows as row}
                <div class="grid {row[0] === 'scaleX' && row[1] === 'scaleY' ? 'grid-cols-[1fr_auto_1fr]' : (row.length === 2 ? 'grid-cols-2' : 'grid-cols-1')} gap-2 items-stretch">
                  {#each row as fieldId}
                    {@const field = getTransformField(fieldId)}
                    {#if field}
                      <div class="space-y-1">
                        <div class="flex items-center gap-1.5 px-0.5">
                          <Diamond size={8} class="{field.id === 'scaleX' || field.id === 'scaleY' ? 'text-sky-400 fill-current' : 'text-zinc-500'}" />
                          <span class="text-[10px] text-zinc-400">{getTransformLabel(field.id)}</span>
                        </div>

                        <div class="h-8 rounded-xl border border-zinc-800 bg-zinc-900/80 flex items-center gap-1.5 px-2">
                          <span class="text-[10px] text-zinc-500 font-medium">{getTransformPrefix(field.id)}</span>
                          <input
                            type="number"
                            step={field.step}
                            value={properties.transform[field.id] ?? (field.id === 'scaleX' || field.id === 'scaleY' ? (properties.transform.scale ?? 1) : getTransformDefault(field.id))}
                            on:input={(e) => updateTransform(field.id, parseFloat(e.currentTarget.value))}
                            class="no-spinner w-full bg-transparent text-[11px] text-zinc-200 font-mono outline-none"
                          />
                          {#if field.id === 'scaleX' || field.id === 'scaleY'}
                            <button
                              type="button"
                              class="text-zinc-500 hover:text-zinc-300 transition-colors"
                              title={`Reset ${getTransformLabel(field.id)}`}
                              on:click={() => updateTransform(field.id, 1)}
                            >
                              <RotateCw size={10} />
                            </button>
                          {/if}
                        </div>
                      </div>

                      {#if row[0] === 'scaleX' && row[1] === 'scaleY' && field.id === 'scaleX'}
                        <div class="relative self-end mb-0.5 flex items-center justify-center w-8 group/link">
                          <div class="absolute inset-x-[-8px] top-1/2 h-px border-t border-dashed border-zinc-700/50 pointer-events-none"></div>
                          <button
                            type="button"
                            class="relative z-10 h-7 w-7 rounded-lg border border-zinc-800 bg-[#111] text-zinc-500 hover:text-sky-400 hover:border-sky-500/30 transition-all flex items-center justify-center shadow-sm"
                            title={(properties.transform.scaleLocked !== false) ? 'Unlink Width/Height' : 'Link Width/Height'}
                            on:click={() => updateTransform('scaleLocked', properties.transform.scaleLocked === false)}
                          >
                            {#if properties.transform.scaleLocked !== false}
                              <Link size={11} class="text-sky-400" />
                            {:else}
                              <Unlink size={11} />
                            {/if}
                          </button>
                        </div>
                      {/if}
                    {/if}
                  {/each}
                </div>
              {/each}

              <div class="pt-2 border-t border-zinc-800/60"></div>
            </div>
          </div>
        </div>
      {:else if activeTab === 'transitions'}
        <div class="space-y-4">
          <div class="p-2 space-y-4">
            <div class="bg-zinc-800/40 border border-zinc-800 rounded-lg overflow-hidden">
              <div class="flex items-center justify-between p-2 bg-zinc-800/60 border-b border-zinc-800">
                <span class="text-[10px] font-bold text-zinc-300 uppercase tracking-wide flex items-center gap-1.5">
                  <Sparkles size={10} class="text-blue-400" />
                  Entry (In)
                </span>
                {#if properties.transitions?.in?.type && properties.transitions.in.type !== 'none'}
                  <button 
                    on:click={() => updateTransition('none', 0.5, 'start')}
                    class="p-1 text-zinc-600 hover:text-red-400 hover:bg-red-400/10 rounded transition-colors"
                  >
                    <Trash2 size={10} />
                  </button>
                {/if}
              </div>
              <div class="p-3 space-y-3">
                {#if !properties.transitions?.in || properties.transitions.in.type === 'none'}
                  <p class="text-[10px] text-zinc-600 italic">Drag a transition to the start of this clip</p>
                {:else}
                  <div class="space-y-1.5">
                    <div class="flex justify-between items-center">
                      <span class="text-[9px] font-medium text-zinc-500 uppercase">Type: {properties.transitions.in.type}</span>
                    </div>
                    <div class="flex items-center gap-3">
                      <span class="text-[9px] text-zinc-500 uppercase">Duration</span>
                      <input
                        type="range"
                        min="0.1"
                        max="2"
                        step="0.1"
                        value={properties.transitions.in.duration}
                        on:input={(e) => updateTransition(properties.transitions.in.type, parseFloat(e.currentTarget.value), 'start')}
                        class="flex-1 h-1 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-blue-500"
                      />
                      <span class="text-[10px] font-mono text-blue-400 w-8">{properties.transitions.in.duration}s</span>
                    </div>
                  </div>
                {/if}
              </div>
            </div>

            <div class="bg-zinc-800/40 border border-zinc-800 rounded-lg overflow-hidden">
              <div class="flex items-center justify-between p-2 bg-zinc-800/60 border-b border-zinc-800">
                <span class="text-[10px] font-bold text-zinc-300 uppercase tracking-wide flex items-center gap-1.5">
                  <Sparkles size={10} class="text-blue-400" />
                  Exit (Out)
                </span>
                {#if properties.transitions?.out?.type && properties.transitions.out.type !== 'none'}
                  <button 
                    on:click={() => updateTransition('none', 0.5, 'end')}
                    class="p-1 text-zinc-600 hover:text-red-400 hover:bg-red-400/10 rounded transition-colors"
                  >
                    <Trash2 size={10} />
                  </button>
                {/if}
              </div>
              <div class="p-3 space-y-3">
                {#if !properties.transitions?.out || properties.transitions.out.type === 'none'}
                  <p class="text-[10px] text-zinc-600 italic">Drag a transition to the end of this clip</p>
                {:else}
                  <div class="space-y-1.5">
                    <div class="flex justify-between items-center">
                      <span class="text-[9px] font-medium text-zinc-500 uppercase">Type: {properties.transitions.out.type}</span>
                    </div>
                    <div class="flex items-center gap-3">
                      <span class="text-[9px] text-zinc-500 uppercase">Duration</span>
                      <input
                        type="range"
                        min="0.1"
                        max="2"
                        step="0.1"
                        value={properties.transitions.out.duration}
                        on:input={(e) => updateTransition(properties.transitions.out.type, parseFloat(e.currentTarget.value), 'end')}
                        class="flex-1 h-1 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-blue-500"
                      />
                      <span class="text-[10px] font-mono text-blue-400 w-8">{properties.transitions.out.duration}s</span>
                    </div>
                  </div>
                {/if}
              </div>
            </div>
          </div>
        </div>
      {:else}
        <div class="space-y-2">
          {#if effects.length === 0}
            <div class="py-10 text-center space-y-2">
              <p class="text-[10px] text-zinc-600 uppercase font-bold">No effects applied</p>
              <p class="text-[9px] text-zinc-700">Drag effects from the sidebar onto this clip</p>
            </div>
          {/if}
          
          {#each effects as effect (effect.id)}
            <div class="bg-zinc-800/40 border border-zinc-800 rounded-lg overflow-hidden group/card">
              <div class="flex items-center justify-between p-2 bg-zinc-800/60 border-b border-zinc-800">
                <button 
                  class="flex items-center gap-1.5 flex-1 text-left"
                  on:click={() => toggleExpand(effect.id)}
                >
                  {#if expandedEffects[effect.id]}
                    <ChevronDown size={10} class="text-zinc-500" />
                  {:else}
                    <ChevronRight size={10} class="text-zinc-500" />
                  {/if}
                  <span class="text-[10px] font-bold text-zinc-300 uppercase tracking-wide">{effect.name}</span>
                </button>
                <button 
                  on:click={() => deleteEffect(effect.id)}
                  class="p-1 text-zinc-600 hover:text-red-400 hover:bg-red-400/10 rounded transition-colors opacity-0 group-hover/card:opacity-100"
                >
                  <Trash2 size={10} />
                </button>
              </div>

              {#if expandedEffects[effect.id]}
                <div class="p-3 space-y-4 bg-zinc-900/20">
                  {#each effect.params as param}
                    <div class="space-y-1.5">
                      <div class="flex justify-between items-center">
                        <span class="text-[9px] font-medium text-zinc-500 uppercase tracking-wider">{param.label}</span>
                        {#if param.type === 'color'}
                          <div class="flex items-center gap-2">
                            <button
                              on:click={() => toggleKeyframe(effect.id, param.id)}
                              class="p-1 rounded hover:bg-zinc-700 {param.keyframes?.length > 0 ? 'text-blue-400' : 'text-zinc-500'} hover:text-blue-300 transition-colors"
                              title="Toggle Keyframe"
                            >
                              <CircleDot size={10} />
                            </button>
                            <button
                              on:click={() => startEyedropper(effect.id, param.id)}
                              class="p-1 rounded hover:bg-zinc-700 text-zinc-400 hover:text-blue-400 transition-colors"
                              title="Pick color from preview"
                            >
                              <Pipette size={10} />
                            </button>
                            <div 
                              class="w-4 h-4 rounded border border-zinc-700"
                              style="background-color: {param.value}"
                            ></div>
                          </div>
                        {:else}
                          <span class="text-[10px] font-mono text-zinc-400">{param.value}{param.unit || ''}</span>
                          <button
                            on:click={() => toggleKeyframe(effect.id, param.id)}
                            class="p-1 rounded hover:bg-zinc-700 {param.keyframes?.length > 0 ? 'text-blue-400' : 'text-zinc-500'} hover:text-blue-300 transition-colors ml-1"
                            title="Toggle Keyframe"
                          >
                            <CircleDot size={10} />
                          </button>
                        {/if}
                      </div>
                      
                      {#if param.type === 'color'}
                        <input 
                          type="color" 
                          value={param.value}
                          on:input={(e) => updateEffectParam(effect.id, param.id, e.currentTarget.value)}
                          class="w-full h-6 bg-zinc-800 rounded border-none cursor-pointer p-0"
                        />
                      {:else}
                        <input 
                          type="range" 
                          min={param.min} 
                          max={param.max} 
                          step={param.step}
                          value={param.value}
                          on:input={(e) => updateEffectParam(effect.id, param.id, parseFloat(e.currentTarget.value))}
                          class="w-full h-1 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-blue-500"
                        />
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
      </div>
    </div>
  {:else}
    <div class="flex-1 flex items-center justify-center p-6 text-center">
      <div class="space-y-3">
        <div class="w-12 h-12 bg-zinc-900 rounded-2xl flex items-center justify-center mx-auto border border-zinc-800 shadow-inner">
          <Settings2 size={24} class="text-zinc-800" strokeWidth={1.5} />
        </div>
        <p class="text-[10px] text-zinc-600 leading-relaxed uppercase font-bold tracking-widest">Select a clip to view properties</p>
      </div>
    </div>
  {/if}
</div>

<style>
  .no-spinner::-webkit-outer-spin-button,
  .no-spinner::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  .no-spinner {
    -moz-appearance: textfield;
    appearance: textfield;
  }
</style>
