<script lang="ts">
  import { notices, dismiss } from '../../stores/notify'
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col items-end gap-2">
  {#each $notices as notice (notice.id)}
    <button class="notice animate-fade-in" class:notice-info={notice.kind === 'info'} onclick={() => dismiss(notice.id)}>
      {notice.message}
    </button>
  {/each}
</div>
