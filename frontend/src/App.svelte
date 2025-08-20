<script lang="ts">
  import HomeScreen from './lib/components/home/home-screen.svelte'
  import FacultiesScreen from './lib/components/faculties/faculties-screen.svelte'
  import FacultyEdit from './lib/components/faculties/faculty-edit.svelte'
  import DirectionsScreen from './lib/components/directions/directions-screen.svelte'
  import DirectionEdit from './lib/components/directions/direction-edit.svelte'
  import { route } from './lib/stores/router'
</script>

<main class="min-h-screen bg-slate-900 text-slate-50 font-sans flex items-center justify-center">
  {#if $route.name === 'home'}
    <HomeScreen />
  {:else if $route.name === 'faculties'}
    <FacultiesScreen />
  {:else if $route.name === 'faculty_edit'}
    <FacultyEdit />
  {:else if $route.name === 'directions'}
    <DirectionsScreen />
  {:else if $route.name === 'direction_edit'}
    <DirectionEdit />
  {/if}
</main>
