<script lang="ts">
  import { t } from 'svelte-i18n'
  import logo from '../../../assets/images/logo-universal.png'
  import { navigate, type RouteName } from '../../stores/router'

  const sections: { route: RouteName; label: () => string }[] = [
    { route: 'semesters', label: () => $t('schedule') },
    { route: 'faculties', label: () => $t('faculties') },
    { route: 'teachers', label: () => $t('teachers') },
    { route: 'directions', label: () => $t('directions') },
  ]
</script>

<div class="max-w-md w-full px-6 py-10 text-center">
  <div class="h-6 mb-2"></div>
  <img alt="Logo" class="mx-auto mb-6 h-16 w-16 opacity-90" src={logo}>
  <h1 class="text-3xl font-bold tracking-tight">{$t('welcome')}</h1>
  <p class="muted mt-1 text-sm">{$t('home_description')}</p>

  <div class="mt-8 grid grid-cols-1 gap-3">
    {#each sections as section (section.route)}
      <button class="btn-neutral px-4 py-3 text-base font-medium shadow-sm" onclick={() => navigate(section.route)}>
        {section.label()}
      </button>
    {/each}
  </div>

  <div class="mt-8 text-xs text-slate-500">{$t('app_name')}</div>
</div>
