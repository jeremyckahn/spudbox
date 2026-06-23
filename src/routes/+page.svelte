<script lang="ts">
  import AlbumGrid from "$lib/components/browser/AlbumGrid.svelte";
  import TrackTable from "$lib/components/tracklist/TrackTable.svelte";
  import { library } from "$lib/stores/library.svelte";
</script>

<div class="page">
  <!-- Always mounted (never destroyed by the album/grid toggle below) so its
       scroll position survives navigating to a track list and back. -->
  <div class="layer" class:hidden={library.selectedAlbumId !== null}>
    <AlbumGrid />
  </div>
  {#if library.selectedAlbumId !== null}
    <!-- Keyed by album id so switching directly between two albums' track
         lists (e.g. via the now-playing bar) fully resets the virtualizer
         instead of reusing one whose row count/measurements were set up
         for the previous album. -->
    {#key library.selectedAlbumId}
      <div class="layer">
        <TrackTable />
      </div>
    {/key}
  {/if}
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
  }

  .layer {
    position: absolute;
    inset: 0;
  }

  .hidden {
    display: none;
  }
</style>
