<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Play, Pause, Plus, Video, ChevronDown } from 'lucide-svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { convertFileSrc } from '@tauri-apps/api/core';

  export let videoSrc = '';
  export let videoType = '';
  export let activeClips: any[] = [];
  export let allClipProperties: any = {};
  export let currentTime = 0;
  export let duration = 0;
  export let seekRequest = -1;
  export let isPlaying = false;
  export let selectedClip: any = null;
  export let properties: any = null; // { transform: {}, effects: [] }
  export let isEyedropperActive = false;
  export let projectWidth = 1920;
  export let projectHeight = 1080;

  let zoomMode: 'fit' | '25' | '50' | '75' | '100' | '200' = 'fit';
  let zoomLevel = 1;
  let showZoomMenu = false;

  const zoomOptions = [
    { label: 'Fit', value: 'fit' },
    { label: '25%', value: '25' },
    { label: '50%', value: '50' },
    { label: '75%', value: '75' },
    { label: '100%', value: '100' },
    { label: '200%', value: '200' }
  ];

  function getFitScale() {
    if (!containerWidth || !containerHeight) return 1;
    const padding = 64;
    return Math.min((containerWidth - padding) / projectWidth, (containerHeight - padding) / projectHeight);
  }

  $: {
    if (zoomMode === 'fit') {
      zoomLevel = getFitScale();
    } else {
      zoomLevel = parseInt(zoomMode) / 100;
    }
  }

  // Update zoom when container size changes
  $: if (containerWidth || containerHeight) {
    if (zoomMode === 'fit') {
      zoomLevel = getFitScale();
    }
  }



  const dispatch = createEventDispatcher();
  let video: HTMLVideoElement;
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let lastSeek = -1;
  let currentFilePath = '';
  let isImporting = false;
  let importError = '';
  let playbackError = '';
  let isDraggingClip = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let dragBaseX = 0;
  let dragBaseY = 0;
  let isResizingClip = false;
  let resizeHandle = '';
  let resizeBaseWidth = 0;
  let resizeBaseScaleX = 1;
  let resizeBaseScaleY = 1;
  let resizeStartX = 0;
  let resizeStartY = 0;
  let previewStageWidth = 0;
  let previewStageHeight = 0;
  let containerWidth = 0;
  let containerHeight = 0;
  let isPanning = false;
  let panStartX = 0;
  let panStartY = 0;
  let scrollStartX = 0;
  let scrollStartY = 0;
  let scrollContainer: HTMLDivElement;
  let animationFrame: number;
  let mouseX = 0;
  let mouseY = 0;
  let isMouseOverStage = false;

  function handleMouseMove(e: MouseEvent) {
    if (scrollContainer) {
      const rect = scrollContainer.getBoundingClientRect();
      mouseX = e.clientX - rect.left + scrollContainer.scrollLeft;
      mouseY = e.clientY - rect.top + scrollContainer.scrollTop;
    }
  }

  function handleMouseEnter() {
    isMouseOverStage = true;
  }

  function handleMouseLeave() {
    isMouseOverStage = false;
  }
  function handleWheel(e: WheelEvent) {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      const zoomSpeed = 0.001;
      const delta = -e.deltaY;
      const newZoom = zoomLevel * (1 + delta * zoomSpeed);
      
      // Cap zoom between 10% and 500%
      zoomLevel = Math.max(0.1, Math.min(5, newZoom));
      zoomMode = (Math.round(zoomLevel * 100)).toString() as any;
    }
  }

  function handlePanMouseDown(e: MouseEvent) {
    if (e.button === 1) { // Middle click
      e.preventDefault();
      isPanning = true;
      panStartX = e.clientX;
      panStartY = e.clientY;
      if (scrollContainer) {
        scrollStartX = scrollContainer.scrollLeft;
        scrollStartY = scrollContainer.scrollTop;
      }
      document.body.style.cursor = 'grabbing';
    }
  }

  function handlePanMouseMove(e: MouseEvent) {
    if (isPanning && scrollContainer) {
      const dx = e.clientX - panStartX;
      const dy = e.clientY - panStartY;
      scrollContainer.scrollLeft = scrollStartX - dx;
      scrollContainer.scrollTop = scrollStartY - dy;
    }
  }

  function handlePanMouseUp() {
    if (isPanning) {
      isPanning = false;
      document.body.style.cursor = 'default';
    }
  }

  // Helper to get properties for a specific clip
  function getClipProps(clipId: string) {
    return allClipProperties[clipId] || {
      transform: { x: 0, y: 0, scale: 1, rotation: 0, opacity: 1 },
      effects: [],
      text: { content: 'Your text', fontSize: 64, fontFamily: 'Arial', color: '#ffffff', align: 'center', weight: 700, type: 'normal', strokeWidth: 0, strokeColor: '#000000' }
    };
  }

  function getFilterStyle(clipId: string) {
    const props = getClipProps(clipId);
    if (!props.effects) return '';
    let filterString = '';
    props.effects.forEach((eff: any) => {
      if (eff.type === 'brightness') filterString += `brightness(${eff.params[0].value}%) `;
      if (eff.type === 'contrast') filterString += `contrast(${eff.params[0].value}%) `;
      if (eff.type === 'saturation') filterString += `saturate(${eff.params[0].value}%) `;
      if (eff.type === 'sepia') filterString += `sepia(${eff.params[0].value}%) `;
      if (eff.type === 'hueshift') filterString += `hue-rotate(${eff.params[0].value}deg) `;
      if (eff.type === 'invert') filterString += `invert(${eff.params[0].value}%) `;
      if (eff.type === 'blur') filterString += `blur(${eff.params[0].value}px) `;
      if (eff.type === 'vintage') filterString += `sepia(${eff.params[0].value}%) contrast(${eff.params[1].value}%) `;
      if (eff.type === 'cinematic') filterString += `contrast(${eff.params[0].value}%) saturate(${eff.params[1].value}%) `;
    });
    return filterString;
  }

  function getTransformStyle(clipId: string) {
    const props = getClipProps(clipId);
    const t = props.transform;
    const scaleX = t.scaleX ?? t.scale ?? 1;
    const scaleY = t.scaleY ?? t.scale ?? 1;
    return `translate(${t.x}px, ${t.y}px) scale(${scaleX}, ${scaleY}) rotate(${t.rotation}deg)`;
  }

  function hasAdvancedEffects(clipId: string) {
    const props = getClipProps(clipId);
    return !!props.effects?.find((e: any) => 
      ['chromakey', 'distortion', 'rgbsplit', 'pixelate', 'vignette'].includes(e.type)
    );
  }

  // --- REACTIVE TRIGGERS ---

  $: if (video) {
    if (isPlaying && video.paused) {
      video.play().catch(e => console.error("Playback failed:", e));
    } else if (!isPlaying && !video.paused) {
      video.pause();
    }
  }

  $: if (video && seekRequest !== lastSeek) {
    // If we are playing, only seek if we drift by more than 0.15s (about 4-5 frames)
    // to avoid constant micro-stuttering while keeping sync tight.
    const driftThreshold = isPlaying ? 0.15 : 0.01;
    if (Math.abs(video.currentTime - seekRequest) > driftThreshold) {
      lastSeek = seekRequest;
      video.currentTime = Math.max(0, Math.min(video.duration || 0, seekRequest));
    }
  }

  // Trigger canvas rendering whenever properties, active clips, or time change
  $: {
    if (allClipProperties || activeClips || currentTime) {
      if (typeof window !== 'undefined') {
        renderCanvas();
      }
    }
  }

  // Find the "primary" video for playback syncing
  $: primaryVideoClip = [...activeClips].reverse().find(c => c.type.startsWith('video'));
  $: if (primaryVideoClip && videoSrc !== primaryVideoClip.src) {
    videoSrc = primaryVideoClip.src;
    videoType = primaryVideoClip.type;
  }

  // Update zoom when container size changes
  $: if (containerWidth || containerHeight) {
    if (zoomMode === 'fit') {
      zoomLevel = getFitScale();
    }
  }

  function getTransitionStyle(clip: any) {
    const props = getClipProps(clip.id);
    if (!props.transitions) return '';
    
    const relTime = currentTime - clip.startTime;
    const timeFromEnd = (clip.startTime + clip.duration) - currentTime;
    
    let style = '';

    // Handle Entry Transition (In)
    const transIn = props.transitions.in;
    if (transIn && transIn.type !== 'none' && relTime <= transIn.duration) {
      const progress = Math.min(1, relTime / transIn.duration);
      const ease = progress * (2 - progress); // ease-out
      style += calculateTransitionCSS(transIn.type, ease);
    }

    // Handle Exit Transition (Out)
    const transOut = props.transitions.out;
    if (transOut && transOut.type !== 'none' && timeFromEnd <= transOut.duration && timeFromEnd >= 0) {
      const progress = Math.min(1, timeFromEnd / transOut.duration);
      const ease = progress * (2 - progress); // ease-out (reversed for exit)
      style += calculateTransitionCSS(transOut.type, ease);
    }
    
    return style;
  }

  function calculateTransitionCSS(type: string, ease: number) {
    let css = '';
    if (type === 'fade') {
      css += `opacity: ${ease}; `;
    } else if (type === 'scale') {
      css += `transform: scale(${0.5 + 0.5 * ease}); opacity: ${ease}; `;
    } else if (type === 'slide-up') {
      css += `transform: translateY(${(1 - ease) * 100}%); opacity: ${ease}; `;
    } else if (type === 'slide-down') {
      css += `transform: translateY(${(ease - 1) * 100}%); opacity: ${ease}; `;
    } else if (type === 'slide-left') {
      css += `transform: translateX(${(1 - ease) * 100}%); opacity: ${ease}; `;
    } else if (type === 'slide-right') {
      css += `transform: translateX(${(ease - 1) * 100}%); opacity: ${ease}; `;
    } else if (type === 'rotate') {
      css += `transform: rotate(${(1 - ease) * 45}deg) scale(${ease}); opacity: ${ease}; `;
    }
    return css;
  }

  // Real-time canvas rendering for advanced effects (like Chroma Key)

  
  function hexToRgb(hex: string) {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
      r: parseInt(result[1], 16),
      g: parseInt(result[2], 16),
      b: parseInt(result[3], 16)
    } : { r: 0, g: 255, b: 0 };
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

  function renderCanvas() {
    if (!canvas) return;
    if (!ctx) ctx = canvas.getContext('2d', { willReadFrequently: true })!;
    if (!ctx) return;
    
    // Find which clip this canvas belongs to. 
    // In our simplified model, we'll use the properties of the selected clip 
    // if it has advanced effects, otherwise the top-most active clip with effects.
    const effectClip = activeClips.find(c => c.id === selectedClip?.id && hasAdvancedEffects(c.id)) 
                     || [...activeClips].reverse().find(c => hasAdvancedEffects(c.id));
    
    if (!effectClip) return;
    const clipProps = getClipProps(effectClip.id);
    const clipType = effectClip.type;
    
    const isText = clipType.startsWith('text') || clipType === 'image/text-raster';
    const isImage = clipType.startsWith('image');

    if (isText) {
      // For text clips, we use the untransformed container dimensions
      const parent = canvas.parentElement;
      if (parent) {
        const width = parent.clientWidth || 1920;
        const height = parent.clientHeight || 1080;
        if (canvas.width !== width || canvas.height !== height) {
          canvas.width = width;
          canvas.height = height;
        }
      }

      ctx.clearRect(0, 0, canvas.width, canvas.height);
      
      const text = clipProps?.text;
      if (text) {
        ctx.save();
        
        // Match CSS alignment/flex behavior
        const align = text.align || 'center';
        const fontSize = text.fontSize || 64;
        const weight = text.weight || 700;
        const font = text.fontFamily || 'Arial';
        
        ctx.font = `${weight} ${fontSize}px "${font}"`;
        ctx.fillStyle = text.color || '#ffffff';
        ctx.textAlign = align as CanvasTextAlign;
        ctx.textBaseline = 'middle';
        
        // Multi-line support
        const lines = (text.content || 'Your text').split('\n');
        const lineHeight = fontSize * 1.2;
        const totalHeight = lines.length * lineHeight;
        
        let startY = (canvas.height / 2) - (totalHeight / 2) + (lineHeight / 2);
        let startX = canvas.width / 2;
        
        if (align === 'left') startX = canvas.width * 0.05;
        if (align === 'right') startX = canvas.width * 0.95;

        lines.forEach((line: string, i: number) => {
          // Draw stroke if exists
          if (text.strokeWidth > 0) {
            ctx.strokeStyle = text.strokeColor || '#000000';
            ctx.lineWidth = text.strokeWidth * 2;
            ctx.lineJoin = 'round';
            ctx.strokeText(line, startX, startY + (i * lineHeight));
          }
          ctx.fillText(line, startX, startY + (i * lineHeight));
        });
        
        ctx.restore();
      }
    } else if (isImage) {
      const img = canvas.parentElement?.querySelector('img');
      if (img && img.complete) {
        if (canvas.width !== img.naturalWidth || canvas.height !== img.naturalHeight) {
          canvas.width = img.naturalWidth;
          canvas.height = img.naturalHeight;
        }
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
      } else {
        if (isPlaying) animationFrame = requestAnimationFrame(renderCanvas);
        return;
      }
    } else {
      if (!video) return;
      // Ensure we have valid dimensions
      if (video.videoWidth === 0) {
        if (isPlaying) animationFrame = requestAnimationFrame(renderCanvas);
        return;
      }

      if (canvas.width !== video.videoWidth || canvas.height !== video.videoHeight) {
        canvas.width = video.videoWidth;
        canvas.height = video.videoHeight;
      }

      ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
    }
    
    // Check for advanced effects
    const effects = clipProps?.effects || [];
    
    // 1. Chroma Key (Process first)
    const chromaEffect = effects.find((e: any) => e.type === 'chromakey');
    if (chromaEffect) {
      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const data = imageData.data;
      
      const targetColor = hexToRgb(chromaEffect.params.find((p: any) => p.id === 'color').value);
      const similarity = (chromaEffect.params.find((p: any) => p.id === 'similarity').value / 100);
      const smoothness = (chromaEffect.params.find((p: any) => p.id === 'smoothness').value / 100);
      const spill = (chromaEffect.params.find((p: any) => p.id === 'spill').value / 100);
      
      const simSq = similarity * similarity * 195075;
      const smoothSq = smoothness * smoothness * 195075;
      const spillFactor = spill;

      for (let i = 0; i < data.length; i += 4) {
        const r = data[i];
        const g = data[i + 1];
        const b = data[i + 2];
        
        const dR = r - targetColor.r;
        const dG = g - targetColor.g;
        const dB = b - targetColor.b;
        const dSq = dR * dR + dG * dG + dB * dB;
        
        if (dSq < simSq) {
          data[i + 3] = 0;
        } else if (dSq < simSq + smoothSq) {
          data[i + 3] = ((dSq - simSq) / smoothSq) * 255;
        } else {
          data[i + 3] = 255;
        }

        if (data[i + 3] > 0 && spillFactor > 0) {
          const greenExcess = g - Math.max(r, b);
          if (greenExcess > 0 && targetColor.g > targetColor.r && targetColor.g > targetColor.b) {
            data[i + 1] -= greenExcess * spillFactor;
          }
        }
      }
      ctx.putImageData(imageData, 0, 0);
    }

    // 3. RGB Split
    const rgbSplitEffect = effects.find((e: any) => e.type === 'rgbsplit');
    if (rgbSplitEffect) {
      const amount = rgbSplitEffect.params.find((p: any) => p.id === 'amount').value;
      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const sourceData = new Uint8ClampedArray(imageData.data);
      const targetData = imageData.data;
      const w = canvas.width;
      const h = canvas.height;
      const offset = Math.max(0, Math.floor(amount));

      for (let y = 0; y < h; y++) {
        for (let x = 0; x < w; x++) {
          const idx = (y * w + x) * 4;
          const xr = Math.min(w - 1, x + offset);
          const xb = Math.max(0, x - offset);
          const idxR = (y * w + xr) * 4;
          const idxB = (y * w + xb) * 4;

          targetData[idx] = sourceData[idxR];
          targetData[idx + 1] = sourceData[idx + 1];
          targetData[idx + 2] = sourceData[idxB + 2];
          targetData[idx + 3] = sourceData[idx + 3];
        }
      }
      ctx.putImageData(imageData, 0, 0);
    }

    // 4. Pixelate
    const pixelateEffect = effects.find((e: any) => e.type === 'pixelate');
    if (pixelateEffect) {
      const size = Math.max(1, Math.floor(pixelateEffect.params.find((p: any) => p.id === 'size').value));
      if (size > 1) {
        const w = canvas.width;
        const h = canvas.height;
        const tmpW = Math.max(1, Math.floor(w / size));
        const tmpH = Math.max(1, Math.floor(h / size));

        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(canvas, 0, 0, w, h, 0, 0, tmpW, tmpH);
        ctx.drawImage(canvas, 0, 0, tmpW, tmpH, 0, 0, w, h);
        ctx.imageSmoothingEnabled = true;
      }
    }

    // 5. Vignette
    const vignetteEffect = effects.find((e: any) => e.type === 'vignette');
    if (vignetteEffect) {
      const strength = vignetteEffect.params.find((p: any) => p.id === 'strength').value / 100;
      const softness = Math.max(0.01, vignetteEffect.params.find((p: any) => p.id === 'softness').value / 100);
      const w = canvas.width;
      const h = canvas.height;
      const cx = w / 2;
      const cy = h / 2;
      const maxDist = Math.sqrt(cx * cx + cy * cy);
      const imageData = ctx.getImageData(0, 0, w, h);
      const data = imageData.data;

      for (let y = 0; y < h; y++) {
        for (let x = 0; x < w; x++) {
          const idx = (y * w + x) * 4;
          const dx = x - cx;
          const dy = y - cy;
          const dist = Math.sqrt(dx * dx + dy * dy) / maxDist;
          const edge = Math.max(0, (dist - (1 - softness)) / softness);
          const darken = 1 - edge * strength;

          data[idx] *= darken;
          data[idx + 1] *= darken;
          data[idx + 2] *= darken;
        }
      }
      ctx.putImageData(imageData, 0, 0);
    }

    // 2. Distortion Effect (Wave)
    const distortionEffect = effects.find((e: any) => e.type === 'distortion');
    if (distortionEffect) {
      const amplitude = distortionEffect.params.find((p: any) => p.id === 'amplitude').value;
      const frequency = distortionEffect.params.find((p: any) => p.id === 'frequency').value;
      const speed = distortionEffect.params.find((p: any) => p.id === 'speed').value;
      
      const time = performance.now() / 1000 * speed;
      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const sourceData = new Uint8ClampedArray(imageData.data);
      const targetData = imageData.data;
      
      const w = canvas.width;
      const h = canvas.height;

      for (let y = 0; y < h; y++) {
        // Calculate offset based on sine wave
        const xOffset = Math.sin(y * frequency + time) * amplitude;
        
        for (let x = 0; x < w; x++) {
          const sourceX = Math.floor(x + xOffset);
          
          if (sourceX >= 0 && sourceX < w) {
            const sourceIdx = (y * w + sourceX) * 4;
            const targetIdx = (y * w + x) * 4;
            
            targetData[targetIdx] = sourceData[sourceIdx];
            targetData[targetIdx + 1] = sourceData[sourceIdx + 1];
            targetData[targetIdx + 2] = sourceData[sourceIdx + 2];
            targetData[targetIdx + 3] = sourceData[sourceIdx + 3];
          } else {
            // Out of bounds - make transparent or black
            const targetIdx = (y * w + x) * 4;
            targetData[targetIdx + 3] = 0;
          }
        }
      }
      ctx.putImageData(imageData, 0, 0);
    }

    if (isPlaying) {
      animationFrame = requestAnimationFrame(renderCanvas);
    }
  }

  // Trigger render when properties change (for real-time slider updates while paused)
  $: if (properties && !isPlaying && canvas) {
    if (video || videoType.startsWith('text') || videoType === 'image/text-raster') {
      renderCanvas();
    }
  }

  // Reactive loop control - ONLY depends on video and isPlaying
  $: {
    const hasSource = video || videoType.startsWith('text') || videoType === 'image/text-raster';
    if (hasSource && isPlaying) {
      cancelAnimationFrame(animationFrame);
      renderCanvas();
    } else if (hasSource && !isPlaying) {
      cancelAnimationFrame(animationFrame);
      if (video) video.pause();
    }
  }

  $: if (!isPlaying && canvas && seekRequest !== lastSeek) {
    // Render single frame on seek
    setTimeout(() => {
      renderCanvas();
    }, 10);
  }

  function onLoaded() { 
    if (!video) return;
    duration = video.duration;
    if (video.currentTime === 0) {
      video.currentTime = 0.001;
    }
    
    // Initialize canvas context
    ctx = canvas.getContext('2d', { willReadFrequently: true })!;
    renderCanvas();
    
    dispatch('durationchange', { duration });
  }

  let currentFileName = '';
  // Fallback: Get video duration using browser's built-in parser
  function getDurationFromBrowser(videoSrc: string): Promise<number> {
    return new Promise((resolve) => {
      const v = document.createElement('video');
      v.preload = 'metadata';
      
      v.onloadedmetadata = () => {
        console.log('VideoPreview browser fallback duration:', v.duration);
        resolve(v.duration || 0);
      };
      
      v.onerror = () => {
        console.error('VideoPreview browser fallback failed');
        resolve(0);
      };
      
      v.src = videoSrc;
    });
  }
  
  // Handle file picker using Tauri dialog for reliable file access
  async function onImportClick() {
    isImporting = true;
    importError = '';
    playbackError = '';

    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: 'Video', extensions: ['mp4', 'mov', 'avi', 'mkv', 'webm'] },
          { name: 'Image', extensions: ['jpg', 'jpeg', 'png', 'gif', 'webp'] },
          { name: 'All Files', extensions: ['*'] }
        ]
      });
      
      if (!selected || typeof selected !== 'string') {
        return;
      }
      
      currentFilePath = selected;
      currentFileName = selected.split('/').pop() || selected.split('\\').pop() || 'Unknown';
      
      // Guess mime type from extension
      const ext = currentFileName.split('.').pop()?.toLowerCase() || '';
      const extToMime: Record<string, string> = {
        'mp4': 'video/mp4', 'mov': 'video/quicktime', 'avi': 'video/x-msvideo',
        'mkv': 'video/x-matroska', 'webm': 'video/webm',
        'jpg': 'image/jpeg', 'jpeg': 'image/jpeg', 'png': 'image/png',
        'gif': 'image/gif', 'webp': 'image/webp'
      };
      videoType = extToMime[ext] || 'video/mp4';
      
      let finalDuration = videoType.startsWith('image') ? 5 : 0;
      
      if (videoType.startsWith('image')) {
        videoSrc = convertFileSrc(currentFilePath);
      } else {
        // For videos, generate proxy
        try {
          console.log('Generating proxy for preview...');
          const proxyData: any = await invoke('generate_proxy_video', { filePath: currentFilePath });
          
          // CRITICAL FIX: Read proxy as blob to bypass asset:// protocol issues on Linux
          console.log('Reading proxy into memory for reliable playback...');
          const bytes: number[] = await invoke('read_video_file', { filePath: proxyData.proxy_path });
          const blob = new Blob([new Uint8Array(bytes)], { type: 'video/webm' });
          videoSrc = URL.createObjectURL(blob);
          console.log('Created Blob URL for proxy:', videoSrc);

          if (proxyData.duration_secs > 0) {
            duration = proxyData.duration_secs;
            finalDuration = proxyData.duration_secs;
          }
        } catch (e) {
          console.error('Proxy generation/loading failed in preview:', e);
          videoSrc = convertFileSrc(currentFilePath);
        }
      }
      
      // For videos, get metadata from Rust backend first if duration is still 0
      if (videoType.startsWith('video') && finalDuration === 0) {
        try {
          const meta: any = await invoke('get_video_metadata', { filePath: currentFilePath });
          if (meta.duration_secs > 0) {
            finalDuration = meta.duration_secs;
            duration = meta.duration_secs;
          } else {
            const browserDur = await getDurationFromBrowser(videoSrc);
            if (browserDur > 0) {
              finalDuration = browserDur;
              duration = browserDur;
            }
          }
        } catch (e) {
          const browserDur = await getDurationFromBrowser(videoSrc);
          if (browserDur > 0) {
            finalDuration = browserDur;
            duration = browserDur;
          }
        }
      }
      
      // Dispatch with the (possibly updated) duration
      dispatch('videoimport', { 
        name: currentFileName, 
        src: videoSrc, 
        type: videoType, 
        duration: finalDuration,
        filePath: currentFilePath
      });
    } catch (error) {
      importError = 'Failed to import media. Please try another file.';
      console.error('Failed to open file dialog:', error);
    } finally {
      isImporting = false;
    }
  }

  function onTimeUpdate() {
    dispatch('timeupdate', { currentTime: video.currentTime });
  }

  function onBackgroundClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (target?.closest('[data-preview-clip="true"]')) return;
    dispatch('deselect');
  }

  function onCanvasClick(event: MouseEvent) {
    if (!isEyedropperActive || !canvas || !ctx) return;
    
    const rect = canvas.getBoundingClientRect();
    // Map mouse coordinates to internal canvas resolution
    const x = ((event.clientX - rect.left) / rect.width) * canvas.width;
    const y = ((event.clientY - rect.top) / rect.height) * canvas.height;
    
    const pixel = ctx.getImageData(x, y, 1, 1).data;
    const hex = `#${pixel[0].toString(16).padStart(2, '0')}${pixel[1].toString(16).padStart(2, '0')}${pixel[2].toString(16).padStart(2, '0')}`;
    
    dispatch('colorpicked', { color: hex });
  }

  function onClipMouseDown(event: MouseEvent) {
    if (!selectedClip || !properties?.transform) return;
    event.preventDefault();
    event.stopPropagation();

    isDraggingClip = true;
    dragStartX = event.clientX;
    dragStartY = event.clientY;
    dragBaseX = properties.transform.x || 0;
    dragBaseY = properties.transform.y || 0;
  }

  function onHandleMouseDown(event: MouseEvent, handle: string) {
    if (!selectedClip) return;
    event.preventDefault();
    event.stopPropagation();

    isResizingClip = true;
    resizeHandle = handle;
    resizeStartX = event.clientX;
    resizeStartY = event.clientY;
    
    if (selectedClip.type?.startsWith('text')) {
      resizeBaseWidth = properties?.text?.width || 400;
    } else {
      resizeBaseScaleX = properties?.transform?.scaleX ?? properties?.transform?.scale ?? 1;
      resizeBaseScaleY = properties?.transform?.scaleY ?? properties?.transform?.scale ?? 1;
    }
  }

  function onWindowMouseMove(event: MouseEvent) {
    handlePanMouseMove(event);
    if (isDraggingClip) {
      const nextX = dragBaseX + (event.clientX - dragStartX);
      const nextY = dragBaseY + (event.clientY - dragStartY);
      dispatch('clipdrag', { x: nextX, y: nextY });
    } else if (isResizingClip) {
      const deltaX = event.clientX - resizeStartX;
      const deltaY = event.clientY - resizeStartY;

      if (selectedClip?.type?.startsWith('text')) {
        let nextWidth = resizeBaseWidth;
        if (resizeHandle === 'e' || resizeHandle.includes('e')) {
          nextWidth = resizeBaseWidth + deltaX * 2;
        } else if (resizeHandle === 'w' || resizeHandle.includes('w')) {
          nextWidth = resizeBaseWidth - deltaX * 2;
        }
        dispatch('clipresize', { width: Math.max(50, nextWidth) });
      } else {
        const isCorner = ['nw', 'ne', 'sw', 'se'].includes(resizeHandle);
        const isScaleLocked = properties?.transform?.scaleLocked !== false;
        const scaleFactor = 0.015;

        if (isCorner) {
          const xDirection = resizeHandle.includes('e') ? 1 : -1;
          const yDirection = resizeHandle.includes('s') ? 1 : -1;
          const cornerDelta = ((deltaX * xDirection) + (deltaY * yDirection)) / 2;
          const nextScaleX = Math.max(0.1, resizeBaseScaleX + cornerDelta * scaleFactor);
          const nextScaleY = Math.max(0.1, resizeBaseScaleY + cornerDelta * scaleFactor);
          dispatch('clipresize', { scaleX: nextScaleX, scaleY: nextScaleY });
        } else {
          const xDirection = resizeHandle.includes('e') ? 1 : resizeHandle.includes('w') ? -1 : 0;
          const yDirection = resizeHandle.includes('s') ? 1 : resizeHandle.includes('n') ? -1 : 0;
          const nextScaleX = Math.max(0.1, resizeBaseScaleX + (deltaX * xDirection) * scaleFactor);
          const nextScaleY = Math.max(0.1, resizeBaseScaleY + (deltaY * yDirection) * scaleFactor);
          if (isScaleLocked) {
            const uniformDelta = xDirection !== 0 ? (deltaX * xDirection) : (deltaY * yDirection);
            const uniformScale = Math.max(0.1, ((resizeBaseScaleX + resizeBaseScaleY) / 2) + uniformDelta * scaleFactor);
            dispatch('clipresize', { scaleX: uniformScale, scaleY: uniformScale });
            return;
          }
          dispatch('clipresize', { scaleX: nextScaleX, scaleY: nextScaleY });
        }
      }
    }
  }

  function onWindowMouseUp() {
    handlePanMouseUp();
    isDraggingClip = false;
    isResizingClip = false;
    resizeHandle = '';
  }
