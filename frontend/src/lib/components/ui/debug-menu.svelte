<script lang="ts">
  import { t } from 'svelte-i18n'
  import type { Config } from '../../api'
  import { overlay } from './overlay'

  let {
    open = false,
    configData = null,
    onclose
  }: {
    open?: boolean
    configData?: Config | null
    onclose?: () => void
  } = $props()
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <button class="absolute inset-0 cursor-default bg-black/60" aria-label={$t('close')} tabindex="-1" onclick={onclose}></button>
    <div class="modal relative mx-4 w-full max-w-lg" role="dialog" aria-modal="true" aria-labelledby="debugMenuTitle" tabindex="-1" use:overlay={{ onclose: () => onclose?.() }}>
      <div class="modal-section border-b px-5 py-4">
        <h3 id="debugMenuTitle" class="text-base font-semibold">{$t('debug_menu')}</h3>
      </div>
      <div class="px-5 py-4 text-sm text-slate-700">
        <h4 class="mb-2 text-sm font-medium">{$t('config_info')}</h4>
        {#if configData}
          <div class="muted space-y-1">
            <p><span class="text-slate-800">{$t('database')}:</span> {configData.dbPath}</p>
          </div>
        {:else}
          <div class="text-slate-500">{$t('empty')}</div>
        {/if}
      </div>
      <div class="modal-section flex justify-end gap-3 border-t px-5 py-4">
        <button class="btn-neutral px-4 py-2 text-sm font-medium shadow-sm" onclick={onclose}>{$t('close')}</button>
      </div>
    </div>
  </div>
{/if}
