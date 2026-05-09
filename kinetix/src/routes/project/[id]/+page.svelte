<script lang="ts">
  import { page } from '$app/stores';
  import { onDestroy, onMount } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { Play, Pause, Plus, FileVideo, X, Sparkles } from 'lucide-svelte';
  import Sidebar from '../../../components/Sidebar.svelte';
  import VideoPreview from '../../../components/VideoPreview.svelte';
  import GraphEditor from '../../../components/GraphEditor.svelte';
  import Timeline from '../../../components/Timeline.svelte';
  import TitleBar from '../../../components/TitleBar.svelte';
  import PropertiesPanel from '../../../components/PropertiesPanel.svelte';

  // --- PROJECT STATE ---
  let mediaFiles: any[] = [];
  let timelineTracks: any[][] = [[], [], []];
  let projectDuration = 60;
  
  // --- CLIP PROPERTIES ---
  let clipProperties: Record<string, any> = {}; 

  // --- PLAYBACK STATE ---
  let currentTime = 0;
  let isPlaying = false;
  let videoSrc = '';
  let videoType = '';
  let videoDuration = 0;
  let seekRequest = -1;
  let playbackFrame: number;
  let lastFrameTime: number;

  function startPlayback() {
    if (isPlaying) return;
    console.log('Starting playback at', currentTime);
    isPlaying = true;
    lastFrameTime = performance.now();
    playbackFrame = requestAnimationFrame(renderStep);
  }

  function stopPlayback() {
    console.log('Stopping playback at', currentTime);
    isPlaying = false;
    if (playbackFrame) cancelAnimationFrame(playbackFrame);
  }

  function renderStep(timestamp: number) {
    if (!isPlaying) return;

    const delta = (timestamp - lastFrameTime) / 1000;
    lastFrameTime = timestamp;

    // Drive currentTime manually to ensure project-wide consistency
    currentTime += delta;

    if (currentTime >= projectDuration) {
      stopPlayback();
      currentTime = 0;
    }

    syncPreviewToTime();
    playbackFrame = requestAnimationFrame(renderStep);
  }

  // --- UI STATE ---
  let selectedClip: any = null;
  let selectedMedia: any = null; // New state for single media preview
  let sidebarTab: 'media' | 'filters' = 'media';
  let sidebarWidth = 280;
  let propertiesWidth = 300;
  let timelineHeight = 400;
  let resizingPanel: 'sidebar' | 'properties' | 'timeline' | null = null;
  let activeClips: any[] = [];
  let activeClipId: string | null = null;
  let isEyedropperActive = false;
  let eyedropperTarget: { effectId: string; paramId: string } | null = null;
  let previewMode: 'preview' | 'graph' = 'preview';

  // --- Sequence Settings ---
  let sequenceSettings = {
    width: 1920,
    height: 1080,
    fps: 30
  };
  let showSequenceSettings = false;

  function openSequenceSettings() {
    showSequenceSettings = true;
  }
  
  function closeSequenceSettings() {
    showSequenceSettings = false;
  }


  // Sync current properties for the selected clip
  $: currentProperties = selectedClip ? (clipProperties[selectedClip.id] || null) : null;

  // Sync preview when time, tracks, or properties change
  $: {
    if (currentTime !== undefined && timelineTracks && clipProperties) {
      syncPreviewToTime();
    }
  }

  // --- ACTIONS ---

  function addMedia(name: string, src: string, dur: number, type: string, thumbnail?: string) {
    console.log('addMedia:', name, type, dur, thumbnail ? 'with thumbnail' : 'no thumbnail');
    
    const id = crypto.randomUUID();
    const finalDur = dur || 0.1;
    const newFile = { id, name, src, type, duration: finalDur, thumbnail };
    
    // 1. ALWAYS add to Media Pool
    mediaFiles.push(newFile);
    mediaFiles = mediaFiles; // Trigger Svelte reactivity
    console.log('Media pool now has', mediaFiles.length, 'items');

    // 3. Add to Timeline
    // For video/image: find first empty video track or use track 0
    // For audio: use track 1
    let trackIdx: number;
    if (type.startsWith('video') || type.startsWith('image') || type.startsWith('text')) {
      // Find a video track with space (tracks 0, 1, 2 are video tracks)
      trackIdx = 0;
      for (let i = 0; i < 3; i++) {
        if (timelineTracks[i].length === 0) {
          trackIdx = i;
          break;
        }
      }
    } else {
      trackIdx = 1; // Audio track
    }
    
    const newClip = { 
      id: crypto.randomUUID(), 
      name, 
      src, 
      type, 
      duration: finalDur, 
      startTime: 0 // All clips start at time 0 for now
    };
    timelineTracks[trackIdx].push(newClip);
    timelineTracks = [...timelineTracks];
    
    // 2. Initialize properties for THIS CLIP (not media)
    clipProperties[newClip.id] = {
      transform: { x: 0, y: 0, scale: 1, scaleX: 1, scaleY: 1, scaleLocked: true, rotation: 0, opacity: 1, keyframes: {} },
      effects: [],
      transition: { type: 'none', duration: 0.5 }
    };
    
    if (type.startsWith('text')) {
      clipProperties[newClip.id].text = {
        content: 'Your text',
        fontSize: 64,
        fontFamily: 'Arial',
        color: '#ffffff',
        align: 'center',
        weight: 700,
        type: 'normal',
        strokeWidth: 0,
        strokeColor: '#000000',
        width: 400
      };
    }
    
    // 4. Update Preview
    videoSrc = src;
    videoType = type;
    videoDuration = finalDur;
    selectedClip = newClip;
    activeClipId = newClip.id;
    currentTime = newClip.startTime;
    seekRequest = 0;
    
    updateProjectDuration();
  }

  function addTextClip() {
    const duration = 5;
    const newClip = {
      id: crypto.randomUUID(),
      name: 'Text',
      src: '',
      type: 'text/plain',
      duration,
      startTime: currentTime
    };

    timelineTracks[0].push(newClip);
    timelineTracks = [...timelineTracks];

    clipProperties[newClip.id] = {
      transform: { x: 0, y: 0, scale: 1, scaleX: 1, scaleY: 1, scaleLocked: true, rotation: 0, opacity: 1 },
      effects: [],
      text: {
        content: 'Your text',
        fontSize: 64,
        color: '#ffffff',
        weight: 700,
        fontFamily: 'Arial, sans-serif',
        align: 'center',
        width: 400,
        strokeColor: '#000000',
        strokeWidth: 2
      }
    };

    selectedMedia = null;
    selectedClip = newClip;
    activeClipId = newClip.id;
    videoSrc = '';
    videoType = 'text/plain';
    videoDuration = duration;
    seekRequest = -1;

    updateProjectDuration();
  }

  function handleClipSelect(clip: any) {
    console.log('handleClipSelect called:', clip?.name, clip?.id);
    if (!clip) {
      selectedClip = null;
      return;
    }
    selectedClip = clip;
    
    // Ensure properties exist for this clip
    if (!clipProperties[clip.id]) {
      clipProperties[clip.id] = {
        transform: { x: 0, y: 0, scale: 1, scaleX: 1, scaleY: 1, scaleLocked: true, rotation: 0, opacity: 1 },
        effects: []
      };
    }
    
    // Syncing to time will update activeClipId and videoSrc if needed
    currentTime = clip.startTime;
    seekRequest = 0;
  }

  function handleMediaSelect(media: any) {
    selectedMedia = media;
    selectedClip = null;
    activeClipId = null;
    
    videoSrc = media.src;
    videoType = media.type;
    videoDuration = media.duration;
    seekRequest = 0;
    currentTime = 0;
  }

  function goBackToTimeline() {
    selectedMedia = null;
    syncPreviewToTime();
  }

  function handleClipDelete(e: CustomEvent) {
    const { clipId } = e.detail;
    for (let i = 0; i < timelineTracks.length; i++) {
      timelineTracks[i] = timelineTracks[i].filter(c => c.id !== clipId);
    }
    timelineTracks = [...timelineTracks];
    
    if (selectedClip?.id === clipId) {
      selectedClip = null;
      activeClipId = null;
    }
    
    updateProjectDuration();
    syncPreviewToTime(); // Immediately clear preview if the clip at current time was deleted
  }

  function handleClipCut(e: CustomEvent) {
    const { clipId, cutTime } = e.detail;
    for (let i = 0; i < timelineTracks.length; i++) {
      const idx = timelineTracks[i].findIndex(c => c.id === clipId);
      if (idx !== -1) {
        const clip = timelineTracks[i][idx];
        const localCutTime = cutTime - clip.startTime;
        
        if (localCutTime > 0.1 && localCutTime < clip.duration - 0.1) {
          const originalDuration = clip.duration;
          
          clip.duration = localCutTime;
          
          const newClip = {
            ...JSON.parse(JSON.stringify(clip)),
            id: crypto.randomUUID(),
            startTime: clip.startTime + localCutTime,
            duration: originalDuration - localCutTime
          };
          
          if (clipProperties[clipId]) {
            clipProperties[newClip.id] = JSON.parse(JSON.stringify(clipProperties[clipId]));
          }
          
          timelineTracks[i].splice(idx + 1, 0, newClip);
          timelineTracks = [...timelineTracks];
          updateProjectDuration();
        }
        break;
      }
    }
  }

  function handleClipResize(e: CustomEvent) {
    const { id, edge, delta, initialStart, initialDuration } = e.detail;
    for (let i = 0; i < timelineTracks.length; i++) {
      const clipIdx = timelineTracks[i].findIndex(c => c.id === id);
      if (clipIdx !== -1) {
        const clip = timelineTracks[i][clipIdx];
        
        // Find original media duration to prohibit over-resizing (except for images/text)
        const mediaItem = mediaFiles.find(f => f.src === clip.src);
        const isInfinite = clip.type?.startsWith('image') || clip.type?.startsWith('text');
        const maxDuration = isInfinite ? 999999 : (mediaItem ? mediaItem.duration : 999999);

        if (edge === 'left') {
          const newStart = Math.max(0, initialStart + delta);
          const diff = initialStart - newStart;
          const newDuration = Math.min(maxDuration, initialDuration + diff);
          if (newDuration > 0.1) {
            clip.startTime = newStart;
            clip.duration = newDuration;
          }
        } else {
          clip.duration = Math.max(0.1, Math.min(maxDuration, initialDuration + delta));
        }
        timelineTracks = [...timelineTracks];
        updateProjectDuration();
        break;
      }
    }
  }

  function handleStartEyedropper(e: CustomEvent) {
    isEyedropperActive = true;
    eyedropperTarget = { effectId: e.detail.effectId, paramId: e.detail.paramId };
  }

  function handleColorPicked(e: CustomEvent) {
    if (!selectedClip || !eyedropperTarget) return;
    
    const { color } = e.detail;
    const { effectId, paramId } = eyedropperTarget;
    
    handleUpdateEffect(new CustomEvent('updateEffect', {
      detail: { effectId, paramId, value: color }
    }));
    
    isEyedropperActive = false;
    eyedropperTarget = null;
  }

  // Rasterization Modal State
  let showRasterizeModal = false;
  let pendingFilterId: string | null = null;
  let pendingClipId: string | null = null;

  function handleFilterDrop(e: CustomEvent) {
    const { filterId, clipId } = e.detail;
    
    // Check if this is a text clip and a "heavy" visual effect
    const targetTrackIdx = timelineTracks.findIndex(t => t.some(c => c.id === clipId));
    if (targetTrackIdx === -1) return;
    const targetClip = timelineTracks[targetTrackIdx].find(c => c.id === clipId);
    
    const pixelEffects = ['distortion', 'pixelate', 'rgbsplit', 'chromakey'];
    
    if (targetClip?.type?.startsWith('text') && pixelEffects.includes(filterId)) {
      pendingFilterId = filterId;
      pendingClipId = clipId;
      showRasterizeModal = true;
      return;
    }

    applyFilter(filterId, clipId);
  }

  function confirmRasterize() {
    if (!pendingClipId || !pendingFilterId) return;
    
    // 1. Find the clip
    let clip: any = null;
    let trackIdx = -1;
    let clipIdx = -1;
    
    timelineTracks.forEach((track, tIdx) => {
      const cIdx = track.findIndex(c => c.id === pendingClipId);
      if (cIdx !== -1) {
        clip = track[cIdx];
        trackIdx = tIdx;
        clipIdx = cIdx;
      }
    });

    if (clip) {
      // 2. "Rasterize" the clip
      // In a real app, we'd render the text to a canvas and save as image.
      // For this prototype, we'll mark it as rasterized so the engine treats it as pixels.
      clip.type = 'image/text-raster'; 
      timelineTracks[trackIdx][clipIdx] = { ...clip };
      timelineTracks = [...timelineTracks];
      
      // 3. Apply the filter
      applyFilter(pendingFilterId, pendingClipId);
    }
    
    closeRasterizeModal();
  }

  function closeRasterizeModal() {
    showRasterizeModal = false;
    pendingFilterId = null;
    pendingClipId = null;
  }

  function applyFilter(filterId: string, clipId: string) {
    const definitions: Record<string, any> = {
      brightness: { name: 'Brightness', params: [{ id: 'level', label: 'Intensity', value: 120, min: 0, max: 200, unit: '%' }] },
      contrast: { name: 'Contrast', params: [{ id: 'level', label: 'Intensity', value: 120, min: 0, max: 200, unit: '%' }] },
      saturation: { name: 'Saturation', params: [{ id: 'level', label: 'Intensity', value: 150, min: 0, max: 200, unit: '%' }] },
      sepia: { name: 'Sepia', params: [{ id: 'level', label: 'Intensity', value: 80, min: 0, max: 100, unit: '%' }] },
      vintage: { name: 'Vintage', params: [
        { id: 'sepia', label: 'Sepia', value: 50, min: 0, max: 100, unit: '%' },
        { id: 'contrast', label: 'Contrast', value: 90, min: 0, max: 200, unit: '%' }
      ]},
      cinematic: { name: 'Cinematic', params: [
        { id: 'contrast', label: 'Contrast', value: 120, min: 0, max: 200, unit: '%' },
        { id: 'saturation', label: 'Saturation', value: 80, min: 0, max: 200, unit: '%' }
      ]},
      chromakey: { name: 'Chroma Key', params: [
        { id: 'color', label: 'Key Color', value: '#00ff00', type: 'color' },
        { id: 'similarity', label: 'Similarity', value: 15, min: 0, max: 100, unit: '%' },
        { id: 'smoothness', label: 'Smoothness', value: 10, min: 0, max: 100, unit: '%' },
        { id: 'spill', label: 'Spill Reduction', value: 10, min: 0, max: 100, unit: '%' }
      ]},
      blur: { name: 'Gaussian Blur', params: [{ id: 'radius', label: 'Radius', value: 5, min: 0, max: 50, unit: 'px' }] },
      invert: { name: 'Invert', params: [{ id: 'level', label: 'Intensity', value: 100, min: 0, max: 100, unit: '%' }] },
      distortion: { name: 'Distortion', params: [
        { id: 'amplitude', label: 'Amplitude', value: 20, min: 0, max: 100, unit: 'px' },
        { id: 'frequency', label: 'Frequency', value: 0.1, min: 0, max: 1, step: 0.01, unit: 'Hz' },
        { id: 'speed', label: 'Speed', value: 1, min: 0, max: 5, step: 0.1, unit: 'x' }
      ]},
      rgbsplit: { name: 'RGB Split', params: [
        { id: 'amount', label: 'Amount', value: 6, min: 0, max: 40, unit: 'px' }
      ]},
      pixelate: { name: 'Pixelate', params: [
        { id: 'size', label: 'Pixel Size', value: 4, min: 1, max: 40, unit: 'px' }
      ]},
      vignette: { name: 'Vignette', params: [
        { id: 'strength', label: 'Strength', value: 35, min: 0, max: 100, unit: '%' },
        { id: 'softness', label: 'Softness', value: 55, min: 1, max: 100, unit: '%' }
      ]},
      hueshift: { name: 'Hue Shift', params: [
        { id: 'angle', label: 'Angle', value: 30, min: -180, max: 180, unit: 'deg' }
      ]}
    };

    const def = definitions[filterId];
    if (def && clipProperties[clipId]) {
      const newEffect = {
        id: crypto.randomUUID(),
        type: filterId,
        name: def.name,
        params: JSON.parse(JSON.stringify(def.params))
      };
      clipProperties[clipId].effects = [...clipProperties[clipId].effects, newEffect];
      clipProperties = { ...clipProperties };
    }
  }

  function updateProjectDuration() {
    let max = 5;
    timelineTracks.forEach(track => {
      track.forEach(clip => {
        const end = clip.startTime + clip.duration;
        if (end > max) max = end;
      });
    });
    projectDuration = max;
  }

  function handleUpdateKeyframe(e: any) {
    const { property, index, time, value } = e.detail;
    if (!selectedClip || !clipProperties[selectedClip.id]) return;

    const props = clipProperties[selectedClip.id];
    if (!props.transform.keyframes) props.transform.keyframes = {};
    if (!props.transform.keyframes[property]) props.transform.keyframes[property] = [];

    const kfs = props.transform.keyframes[property];
    kfs[index] = { time, value };
    
    // Sort keyframes by time
    kfs.sort((a: any, b: any) => a.time - b.time);
    
    clipProperties = { ...clipProperties };
  }

  function handleDeleteKeyframe(e: any) {
    const { property, index } = e.detail;
    if (!selectedClip || !clipProperties[selectedClip.id]) return;

    const props = clipProperties[selectedClip.id];
    if (props.transform.keyframes && props.transform.keyframes[property]) {
      props.transform.keyframes[property].splice(index, 1);
      clipProperties = { ...clipProperties };
    }
  }

  function findAllVisualClipsAtTime(time: number) {
    const active: any[] = [];
    // Go in reverse (Track 2, then 1, then 0) so top tracks are first in array?
    // Actually, usually we want bottom track (0) rendered first, so it's under.
    for (let i = 0; i < timelineTracks.length; i++) {
      const track = timelineTracks[i];
      const clip = track.find((c: any) => {
        const isVisual = c.type?.startsWith('video') || c.type?.startsWith('image') || c.type?.startsWith('text');
        return isVisual && time >= c.startTime && time <= c.startTime + c.duration;
      });
      if (clip) {
        active.push({ ...clip, trackIndex: i });
      }
    }
    return active;
  }

  function syncPreviewToTime() {
    const allActive = findAllVisualClipsAtTime(currentTime);
    activeClips = allActive;

    const activeClipAtTime = allActive.length > 0 ? allActive[allActive.length - 1] : null;
    const activeVideoAtTime = [...allActive].reverse().find((c: any) => c.type?.startsWith('video')) || null;

    if (activeClipAtTime) {
      if (activeClipId !== activeClipAtTime.id) {
        activeClipId = activeClipAtTime.id;
      }

      const mediaDriver = activeVideoAtTime || activeClipAtTime;
      if (mediaDriver) {
        videoSrc = mediaDriver.src;
        videoType = mediaDriver.type;
        videoDuration = mediaDriver.duration;
      }

      // Keyframe Interpolation for ALL active clips
      allActive.forEach(clip => {
        const props = clipProperties[clip.id];
        if (props) {
          const relTime = currentTime - clip.startTime;
          
          // Transform Keyframes
          if (props.transform.keyframes) {
            Object.keys(props.transform.keyframes).forEach(prop => {
              const kfs = props.transform.keyframes[prop];
              if (kfs && kfs.length > 0) {
                props.transform[prop] = interpolateKeyframes(kfs, relTime);
              }
            });
          }

          // Effect Keyframes
          if (props.effects) {
            props.effects.forEach((eff: any) => {
              eff.params.forEach((param: any) => {
                if (param.keyframes && param.keyframes.length > 0) {
                  param.value = interpolateKeyframes(param.keyframes, relTime);
                }
              });
            });
          }
        }
      });
      
      clipProperties = { ...clipProperties };
      seekRequest = activeVideoAtTime ? currentTime - activeVideoAtTime.startTime : -1;
    } else {
      activeClipId = null;
      videoSrc = '';
      videoType = '';
      videoDuration = 0;
      seekRequest = -1;
    }
  }

  function interpolateKeyframes(keyframes: any[], time: number) {
    if (keyframes.length === 0) return 0;
    if (time <= keyframes[0].time) return keyframes[0].value;
    if (time >= keyframes[keyframes.length - 1].time) return keyframes[keyframes.length - 1].value;

    for (let i = 0; i < keyframes.length - 1; i++) {
      const k1 = keyframes[i];
      const k2 = keyframes[i + 1];
      if (time >= k1.time && time <= k2.time) {
        const t = (time - k1.time) / (k2.time - k1.time);
        
        // Handle color interpolation
        if (typeof k1.value === 'string' && k1.value.startsWith('#')) {
          return interpolateColor(k1.value, k2.value, t);
        }
        
        return k1.value + (k2.value - k1.value) * t;
      }
    }
    return keyframes[0].value;
  }

  function interpolateColor(c1: string, c2: string, t: number) {
    const r1 = parseInt(c1.substring(1, 3), 16);
    const g1 = parseInt(c1.substring(3, 5), 16);
    const b1 = parseInt(c1.substring(5, 7), 16);
    
    const r2 = parseInt(c2.substring(1, 3), 16);
    const g2 = parseInt(c2.substring(3, 5), 16);
    const b2 = parseInt(c2.substring(5, 7), 16);
    
    const r = Math.round(r1 + (r2 - r1) * t);
    const g = Math.round(g1 + (g2 - g1) * t);
    const b = Math.round(b1 + (b2 - b1) * t);
    
    return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
  }

  function handleSeek(time: number) {
    currentTime = time;
    syncPreviewToTime();
  }

  function handleUpdateTransform(e: CustomEvent) {
    if (!selectedClip) return;
    const { prop, value } = e.detail;
    const props = clipProperties[selectedClip.id];
    props.transform[prop] = value;

    const isScaleLocked = props.transform.scaleLocked !== false;
    if (prop === 'scale') {
      props.transform.scaleX = value;
      props.transform.scaleY = value;
    } else if (prop === 'scaleX') {
      if (isScaleLocked) props.transform.scaleY = value;
    } else if (prop === 'scaleY') {
      if (isScaleLocked) props.transform.scaleX = value;
    } else if (prop === 'scaleLocked' && value) {
      props.transform.scaleY = props.transform.scaleX ?? props.transform.scale ?? 1;
    }

    const sx = props.transform.scaleX ?? props.transform.scale ?? 1;
    const sy = props.transform.scaleY ?? props.transform.scale ?? 1;
    props.transform.scale = (sx + sy) / 2;
    
    // Support keyframing for transforms
    if (prop !== 'scaleLocked') {
      if (!props.transform.keyframes) props.transform.keyframes = {};
      if (props.transform.keyframes[prop] && props.transform.keyframes[prop].length > 0) {
      const relTime = currentTime - selectedClip.startTime;
      const existingIdx = props.transform.keyframes[prop].findIndex((k: any) => Math.abs(k.time - relTime) < 0.1);
      if (existingIdx !== -1) {
        props.transform.keyframes[prop][existingIdx].value = value;
      }
      }
    }
    
    clipProperties = { ...clipProperties };
  }

  function handleUpdateText(e: CustomEvent) {
    if (!selectedClip || !clipProperties[selectedClip.id]?.text) return;
    const { prop, value } = e.detail;
    clipProperties[selectedClip.id].text[prop] = value;
    clipProperties = { ...clipProperties };
  }

  function handleUpdateTransition(e: CustomEvent) {
    const targetId = selectedClip?.id || activeClipId;
    if (!targetId || !clipProperties[targetId]) return;
    const { type, duration, position } = e.detail;
    
    if (!clipProperties[targetId].transitions) {
      clipProperties[targetId].transitions = { 
        in: { type: 'none', duration: 0.5 }, 
        out: { type: 'none', duration: 0.5 } 
      };
    }

    if (position === 'start') {
      clipProperties[targetId].transitions.in = { type, duration: duration || 0.5 };
    } else if (position === 'end') {
      clipProperties[targetId].transitions.out = { type, duration: duration || 0.5 };
    } else {
      // Fallback for direct property panel updates if still used
      clipProperties[targetId].transitions.in = { type, duration: duration || 0.5 };
    }
    
    clipProperties = { ...clipProperties };
  }

  function handleTimelineUpdateTransition(e: CustomEvent) {
    const { clipId, position, duration, type } = e.detail;
    if (!clipProperties[clipId]) return;
    
    if (!clipProperties[clipId].transitions) {
      clipProperties[clipId].transitions = { 
        in: { type: 'none', duration: 0.5 }, 
        out: { type: 'none', duration: 0.5 } 
      };
    }

    const pos = position === 'start' ? 'in' : 'out';
    
    if (type === 'none') {
      clipProperties[clipId].transitions[pos] = { type: 'none', duration: 0.5 };
    } else {
      clipProperties[clipId].transitions[pos].duration = duration;
    }
    
    clipProperties = { ...clipProperties };
  }

  function handleTransitionDrop(e: CustomEvent) {
    const { transitionId, clipId, position } = e.detail;
    handleUpdateTransition(new CustomEvent('updateTransition', { 
      detail: { type: transitionId, duration: 0.5, position } 
    }));
  }

  function handleClipDrag(e: CustomEvent) {
    if (!selectedClip || !clipProperties[selectedClip.id]?.transform) return;
    const { x, y } = e.detail;
    handleUpdateTransform(new CustomEvent('updateTransform', { detail: { prop: 'x', value: x } }));
    handleUpdateTransform(new CustomEvent('updateTransform', { detail: { prop: 'y', value: y } }));
  }

  function handleClipResizePreview(e: CustomEvent) {
    if (!selectedClip || !clipProperties[selectedClip.id]) return;
    const { width, scale, scaleX, scaleY } = e.detail;
    if (width !== undefined) {
      handleUpdateText(new CustomEvent('updateText', { detail: { prop: 'width', value: width } }));
    }
    if (scale !== undefined) {
      handleUpdateTransform(new CustomEvent('updateTransform', { detail: { prop: 'scale', value: scale } }));
    }
    if (scaleX !== undefined) {
      handleUpdateTransform(new CustomEvent('updateTransform', { detail: { prop: 'scaleX', value: scaleX } }));
    }
    if (scaleY !== undefined) {
      handleUpdateTransform(new CustomEvent('updateTransform', { detail: { prop: 'scaleY', value: scaleY } }));
    }
  }

  function handleToggleKeyframe(e: CustomEvent) {
    const targetId = selectedClip?.id || activeClipId;
    if (!targetId) return;
    const { effectId, paramId, prop } = e.detail;
    const props = clipProperties[targetId];
    
    if (prop) {
      // Transform property keyframing
      if (!props.transform.keyframes) props.transform.keyframes = {};
      if (!props.transform.keyframes[prop]) props.transform.keyframes[prop] = [];
      
      const targetClip = selectedClip || timelineTracks.flat().find(c => c.id === targetId);
      if (!targetClip) return;

      const relTime = currentTime - targetClip.startTime;
      const existingIdx = props.transform.keyframes[prop].findIndex((k: any) => Math.abs(k.time - relTime) < 0.1);
      
      if (existingIdx !== -1) {
        props.transform.keyframes[prop].splice(existingIdx, 1);
      } else {
        props.transform.keyframes[prop].push({ time: relTime, value: props.transform[prop] });
        props.transform.keyframes[prop].sort((a: any, b: any) => a.time - b.time);
      }
    } else {
      // Effect parameter keyframing
      const effect = props.effects.find((f: any) => f.id === effectId);
      if (!effect) return;
      const param = effect.params.find((p: any) => p.id === paramId);
      if (!param) return;

      if (!param.keyframes) param.keyframes = [];

      const targetClip = selectedClip || timelineTracks.flat().find(c => c.id === targetId);
      if (!targetClip) return;

      const relTime = currentTime - targetClip.startTime;
      const existingIdx = param.keyframes.findIndex((k: any) => Math.abs(k.time - relTime) < 0.1);

      if (existingIdx !== -1) {
        param.keyframes.splice(existingIdx, 1);
      } else {
        param.keyframes.push({ time: relTime, value: param.value });
        param.keyframes.sort((a: any, b: any) => a.time - b.time);
      }
    }

    clipProperties = { ...clipProperties };
  }

  function handleUpdateEffect(e: CustomEvent) {
    const targetId = selectedClip?.id || activeClipId;
    if (!targetId) return;
    const { effectId, paramId, value } = e.detail;
    const props = clipProperties[targetId];
    const effect = props.effects.find((f: any) => f.id === effectId);
    if (effect) {
      const param = effect.params.find((p: any) => p.id === paramId);
      if (param) {
        param.value = value;
        
        // If keyframes exist, update the one at current time or add/update
        if (param.keyframes && param.keyframes.length > 0) {
          const targetClip = selectedClip || timelineTracks.flat().find(c => c.id === targetId);
          if (targetClip) {
            const relTime = currentTime - targetClip.startTime;
            const existingIdx = param.keyframes.findIndex((k: any) => Math.abs(k.time - relTime) < 0.1);
            if (existingIdx !== -1) {
              param.keyframes[existingIdx].value = value;
            }
          }
        }
      }
    }
    clipProperties = { ...clipProperties };
  }

  function handleDeleteEffect(e: CustomEvent) {
    if (!selectedClip) return;
    const { effectId } = e.detail;
    clipProperties[selectedClip.id].effects = clipProperties[selectedClip.id].effects.filter((f: any) => f.id !== effectId);
    clipProperties = { ...clipProperties };
  }

  function startResize(panel: 'sidebar' | 'properties' | 'timeline', e: MouseEvent) {
    resizingPanel = panel;
    if (panel === 'sidebar' || panel === 'properties') {
      document.body.style.cursor = 'col-resize';
    } else {
      document.body.style.cursor = 'row-resize';
    }
  }

  function handleMouseMove(e: MouseEvent) {
    if (!resizingPanel) return;
    if (resizingPanel === 'sidebar') {
      sidebarWidth = Math.max(150, Math.min(500, e.clientX));
    } else if (resizingPanel === 'properties') {
      propertiesWidth = Math.max(150, Math.min(500, window.innerWidth - e.clientX));
    } else if (resizingPanel === 'timeline') {
      timelineHeight = Math.max(100, Math.min(600, window.innerHeight - e.clientY - 20));
    }
  }

  function handleMouseUp() {
    resizingPanel = null;
    document.body.style.cursor = 'default';
  }

  onDestroy(() => {
    stopPlayback();
  });

  onMount(() => {
    let unlisten: any;
    
    const setup = async () => {
      unlisten = await getCurrentWindow().onDragDropEvent(async (event) => {
        if (event.payload.type === 'drop') {
          const paths = event.payload.paths;
          for (const path of paths) {
            const fileName = path.split('/').pop() || path.split('\\').pop() || 'Unknown';
            const ext = fileName.split('.').pop()?.toLowerCase() || '';
            const isVideo = ['mp4', 'mov', 'avi', 'mkv', 'webm'].includes(ext);
            const isImage = ['jpg', 'jpeg', 'png', 'gif', 'webp'].includes(ext);
            
            if (isVideo || isImage) {
              const mimeType = isVideo ? `video/${ext === 'mov' ? 'quicktime' : ext}` : `image/${ext}`;
              
              // Trigger proxy/thumb generation for dropped files
              if (isVideo) {
                try {
                  console.log('Generating proxy for dropped file:', fileName);
                  const proxyData: any = await invoke('generate_proxy_video', { filePath: path });
                  
                  // Read proxy to blob
                  const bytes: number[] = await invoke('read_video_file', { filePath: proxyData.proxy_path });
                  const blob = new Blob([new Uint8Array(bytes)], { type: 'video/webm' });
                  const src = URL.createObjectURL(blob);
                  const thumbnail = convertFileSrc(proxyData.thumbnail_path);
                  
                  addMedia(fileName, src, proxyData.duration_secs, mimeType, thumbnail);
                } catch (e) {
                  console.error('Failed to proxy dropped file:', e);
                  addMedia(fileName, convertFileSrc(path), 0, mimeType);
                }
              } else {
                addMedia(fileName, convertFileSrc(path), 5, mimeType);
              }
            }
          }
        }
      });
    };

    setup();

    return () => {
      if (unlisten) unlisten();
    };
  });

  $: projectId = $page.params.id;
  $: breadcrumbs = [
    { label: 'Project', onClick: goBackToTimeline },
    ...(selectedMedia ? [{ label: selectedMedia.name }] : []),
    ...(selectedClip && !selectedMedia ? [{ label: selectedClip.name }] : [])
  ];
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<div class="h-screen flex flex-col bg-[#050505] text-zinc-100 overflow-hidden select-none p-2.5 gap-2.5">
  <div class="shrink-0 bg-[#0d0d0d] border border-zinc-800/60 rounded-xl px-1 shadow-lg shadow-black/40">
    <TitleBar 
      showBackButton={true} 
      {breadcrumbs} 
      onOpenSequenceSettings={openSequenceSettings}
    />
  </div>

  {#if showSequenceSettings}
    <div 
      transition:fade={{ duration: 150 }}
      class="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 backdrop-blur-md p-4"
    >
      <div 
        transition:scale={{ duration: 250, start: 0.95 }}
        class="bg-[#111] border border-zinc-800 rounded-2xl w-full max-w-[420px] shadow-[0_0_50px_rgba(0,0,0,0.5)] overflow-hidden"
      >
        <!-- Header (Draggable look) -->
        <div class="h-10 bg-[#181818] border-b border-zinc-800 flex items-center justify-between px-4">
          <div class="flex items-center gap-2">
            <FileVideo size={14} class="text-blue-500" />
            <span class="text-xs font-bold text-zinc-300 uppercase tracking-widest">Sequence Settings</span>
          </div>
          <button on:click={closeSequenceSettings} class="text-zinc-500 hover:text-white transition-colors">
            <X size={16} />
          </button>
        </div>

        <!-- Content -->
        <div class="p-6 space-y-6">
          <div class="space-y-4">
            <div class="space-y-2">
              <span class="text-[10px] font-black text-zinc-500 uppercase tracking-[0.2em]">Video Resolution</span>
              <div class="grid grid-cols-2 gap-3">
                <div class="space-y-1.5">
                  <span class="text-[9px] text-zinc-600 font-bold uppercase">Width</span>
                  <div class="h-9 bg-[#0d0d0d] border border-zinc-800 rounded-lg flex items-center px-3 group focus-within:border-blue-500/50 transition-colors">
                    <input 
                      type="number" 
                      bind:value={sequenceSettings.width} 
                      class="bg-transparent border-none outline-none text-xs text-zinc-200 w-full no-spinner"
                    />
                    <span class="text-[10px] text-zinc-600 font-mono">PX</span>
                  </div>
                </div>
                <div class="space-y-1.5">
                  <span class="text-[9px] text-zinc-600 font-bold uppercase">Height</span>
                  <div class="h-9 bg-[#0d0d0d] border border-zinc-800 rounded-lg flex items-center px-3 group focus-within:border-blue-500/50 transition-colors">
                    <input 
                      type="number" 
                      bind:value={sequenceSettings.height} 
                      class="bg-transparent border-none outline-none text-xs text-zinc-200 w-full no-spinner"
                    />
                    <span class="text-[10px] text-zinc-600 font-mono">PX</span>
                  </div>
                </div>
              </div>
            </div>

            <div class="space-y-2">
              <span class="text-[10px] font-black text-zinc-500 uppercase tracking-[0.2em]">Presets</span>
              <div class="grid grid-cols-3 gap-2">
                <button 
                  on:click={() => { sequenceSettings.width = 1920; sequenceSettings.height = 1080; }}
                  class="py-2 bg-zinc-900 border border-zinc-800 rounded-lg text-[10px] font-bold text-zinc-400 hover:border-blue-500/40 hover:text-blue-400 transition-all"
                >1080P (16:9)</button>
                <button 
                  on:click={() => { sequenceSettings.width = 1080; sequenceSettings.height = 1920; }}
                  class="py-2 bg-zinc-900 border border-zinc-800 rounded-lg text-[10px] font-bold text-zinc-400 hover:border-blue-500/40 hover:text-blue-400 transition-all"
                >SHORTS (9:16)</button>
                <button 
                  on:click={() => { sequenceSettings.width = 1080; sequenceSettings.height = 1080; }}
                  class="py-2 bg-zinc-900 border border-zinc-800 rounded-lg text-[10px] font-bold text-zinc-400 hover:border-blue-500/40 hover:text-blue-400 transition-all"
                >SQUARE (1:1)</button>
              </div>
            </div>

            <div class="space-y-2 pt-2">
              <span class="text-[10px] font-black text-zinc-500 uppercase tracking-[0.2em]">Frame Rate</span>
              <div class="h-9 bg-[#0d0d0d] border border-zinc-800 rounded-lg flex items-center px-3 group focus-within:border-blue-500/50 transition-colors">
                <input 
                  type="number" 
                  bind:value={sequenceSettings.fps} 
                  class="bg-transparent border-none outline-none text-xs text-zinc-200 w-full no-spinner"
                />
                <span class="text-[10px] text-zinc-600 font-mono uppercase">FPS</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="bg-[#181818] p-4 flex gap-3 border-t border-zinc-800">
          <button 
            on:click={closeSequenceSettings}
            class="flex-1 h-9 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-[11px] font-bold text-zinc-300 transition-all active:scale-95"
          >
            DISCARD
          </button>
          <button 
            on:click={closeSequenceSettings}
            class="flex-1 h-9 rounded-xl bg-blue-600 hover:bg-blue-500 text-[11px] font-bold text-white transition-all active:scale-95 shadow-lg shadow-blue-600/20"
          >
            APPLY CHANGES
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if showRasterizeModal}
    <div 
      transition:fade={{ duration: 150 }}
      class="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-[2px] p-4"
    >
      <div 
        transition:scale={{ duration: 200, start: 0.98, opacity: 0 }}
        class="bg-zinc-900 border border-zinc-800 rounded-xl p-5 max-w-[280px] w-full shadow-2xl space-y-4"
      >
        <div class="space-y-1.5 text-center">
          <div class="w-10 h-10 bg-blue-500/10 rounded-full flex items-center justify-center mx-auto mb-2">
            <Sparkles class="text-blue-400" size={18} />
          </div>
          <h3 class="text-sm font-bold text-white tracking-tight">Rasterize Text?</h3>
          <p class="text-[11px] text-zinc-400 leading-normal px-2">
            This effect requires pixels. This will freeze content but unlock advanced visual distortions.
          </p>
        </div>

        <div class="flex gap-2 pt-1">
          <button 
            on:click={closeRasterizeModal}
            class="flex-1 py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-[11px] font-bold transition-all active:scale-95 text-zinc-300"
          >
            CANCEL
          </button>
          <button 
            on:click={confirmRasterize}
            class="flex-1 py-2 rounded-lg bg-blue-600 hover:bg-blue-700 text-[11px] font-bold text-white transition-all active:scale-95"
          >
            RASTERIZE
          </button>
        </div>
      </div>
    </div>
  {/if}

  <div class="flex-1 flex min-h-0 gap-1.5 overflow-hidden">
    <!-- Sidebar -->
    <div style="width: {sidebarWidth}px" class="shrink-0 flex flex-col bg-[#0d0d0d] border border-zinc-800/60 rounded-xl overflow-hidden shadow-2xl shadow-black/60">
      <Sidebar
        bind:activeTab={sidebarTab}
        {mediaFiles}
        onMediaSelect={(e: any) => {
          selectedMedia = e.detail;
          videoSrc = e.detail.src;
          videoType = e.detail.type;
          videoDuration = e.detail.duration;
        }}
        on:import={(e: any) => {
          // Sidebar now uses Tauri dialog + Rust backend to get metadata
          const { name, src, type, duration, filePath, thumbnail } = e.detail;
          console.log('Sidebar import with Rust metadata:', name, type, duration);
          addMedia(name, src, duration, type, thumbnail);
        }}
        on:addtext={addTextClip}
      />
    </div>

    <!-- Resize Handle 1 -->
    <div 
      role="button"
      tabindex="0"
      aria-label="Sidebar resize handle" 
      class="w-1.5 -mx-1 relative z-20 cursor-col-resize group flex items-center justify-center bg-transparent border-none p-0 outline-none focus:ring-0" 
      on:mousedown={(e) => startResize('sidebar', e)}
      on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') startResize('sidebar', e as any); }}
    >
      <div class="w-px h-8 bg-zinc-800 group-hover:bg-blue-500/50 transition-colors"></div>
    </div>

    <!-- Main Preview Area -->
    <div class="flex-1 min-h-0 bg-[#0d0d0d] border border-zinc-800/60 rounded-xl overflow-hidden shadow-2xl shadow-black/60 relative flex flex-col">
      <div class="h-8 shrink-0 bg-[#111] border-b border-zinc-800 flex items-center justify-between px-3 z-30">
        <div class="flex items-center gap-1">
          <button 
            on:click={() => previewMode = 'preview'}
            class="h-6 px-2 rounded-md text-[10px] font-bold transition-all {previewMode === 'preview' ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/20' : 'text-zinc-500 hover:text-zinc-300'}"
          >PREVIEW</button>
          <button 
            on:click={() => previewMode = 'graph'}
            class="h-6 px-2 rounded-md text-[10px] font-bold transition-all {previewMode === 'graph' ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/20' : 'text-zinc-500 hover:text-zinc-300'}"
          >GRAPH EDITOR</button>
        </div>
      </div>

      <div class="flex-1 relative">
        {#if previewMode === 'preview'}
          <VideoPreview
            bind:videoSrc
            bind:videoType
            {activeClips}
            allClipProperties={clipProperties}
            currentTime={currentTime}
            duration={videoDuration}
            {isPlaying}
            {seekRequest}
            {isEyedropperActive}
            selectedClip={selectedClip}
            properties={selectedMedia ? null : (selectedClip ? clipProperties[selectedClip.id] : (activeClipId ? clipProperties[activeClipId] : null))}
            projectWidth={sequenceSettings.width}
            projectHeight={sequenceSettings.height}
            on:toggleplay={() => isPlaying ? stopPlayback() : startPlayback()}
            on:videoimport={(e: any) => addMedia(e.detail.name, e.detail.src, e.detail.duration, e.detail.type)}
            on:clipdrag={handleClipDrag}
            on:clipresize={handleClipResizePreview}
            on:select={(e: any) => handleClipSelect(e.detail)}
            on:deselect={() => handleClipSelect(null)}
            on:durationchange={(e) => {
              if (selectedClip) {
                selectedClip.duration = e.detail.duration;
                timelineTracks = [...timelineTracks];
                updateProjectDuration();
              }
            }}
            on:timeupdate={(e) => {
              if (isPlaying) {
                // If the video is driving time, we update currentTime
                // and then sync everything else.
                currentTime = (activeClips.find(c => c.id === activeClipId)?.startTime || 0) + e.detail.currentTime;
              }
            }}
            on:colorpicked={(e) => {
              if (eyedropperTarget && clipProperties[activeClipId!]) {
                const eff = clipProperties[activeClipId!].effects.find((f: any) => f.id === eyedropperTarget!.effectId);
                if (eff) {
                  const param = eff.params.find((p: any) => p.id === eyedropperTarget!.paramId);
                  if (param) param.value = e.detail.color;
                }
                clipProperties = { ...clipProperties };
                isEyedropperActive = false;
                eyedropperTarget = null;
              }
            }}
          />
        {:else}
          <GraphEditor
            {selectedClip}
            properties={selectedMedia ? null : (selectedClip ? clipProperties[selectedClip.id] : null)}
            currentTime={currentTime}
            duration={videoDuration}
            on:updatekeyframe={handleUpdateKeyframe}
            on:deletekeyframe={handleDeleteKeyframe}
          />
        {/if}
      </div>
    </div>

    <!-- Resize Handle 2 -->
    <div 
      role="button"
      tabindex="0"
      aria-label="Properties resize handle" 
      class="w-1.5 -mx-1 relative z-20 cursor-col-resize group flex items-center justify-center bg-transparent border-none p-0 outline-none focus:ring-0" 
      on:mousedown={(e) => startResize('properties', e)}
      on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') startResize('properties', e as any); }}
    >
      <div class="w-px h-8 bg-zinc-800 group-hover:bg-blue-500/50 transition-colors"></div>
    </div>

    <!-- Properties -->
    <div style="width: {propertiesWidth}px" class="shrink-0 flex flex-col bg-[#0d0d0d] border border-zinc-800/60 rounded-xl overflow-hidden shadow-2xl shadow-black/60">
      <PropertiesPanel 
        {selectedClip} 
        properties={selectedClip ? clipProperties[selectedClip.id] : (activeClipId ? clipProperties[activeClipId] : null)}
        textData={selectedClip ? clipProperties[selectedClip.id].text : (activeClipId ? clipProperties[activeClipId].text : null)}
        effects={selectedClip ? clipProperties[selectedClip.id].effects : (activeClipId ? clipProperties[activeClipId].effects : [])}
        currentTime={currentTime}
        on:updateTransform={handleUpdateTransform}
        on:updateText={handleUpdateText}
        on:updateEffect={handleUpdateEffect}
        on:deleteEffect={handleDeleteEffect}
        on:startEyedropper={handleStartEyedropper}
        on:toggleKeyframe={handleToggleKeyframe}
        on:updateTransition={handleUpdateTransition}
      />
    </div>
  </div>

  <!-- Timeline Resize Handle -->
  <div 
    role="button"
    tabindex="0"
    aria-label="Timeline resize handle" 
    class="h-1.5 -my-1 relative z-20 cursor-row-resize group flex items-center justify-center w-full bg-transparent border-none p-0 outline-none focus:ring-0" 
    on:mousedown={(e) => startResize('timeline', e)}
    on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') startResize('timeline', e as any); }}
  >
    <div class="w-8 h-px bg-zinc-800 group-hover:bg-blue-500/50 transition-colors"></div>
  </div>

  <!-- Bottom Timeline Area -->
  <div style="height: {timelineHeight}px" class="shrink-0 bg-[#0d0d0d] w-full border border-zinc-800/60 rounded-xl overflow-hidden shadow-2xl shadow-black/60">
    <Timeline
      {currentTime}
      {projectDuration}
      tracks={timelineTracks}
      {clipProperties}
      on:seek={(e: any) => handleSeek(e.detail)}
      on:clipselect={(e: any) => {
        selectedMedia = null; // Clear single media preview if we select a clip
        handleClipSelect(e.detail);
      }}
      on:clipcut={handleClipCut}
      on:clipdelete={handleClipDelete}
      on:clipresize={handleClipResize}
      on:filterdrop={handleFilterDrop}
      on:transitiondrop={handleTransitionDrop}
      on:updatetransitionduration={handleTimelineUpdateTransition}
    />
  </div>
</div>

<style>
  :global(body) { margin: 0; padding: 0; }
</style>