</script>

<svelte:window on:mousemove={onWindowMouseMove} on:mouseup={onWindowMouseUp} />

<div class="h-full flex flex-col bg-[#0d0d0d]">
  <div 
    bind:this={scrollContainer}
    role="application"
    aria-label="Preview scroll container"
    class="flex-1 overflow-auto relative flex items-center justify-center p-8 no-scrollbar outline-none cursor-none" 
    bind:clientWidth={containerWidth} 
    bind:clientHeight={containerHeight}
    on:wheel|passive={handleWheel}
    on:mousedown={handlePanMouseDown}
    on:mousemove={handleMouseMove}
    on:mouseenter={handleMouseEnter}
    on:mouseleave={handleMouseLeave}
  >
    <!-- Custom Cursor -->
    {#if isMouseOverStage && !isPanning}
      <div 
        class="absolute pointer-events-none z-[100] mix-blend-difference flex items-center justify-center"
        style="left: {mouseX}px; top: {mouseY}px; transform: translate(-50%, -50%); width: 24px; height: 24px;"
      >
        <div class="w-1.5 h-1.5 bg-white rounded-full"></div>
        <div class="absolute inset-0 border border-white/40 rounded-full scale-100"></div>
      </div>
    {/if}
    <div 
      class="relative bg-black shadow-[0_0_100px_rgba(0,0,0,0.5)] overflow-hidden checkerboard shrink-0"
      style="width: {projectWidth}px; height: {projectHeight}px; transform: scale({zoomLevel}); transform-origin: center;"
      bind:clientWidth={previewStageWidth}
      bind:clientHeight={previewStageHeight}
    >
      {#if activeClips.length > 0}
        <div
          role="region"
          aria-label="Preview stage"
          class="relative w-full h-full overflow-hidden outline-none"
          on:mousedown={onBackgroundClick}
          on:keydown={(e) => { if (e.key === 'Escape') dispatch('deselect'); }}
        >
        {#each activeClips as clip (clip.id)}
          {@const props = getClipProps(clip.id)}
          {@const t = props.transform}
          {@const sx = Math.max(0.01, t.scaleX ?? t.scale ?? 1)}
          {@const sy = Math.max(0.01, t.scaleY ?? t.scale ?? 1)}
          <div 
            class="absolute inset-0 flex items-center justify-center pointer-events-none"
            style="
              z-index: {clip.trackIndex};
              filter: {getFilterStyle(clip.id)};
              transform: {getTransformStyle(clip.id)};
              opacity: {getClipProps(clip.id).transform.opacity};
              transition: filter 0.1s ease-out, opacity 0.1s ease-out;
              {getTransitionStyle(clip)}
            "
          >
            <div class="relative pointer-events-auto">
              {#if clip.type.startsWith('text') || clip.type === 'image/text-raster'}
                <div
                  role="presentation"
                  data-preview-clip="true"
                  class="relative flex flex-col whitespace-pre-wrap break-words select-none {selectedClip?.id === clip.id ? 'cursor-move' : 'cursor-pointer pointer-events-auto'}"
                  style="
                    outline: {selectedClip?.id === clip.id ? '2px solid white' : 'none'};
                    outline-offset: -1px;
                    color: {props.text?.color || '#ffffff'};
                    font-size: {props.text?.fontSize || 64}px;
                    font-weight: {props.text?.weight || 700};
                    font-family: '{props.text?.fontFamily || 'Arial, sans-serif'}';
                    font-style: {props.text?.type || 'normal'};
                    text-align: {props.text?.align || 'center'};
                    align-items: {props.text?.align === 'left' ? 'flex-start' : props.text?.align === 'right' ? 'flex-end' : 'center'};
                    width: {props.text?.width ? props.text.width + 'px' : 'fit-content'};
                    min-width: 50px;
                    max-width: 95%;
                    box-sizing: border-box;
                    -webkit-text-stroke: {props.text?.strokeWidth || 2}px {props.text?.strokeColor || '#000000'};
                    text-shadow: 0 2px 16px rgba(0,0,0,0.35);
                    visibility: {hasAdvancedEffects(clip.id) ? 'hidden' : 'visible'};
                  "
                  on:mousedown={(e) => {
                    if (selectedClip?.id !== clip.id) {
                      dispatch('select', clip);
                    }
                    onClipMouseDown(e);
                  }}
                >
                  {props.text?.content || 'Your text'}

                  {#if selectedClip?.id === clip.id}
                    <!-- Handles -->
                    <div role="presentation" class="handle-invert absolute left-0 top-0 w-3 h-3 cursor-nw-resize" style="transform: translate(-50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'nw')}></div>
                    <div role="presentation" class="handle-invert absolute right-0 top-0 w-3 h-3 cursor-ne-resize" style="transform: translate(50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'ne')}></div>
                    <div role="presentation" class="handle-invert absolute left-0 bottom-0 w-3 h-3 cursor-sw-resize" style="transform: translate(-50%, 50%)" on:mousedown={(e) => onHandleMouseDown(e, 'sw')}></div>
                    <div role="presentation" class="handle-invert absolute right-0 bottom-0 w-3 h-3 cursor-se-resize" style="transform: translate(50%, 50%)" on:mousedown={(e) => onHandleMouseDown(e, 'se')}></div>
                    <div role="presentation" class="handle-invert absolute left-0 top-1/2 w-3 h-3 cursor-w-resize" style="transform: translate(-50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'w')}></div>
                    <div role="presentation" class="handle-invert absolute right-0 top-1/2 w-3 h-3 cursor-e-resize" style="transform: translate(50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'e')}></div>
                  {/if}
                </div>
              {:else if clip.type.startsWith('image')}
                <div 
                  role="presentation"
                  data-preview-clip="true"
                  class="max-h-full max-w-full flex items-center justify-center relative {selectedClip?.id === clip.id ? 'cursor-move' : 'cursor-pointer pointer-events-auto'}"
                  style="
                    outline: {selectedClip?.id === clip.id ? '2px solid white' : 'none'};
                    outline-offset: -1px;
                    visibility: {hasAdvancedEffects(clip.id) ? 'hidden' : 'visible'};
                  "
                  on:mousedown={(e) => {
                    if (selectedClip?.id !== clip.id) {
                      dispatch('select', clip);
                    }
                    onClipMouseDown(e);
                  }}
                >
                  <img src={clip.src} alt="" class="max-h-full max-w-full object-contain pointer-events-none" />
                  
                  {#if selectedClip?.id === clip.id}
                    <!-- Handles -->
                    <div role="presentation" class="handle-invert absolute left-0 top-0 w-3 h-3 cursor-nw-resize" style="transform: translate(-50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'nw')}></div>
                    <div role="presentation" class="handle-invert absolute right-0 top-0 w-3 h-3 cursor-ne-resize" style="transform: translate(50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'ne')}></div>
                    <div role="presentation" class="handle-invert absolute left-0 bottom-0 w-3 h-3 cursor-sw-resize" style="transform: translate(-50%, 50%)" on:mousedown={(e) => onHandleMouseDown(e, 'sw')}></div>
                    <div role="presentation" class="handle-invert absolute right-0 bottom-0 w-3 h-3 cursor-se-resize" style="transform: translate(50%, 50%)" on:mousedown={(e) => onHandleMouseDown(e, 'se')}></div>
                    
                    <!-- Edges -->
                    <div role="presentation" class="handle-invert absolute left-1/2 top-0 w-3 h-3 cursor-n-resize" style="transform: translate(-50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'n')}></div>
                    <div role="presentation" class="handle-invert absolute left-1/2 bottom-0 w-3 h-3 cursor-s-resize" style="transform: translate(-50%, 50%)" on:mousedown={(e) => onHandleMouseDown(e, 's')}></div>
                    <div role="presentation" class="handle-invert absolute left-0 top-1/2 w-3 h-3 cursor-w-resize" style="transform: translate(-50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'w')}></div>
                    <div role="presentation" class="handle-invert absolute right-0 top-1/2 w-3 h-3 cursor-e-resize" style="transform: translate(50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'e')}></div>
                  {/if}
                </div>
              {:else if clip.type.startsWith('video')}
                <div
                  role="presentation"
                  data-preview-clip="true"
                  class="max-h-full max-w-full relative {selectedClip?.id === clip.id ? 'cursor-move' : 'cursor-pointer pointer-events-auto'}"
                  style="
                    outline: {selectedClip?.id === clip.id ? '2px solid white' : 'none'};
                    outline-offset: -1px;
                  "
                  on:mousedown={(e) => {
                    if (selectedClip?.id !== clip.id) {
                      dispatch('select', clip);
                    }
                    onClipMouseDown(e);
                  }}
                >
                  {#if clip.id === primaryVideoClip?.id}
                    <video 
                      bind:this={video}
                      src={videoSrc}
                      class="max-h-full max-w-full pointer-events-none"
                      style="visibility: {hasAdvancedEffects(clip.id) ? 'hidden' : 'visible'};"
                      preload="auto"
                      playsinline
                      on:timeupdate={() => dispatch('timeupdate', { currentTime: video.currentTime })}
                      on:loadedmetadata={onLoaded}
                    >
                      <track kind="captions" />
                    </video>
                  {:else}
                    <video 
                      src={clip.src}
                      class="max-h-full max-w-full pointer-events-none"
                      style="visibility: {hasAdvancedEffects(clip.id) ? 'hidden' : 'visible'};"
                      preload="auto"
                      playsinline
                      muted
                    >
                      <track kind="captions" />
                    </video>
                  {/if}

                  {#if selectedClip?.id === clip.id}
                    <!-- Handles -->
                    <div role="presentation" class="handle-invert absolute left-0 top-0 w-3 h-3 cursor-nw-resize" style="transform: translate(-50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'nw')}></div>
                    <div role="presentation" class="handle-invert absolute right-0 top-0 w-3 h-3 cursor-ne-resize" style="transform: translate(50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'ne')}></div>
                    <div role="presentation" class="handle-invert absolute left-0 bottom-0 w-3 h-3 cursor-sw-resize" style="transform: translate(-50%, 50%)" on:mousedown={(e) => onHandleMouseDown(e, 'sw')}></div>
                    <div role="presentation" class="handle-invert absolute right-0 bottom-0 w-3 h-3 cursor-se-resize" style="transform: translate(50%, 50%)" on:mousedown={(e) => onHandleMouseDown(e, 'se')}></div>
                    
                    <!-- Edges -->
                    <div role="presentation" class="handle-invert absolute left-1/2 top-0 w-3 h-3 cursor-n-resize" style="transform: translate(-50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'n')}></div>
                    <div role="presentation" class="handle-invert absolute left-1/2 bottom-0 w-3 h-3 cursor-s-resize" style="transform: translate(-50%, 50%)" on:mousedown={(e) => onHandleMouseDown(e, 's')}></div>
                    <div role="presentation" class="handle-invert absolute left-0 top-1/2 w-3 h-3 cursor-w-resize" style="transform: translate(-50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'w')}></div>
                    <div role="presentation" class="handle-invert absolute right-0 top-1/2 w-3 h-3 cursor-e-resize" style="transform: translate(50%, -50%)" on:mousedown={(e) => onHandleMouseDown(e, 'e')}></div>
                  {/if}
                </div>
              {/if}
            </div>
          </div>
        {/each}

        <!-- Global Effects Canvas (renders the active/top effect layer) -->
        <canvas 
          bind:this={canvas}
          class="max-h-full max-w-full object-contain absolute inset-0 m-auto pointer-events-none {isEyedropperActive ? 'cursor-crosshair pointer-events-auto' : ''}"
          style="z-index: 50; opacity: {activeClips.some(c => hasAdvancedEffects(c.id)) ? 1 : 0};"
          on:click={onCanvasClick}
        ></canvas>
      </div>
    {:else}
      <div class="flex flex-col items-center gap-4 text-zinc-500">
        <Video size={48} strokeWidth={1} />
        <button class="bg-blue-600 px-6 py-2 rounded-lg text-white font-medium disabled:opacity-60" on:click={onImportClick} disabled={isImporting}>
          {isImporting ? 'Importing...' : 'Import Media'}
        </button>
        {#if importError}
          <p class="text-xs text-red-400">{importError}</p>
        {/if}
      </div>
    {/if}
  </div>
</div>


  <div class="h-12 flex items-center gap-4 px-2 border-t border-zinc-800 bg-[#111]">
    <button class="text-white" on:click={() => dispatch('toggleplay')}>
      {#if isPlaying}<Pause size={24} />{:else}<Play size={24} />{/if}
    </button>
    <div class="text-xs font-mono text-zinc-400 tabular-nums">
      {Math.floor(currentTime / 60)}:{Math.floor(currentTime % 60).toString().padStart(2, '0')} / {Math.floor(duration / 60)}:{Math.floor(duration % 60).toString().padStart(2, '0')}
    </div>
    {#if currentFileName}
      <div class="text-[11px] text-zinc-500 truncate max-w-[260px]">{currentFileName}</div>
    {/if}
    <div class="flex-1"></div>

    <!-- Zoom Combobox -->
    <div class="relative">
      <button 
        class="h-7 px-2 flex items-center gap-1.5 bg-zinc-900 border border-zinc-800 rounded text-[10px] font-medium text-zinc-400 hover:text-zinc-200 transition-colors"
        on:click={() => showZoomMenu = !showZoomMenu}
      >
        <span>{zoomMode === 'fit' ? 'Fit' : zoomMode + '%'}</span>
        <ChevronDown size={10} class="transition-transform {showZoomMenu ? 'rotate-180' : ''}" />
      </button>

      {#if showZoomMenu}
        <div 
          use:clickOutside={() => showZoomMenu = false}
          class="absolute bottom-full left-0 mb-1 w-24 bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl z-[60] py-1 overflow-hidden"
        >
          {#each zoomOptions as option}
            <button
              class="w-full px-3 py-1.5 text-left text-[10px] transition-colors {zoomMode === option.value ? 'bg-blue-600/20 text-blue-400' : 'text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200'}"
              on:click={() => {
                zoomMode = option.value as any;
                showZoomMenu = false;
              }}
            >
              {option.label}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <button class="text-zinc-400 hover:text-white disabled:opacity-60" on:click={onImportClick} disabled={isImporting}>
      <Plus size={20} />
    </button>
  </div>
</div>

<style>
  

  .handle-invert {
    border: 1px solid #fff;
    background: #fff;
    box-shadow: 0 0 4px rgba(0,0,0,0.5);
    mix-blend-mode: difference;
  }

  .checkerboard {
    background-image: linear-gradient(45deg, #111 25%, transparent 25%),
      linear-gradient(-45deg, #111 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #111 75%),
      linear-gradient(-45deg, transparent 75%, #111 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0px;
    background-color: #0d0d0d;
  }

  .no-scrollbar::-webkit-scrollbar {
    display: none;
  }
  .no-scrollbar {
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
</style>
