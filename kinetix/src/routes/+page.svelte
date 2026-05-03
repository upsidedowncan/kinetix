<script lang="ts">
  import { goto } from '$app/navigation';
  import TitleBar from '../components/TitleBar.svelte';
  import { open } from '@tauri-apps/plugin-dialog';

  let projects = [
    {
      id: 1,
      title: 'New project',
      duration: '00:29',
      created: 'Apr 22, 2026',
      thumbnail: null
    }
  ];

  let searchQuery: string = '';
  let showNewProjectDialog: boolean = false;
  let projectName: string = '';
  let projectLocation: string = '';
  let projectResolution: string = '1920x1080';
  let projectFrameRate: string = '30';

  function openNewProjectDialog() {
    showNewProjectDialog = true;
    projectName = '';
    projectLocation = '';
  }

  function closeNewProjectDialog() {
    showNewProjectDialog = false;
  }

  async function selectFolder() {
    try {
      console.log('Opening folder dialog...');
      
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Project Location'
      });
      
      console.log('Selected:', selected);
      
      // Handle the result - with multiple: false, it's string or null
      if (selected === null) {
        console.log('User cancelled the dialog');
        return;
      }
      
      // Since multiple: false, selected is always a string
      projectLocation = selected as string;
      console.log('Selected folder:', projectLocation);
    } catch (error) {
      console.error('Failed to select folder:', error);
      alert('Failed to open folder dialog: ' + (error instanceof Error ? error.message : String(error)));
    }
  }

  function createProject() {
    if (projectName && projectLocation) {
      const newProject = {
        id: projects.length + 1,
        title: projectName,
        duration: '00:00',
        created: new Date().toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' }),
        thumbnail: null
      };
      projects = [newProject, ...projects];
      closeNewProjectDialog();
    }
  }

  // Filter projects based on search
  $: filteredProjects = projects.filter(p => 
    p.title.toLowerCase().includes(searchQuery.toLowerCase())
  );

  function openProject(projectId: number) {
    goto(`/project/${projectId}`);
  }
</script>

