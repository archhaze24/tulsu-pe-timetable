<script lang="ts">
  import { t } from 'svelte-i18n'

  let {
    open = false,
    title = '',
    message = '',
    confirmText = 'OK',
    cancelText = 'Cancel',
    onconfirm,
    oncancel
  }: {
    open?: boolean
    title?: string
    message?: string
    confirmText?: string
    cancelText?: string
    onconfirm?: () => void
    oncancel?: () => void
  } = $props()

  const onKeydown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') oncancel?.()
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <button class="absolute inset-0 cursor-default bg-black/60" aria-label={$t('close')} tabindex="-1" onclick={oncancel} onkeydown={onKeydown}></button>
    <div class="modal relative mx-4 w-full max-w-sm" role="dialog" aria-modal="true" aria-labelledby="confirmTitle">
      <div class="modal-section border-b px-5 py-4">
        <h3 id="confirmTitle" class="text-base font-semibold">{title}</h3>
      </div>
      <div class="muted px-5 py-4 text-sm">{message}</div>
      <div class="modal-section flex justify-end gap-3 border-t px-5 py-4">
        <button class="btn-neutral px-4 py-2 text-sm font-medium shadow-sm" onclick={oncancel}>{cancelText}</button>
        <button class="btn-danger px-4 py-2 text-sm font-semibold shadow-sm" onclick={onconfirm}>{confirmText}</button>
      </div>
    </div>
  </div>
{/if}
