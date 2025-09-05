<script lang="ts">
  import { t } from 'svelte-i18n'
  import logo from '../../../assets/images/logo-universal.png'
  import { navigate } from '../../stores/router'
  import type {config} from '../../../../wailsjs/go/models'
  

  export let configData: config.Config | null = null

  const go = (name: string) => navigate(name as any)

  const isDark = () => (configData?.theme ?? 'light') === 'dark'
</script>

<div class="max-w-md w-full px-6 py-10 text-center">
  <div class="h-6 mb-2"></div>
  <img alt="Logo" class="mx-auto mb-6 h-16 w-16 opacity-90" src="{logo}">
  <h1 class="text-3xl font-bold tracking-tight">{$t('welcome')}</h1>
  <p class="mt-1 text-sm text-slate-600 dark:text-slate-300">{$t('home_description')}</p>

  <div class="mt-8 grid grid-cols-1 gap-3">
    <button class="w-full rounded-lg bg-slate-100 hover:bg-slate-200 active:bg-slate-300 dark:bg-slate-800/60 dark:hover:bg-slate-800 dark:active:bg-slate-700 px-4 py-3 text-base font-medium shadow-sm ring-1 ring-black/5 dark:ring-white/10 transition" on:click={() => go('faculties')}>
      {$t('faculties')}
    </button>
    <button class="w-full rounded-lg bg-slate-100 hover:bg-slate-200 active:bg-slate-300 dark:bg-slate-800/60 dark:hover:bg-slate-800 dark:active:bg-slate-700 px-4 py-3 text-base font-medium shadow-sm ring-1 ring-black/5 dark:ring-white/10 transition" on:click={() => go('semesters')}>
      {$t('schedule')}
    </button>
    <button class="w-full rounded-lg bg-slate-100 hover:bg-slate-200 active:bg-slate-300 dark:bg-slate-800/60 dark:hover:bg-slate-800 dark:active:bg-slate-700 px-4 py-3 text-base font-medium shadow-sm ring-1 ring-black/5 dark:ring-white/10 transition" on:click={() => go('teachers')}>
      {$t('teachers')}
    </button>
    <button class="w-full rounded-lg bg-slate-100 hover:bg-slate-200 active:bg-slate-300 dark:bg-slate-800/60 dark:hover:bg-slate-800 dark:active:bg-slate-700 px-4 py-3 text-base font-medium shadow-sm ring-1 ring-black/5 dark:ring-white/10 transition" on:click={() => go('directions')}>{$t('directions')}</button>
  </div>

  <div class="mt-8 text-xs text-slate-500 dark:text-slate-400">{$t('app_name')}</div>
  
  {#if configData}
    <div class="mt-6 p-4 bg-slate-50 rounded-lg border border-slate-200 dark:bg-slate-800/60 dark:border-slate-700">
      <h3 class="text-sm font-medium text-slate-700 dark:text-slate-200 mb-2">{$t('config_info')}</h3>
      <p class="text-xs text-slate-600 dark:text-slate-400 mb-1">
        <span class="text-slate-800 dark:text-slate-300">База данных:</span> {configData.dbPath}
      </p>
    </div>
  {/if}
</div>