<div class="app-container bg-[#0a0a0a] h-screen flex flex-col overflow-hidden text-zinc-100">
  <!-- Custom Title Bar -->
  <TitleBar />

  <!-- Main Content -->
  <main class="flex-1 overflow-hidden flex">
    <!-- Sidebar -->
    <aside class="w-64 bg-[#141414] border-r border-zinc-800 flex flex-col">
      <div class="p-4">
        <button 
          on:click={openNewProjectDialog}
          class="w-full bg-blue-600 hover:bg-blue-500 text-white font-medium py-2.5 px-4 rounded-lg transition-all flex items-center justify-center gap-2"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
          </svg>
          New Project
        </button>
      </div>

      <nav class="flex-1 px-3">
        <div class="space-y-1">
          <a href="/" class="flex items-center gap-3 px-3 py-2 rounded-lg bg-zinc-800/50 text-zinc-100">
            <svg class="w-5 h-5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/>
            </svg>
            <span class="font-medium">Projects</span>
          </a>
        </div>
      </nav>
    </aside>

    <!-- Content Area -->
    <div class="flex-1 overflow-y-auto p-8">
      <!-- Header -->
      <header class="mb-8">
        <h1 class="text-3xl font-bold text-white mb-2">Projects</h1>
        <p class="text-zinc-400">Manage your video editing projects</p>
      </header>

      <!-- Search Bar -->
      <div class="mb-6">
        <div class="relative max-w-md">
          <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
          </svg>
          <input
            type="text"
            placeholder="Search projects..."
            bind:value={searchQuery}
            class="w-full bg-zinc-900 border border-zinc-800 text-white pl-10 pr-4 py-2.5 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent placeholder:text-zinc-500"
          />
        </div>
      </div>

      <!-- Projects Grid -->
      {#if filteredProjects.length > 0}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
          {#each filteredProjects as project}
            <div 
              class="group bg-zinc-900 rounded-xl overflow-hidden border border-zinc-800 hover:border-zinc-700 transition-all cursor-pointer hover:shadow-xl hover:shadow-black/50"
              on:click={() => openProject(project.id)}
              on:keypress={(e) => e.key === 'Enter' && openProject(project.id)}
              tabindex="0"
              role="button"
            >
              <!-- Thumbnail -->
              <div class="aspect-video bg-zinc-950 relative overflow-hidden">
                {#if project.thumbnail}
                  <img src={project.thumbnail} alt={project.title} class="w-full h-full object-cover" />
                {:else}
                  <div class="w-full h-full flex items-center justify-center">
                    <div class="w-16 h-16 rounded-full bg-zinc-800 flex items-center justify-center">
                      <svg class="w-8 h-8 text-zinc-600" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M8 5v14l11-7z"/>
                      </svg>
                    </div>
                  </div>
                {/if}
                <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
                <div class="absolute bottom-3 right-3 bg-black/80 text-white text-xs font-medium px-2 py-1 rounded">
                  {project.duration}
                </div>
              </div>

              <!-- Info -->
              <div class="p-4">
                <h3 class="text-white font-semibold mb-1 truncate">{project.title}</h3>
                <p class="text-zinc-500 text-sm">Created {project.created}</p>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="text-center py-20">
          <div class="w-20 h-20 mx-auto mb-4 rounded-full bg-zinc-900 flex items-center justify-center">
            <svg class="w-10 h-10 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"/>
            </svg>
          </div>
          <h3 class="text-xl font-semibold text-white mb-2">No projects found</h3>
          <p class="text-zinc-500 mb-6">Create your first project to get started</p>
          <button 
            on:click={openNewProjectDialog}
            class="bg-blue-600 hover:bg-blue-500 text-white font-medium py-2 px-6 rounded-lg transition-colors"
          >
            Create Project
          </button>
        </div>
      {/if}
    </div>
  </main>

  <!-- New Project Dialog -->
  {#if showNewProjectDialog}
    <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" on:click={closeNewProjectDialog} role="button" tabindex="0" on:keypress={(e) => e.key === 'Escape' && closeNewProjectDialog()}>
      <div class="bg-[#1a1a1a] rounded-xl p-6 w-[420px] shadow-2xl border border-zinc-800" on:click|stopPropagation role="dialog" aria-modal="true">
        <h2 class="text-white text-xl font-semibold mb-6">Create New Project</h2>
        
        <div class="space-y-5">
          <!-- Project Name -->
          <div>
            <label for="project-name" class="block text-sm font-medium text-zinc-400 mb-2">Project Name</label>
            <input
              id="project-name"
              type="text"
              bind:value={projectName}
              placeholder="My Awesome Video"
              class="w-full bg-zinc-900 border border-zinc-800 text-white text-sm px-3 py-2.5 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent placeholder:text-zinc-600"
            />
          </div>

          <!-- Location -->
          <div>
            <label for="project-location" class="block text-sm font-medium text-zinc-400 mb-2">Save Location</label>
            <div class="flex gap-2">
              <input
                id="project-location"
                type="text"
                bind:value={projectLocation}
                placeholder="Choose a folder..."
                class="flex-1 bg-zinc-900 border border-zinc-800 text-white text-sm px-3 py-2.5 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent placeholder:text-zinc-600"
                readonly
              />
              <button
                on:click={selectFolder}
                class="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-white text-sm font-medium rounded-lg transition-colors border border-zinc-700"
              >
                Browse
              </button>
            </div>
          </div>

          <!-- Resolution -->
          <div>
            <label for="project-resolution" class="block text-sm font-medium text-zinc-400 mb-2">Resolution</label>
            <select
              id="project-resolution"
              bind:value={projectResolution}
              class="w-full bg-zinc-900 border border-zinc-800 text-white text-sm px-3 py-2.5 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent"
            >
              <option value="1920x1080">1920 x 1080 (1080p)</option>
              <option value="3840x2160">3840 x 2160 (4K)</option>
              <option value="1280x720">1280 x 720 (720p)</option>
              <option value="custom">Custom</option>
            </select>
          </div>

          <!-- Frame Rate -->
          <div>
            <label for="project-framerate" class="block text-sm font-medium text-zinc-400 mb-2">Frame Rate</label>
            <select
              id="project-framerate"
              bind:value={projectFrameRate}
              class="w-full bg-zinc-900 border border-zinc-800 text-white text-sm px-3 py-2.5 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent"
            >
              <option value="24">24 fps (Cinematic)</option>
              <option value="30">30 fps (Standard)</option>
              <option value="60">60 fps (Smooth)</option>
            </select>
          </div>
        </div>

        <!-- Dialog Actions -->
        <div class="flex justify-end gap-3 mt-8 pt-4 border-t border-zinc-800">
          <button
            on:click={closeNewProjectDialog}
            class="px-5 py-2.5 text-sm font-medium text-zinc-400 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            on:click={createProject}
            class="px-5 py-2.5 bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-700 disabled:text-zinc-500 text-white text-sm font-medium rounded-lg transition-colors"
            disabled={!projectName || !projectLocation}
          >
            Create Project
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

