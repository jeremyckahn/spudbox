import { commands } from "$lib/api/commands";
import type { AlbumRow, ArtistRow, TrackRow } from "$lib/types";

function createLibraryStore() {
  let artists = $state<ArtistRow[]>([]);
  let albums = $state<AlbumRow[]>([]);
  let tracks = $state<TrackRow[]>([]);
  let selectedArtistId = $state<number | null>(null);
  let selectedAlbumId = $state<number | null>(null);
  let loading = $state(false);
  let hasRoots = $state(true);

  async function loadAlbums() {
    albums = await commands.libraryGetAlbums(selectedArtistId);
  }

  async function selectArtist(artistId: number | null) {
    selectedArtistId = artistId;
    selectedAlbumId = null;
    await loadAlbums();
  }

  async function selectAlbum(albumId: number) {
    // Fetch first, then assign both together: assigning selectedAlbumId
    // alone would immediately remount the keyed track-list view (see
    // +page.svelte) while `tracks` still held the previous album's rows,
    // so the freshly-mounted virtualizer would briefly size itself off
    // stale data.
    const newTracks = await commands.libraryGetTracksByAlbum(albumId);
    selectedAlbumId = albumId;
    tracks = newTracks;
  }

  function backToAlbums() {
    selectedAlbumId = null;
  }

  // Navigates to an album's track list regardless of the currently
  // selected artist filter (e.g. from the now-playing bar, which can
  // point at an album outside whatever's currently browsed) by resetting
  // to "All Albums" first so the target album is guaranteed to be in the
  // loaded list.
  async function goToAlbum(albumId: number) {
    selectedArtistId = null;
    await loadAlbums();
    await selectAlbum(albumId);
  }

  async function refresh() {
    loading = true;
    try {
      hasRoots = await commands.libraryHasRoots();
      artists = await commands.libraryGetArtists();
      await loadAlbums();
    } finally {
      loading = false;
    }
  }

  async function rescan() {
    loading = true;
    try {
      await commands.libraryScan();
      await refresh();
    } finally {
      loading = false;
    }
  }

  async function addFolder(path: string) {
    loading = true;
    try {
      await commands.libraryAddRoot(path);
      await commands.libraryScan();
      await refresh();
    } finally {
      loading = false;
    }
  }

  return {
    get artists() {
      return artists;
    },
    get albums() {
      return albums;
    },
    get tracks() {
      return tracks;
    },
    get selectedArtistId() {
      return selectedArtistId;
    },
    get selectedAlbumId() {
      return selectedAlbumId;
    },
    get loading() {
      return loading;
    },
    get hasRoots() {
      return hasRoots;
    },
    selectArtist,
    selectAlbum,
    goToAlbum,
    backToAlbums,
    refresh,
    rescan,
    addFolder,
  };
}

export const library = createLibraryStore();
