<script lang="ts">
  import { t } from 'svelte-i18n'
  import logo from '../../../assets/images/logo-universal.png'
  import { navigate } from '../../stores/router'

  const go = (name: string) => navigate(name as any)
</script>

<div class="max-w-md w-full px-6 py-10 text-center">
  <img alt="Logo" class="mx-auto mb-6 h-16 w-16 opacity-90" src="{logo}">
  <h1 class="text-3xl font-bold tracking-tight">{$t('welcome')}</h1>
  <p class="mt-1 text-sm text-slate-300">{$t('home_description')}</p>

  <div class="mt-8 grid grid-cols-1 gap-3">
    <button class="w-full rounded-lg bg-slate-800/60 hover:bg-slate-800 active:bg-slate-700 px-4 py-3 text-base font-medium shadow-sm ring-1 ring-white/10 transition" on:click={() => go('faculties')}>
      {$t('faculties')}
    </button>
    <button class="w-full rounded-lg bg-slate-800/60 hover:bg-slate-800 active:bg-slate-700 px-4 py-3 text-base font-medium shadow-sm ring-1 ring-white/10 transition">
      {$t('schedule')}
    </button>
    <button class="w-full rounded-lg bg-slate-800/60 hover:bg-slate-800 active:bg-slate-700 px-4 py-3 text-base font-medium shadow-sm ring-1 ring-white/10 transition">
      {$t('teachers')}
    </button>
    <button class="w-full rounded-lg bg-slate-800/60 hover:bg-slate-800 active:bg-slate-700 px-4 py-3 text-base font-medium shadow-sm ring-1 ring-white/10 transition" on:click={() => go('directions')}>{$t('directions')}</button>
  </div>

  <div class="mt-8 text-xs text-slate-400">{$t('app_name')}</div>
</div>


