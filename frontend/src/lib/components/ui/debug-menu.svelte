<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import { t } from 'svelte-i18n'
  import type { config } from '../../../../wailsjs/go/models'

  export let open: boolean = false
  export let configData: config.Config | null = null

  const dispatch = createEventDispatcher()

  const close = () => dispatch('close')
  const onKeydown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') close()
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <div class="absolute inset-0 bg-black/60" role="button" tabindex="0" aria-label="{$t('close')}" on:click={close} on:keydown={onKeydown}></div>
    <div class="relative mx-4 w-full max-w-lg rounded-xl bg-white text-slate-900 ring-1 ring-black/5 shadow-xl dark:bg-slate-900 dark:text-slate-50 dark:ring-white/10" role="dialog" aria-modal="true" aria-labelledby="debugMenuTitle">
      <div class="px-5 py-4 border-b border-slate-200 dark:border-white/10">
        <h3 id="debugMenuTitle" class="text-base font-semibold">{$t('debug_menu')}</h3>
      </div>
      <div class="px-5 py-4 text-sm text-slate-700 dark:text-slate-200">
        <h4 class="text-sm font-medium mb-2">{$t('config_info')}</h4>
        {#if configData}
          <div class="space-y-1 text-slate-600 dark:text-slate-300">
            <p><span class="text-slate-800 dark:text-slate-200">{$t('database')}:</span> {configData.dbPath}</p>
            <p><span class="text-slate-800 dark:text-slate-200">{$t('theme_label')}:</span> {configData.theme}</p>
          </div>
        {:else}
          <div class="text-slate-500 dark:text-slate-400">{$t('empty')}</div>
        {/if}
      </div>
      <div class="px-5 py-4 flex gap-3 justify-end border-t border-slate-200 dark:border-white/10">
        <button class="rounded-lg bg-slate-100 hover:bg-slate-200 active:bg-slate-300 px-4 py-2 text-sm font-medium shadow-sm ring-1 ring-black/5 transition dark:bg-slate-800/60 dark:hover:bg-slate-800 dark:active:bg-slate-700 dark:ring-white/10" on:click={close}>{$t('close')}</button>
      </div>
    </div>
  </div>
{/if}


