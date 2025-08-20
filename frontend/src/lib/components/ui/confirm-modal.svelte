<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  export let open: boolean = false
  export let title: string = ''
  export let message: string = ''
  export let confirmText: string = 'OK'
  export let cancelText: string = 'Cancel'

  const dispatch = createEventDispatcher()
  const confirm = () => dispatch('confirm')
  const cancel = () => dispatch('cancel')
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <div class="absolute inset-0 bg-black/60" role="button" tabindex="0" on:click={cancel} on:keydown={(e) => (e.key === 'Escape' ? cancel() : null)}></div>
    <div class="relative mx-4 w-full max-w-sm rounded-xl bg-slate-900 text-slate-50 ring-1 ring-white/10 shadow-xl">
      <div class="px-5 py-4 border-b border-white/10">
        <h3 class="text-base font-semibold">{title}</h3>
      </div>
      <div class="px-5 py-4 text-sm text-slate-300">
        {message}
      </div>
      <div class="px-5 py-4 flex gap-3 justify-end border-t border-white/10">
        <button class="rounded-lg bg-slate-800/60 hover:bg-slate-800 active:bg-slate-700 px-4 py-2 text-sm font-medium shadow-sm ring-1 ring-white/10 transition" on:click={cancel}>{cancelText}</button>
        <button class="rounded-lg bg-rose-600/90 hover:bg-rose-600 active:bg-rose-700 px-4 py-2 text-sm font-semibold shadow-sm transition" on:click={confirm}>{confirmText}</button>
      </div>
    </div>
  </div>
{/if}


