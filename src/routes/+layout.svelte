<script lang="ts">
  import { onMount } from 'svelte';
  import './app.css';

  onMount(async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().show();
    } catch (e) {
      console.warn('Could not show window:', e);
    }
  });
</script>

<slot />