<script lang="ts">
  import { Star, StarHalf } from "@lucide/svelte";

  let {
    rating,
    readonly = false,
    size = 14,
    onRate,
  }: {
    rating: number | null;
    readonly?: boolean;
    size?: number;
    onRate?: (rating: number | null) => void;
  } = $props();

  const STAR_INDICES = Array.from({ length: 10 }, (_, i) => i + 1);

  let hoverValue = $state<number | null>(null);

  const displayValue = $derived(hoverValue ?? rating ?? 0);

  function valueFromEvent(e: MouseEvent, starIndex: number): number {
    const target = e.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    const isLeftHalf = e.clientX - rect.left < rect.width / 2;
    return isLeftHalf ? starIndex - 0.5 : starIndex;
  }

  function handleMove(e: MouseEvent, starIndex: number) {
    hoverValue = valueFromEvent(e, starIndex);
  }

  function handleClick(e: MouseEvent, starIndex: number) {
    if (!onRate) return;
    const value = valueFromEvent(e, starIndex);
    // Clicking the slot that matches the current exact rating clears it
    // back to unrated, rather than re-committing the same value with no
    // visible change.
    onRate(value === rating ? null : value);
  }
</script>

{#snippet starIcon(isFull: boolean, isHalf: boolean)}
  {#if isFull}
    <span class="star-icon full"><Star {size} fill="currentColor" /></span>
  {:else if isHalf}
    <span class="star-icon-stack">
      <span class="star-icon empty"><Star {size} /></span>
      <span class="star-icon half-fill"><StarHalf {size} fill="currentColor" /></span>
    </span>
  {:else}
    <span class="star-icon empty"><Star {size} /></span>
  {/if}
{/snippet}

{#if readonly}
  <div class="star-rating readonly" aria-label={`Rating: ${rating ?? "unrated"} of 10`}>
    {#each STAR_INDICES as starIndex}
      {@const isFull = displayValue >= starIndex}
      {@const isHalf = !isFull && displayValue >= starIndex - 0.5}
      <span class="star-slot">{@render starIcon(isFull, isHalf)}</span>
    {/each}
  </div>
{:else}
  <div class="star-rating" role="group" aria-label="Rating" onmouseleave={() => (hoverValue = null)}>
    {#each STAR_INDICES as starIndex}
      {@const isFull = displayValue >= starIndex}
      {@const isHalf = !isFull && displayValue >= starIndex - 0.5}
      <button
        type="button"
        class="star-slot"
        aria-label={`Rate ${starIndex} stars`}
        onmousemove={(e) => handleMove(e, starIndex)}
        onclick={(e) => handleClick(e, starIndex)}
      >
        {@render starIcon(isFull, isHalf)}
      </button>
    {/each}
  </div>
{/if}

<style>
  .star-rating {
    display: inline-flex;
    align-items: center;
    gap: 1px;
  }

  .star-slot {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }

  button.star-slot {
    background: none;
    border: none;
    padding: 2px;
    margin: 0;
    color: inherit;
    cursor: pointer;
  }

  .star-icon {
    display: inline-flex;
  }

  .star-icon.full {
    color: var(--accent);
  }

  .star-icon.empty {
    color: var(--text-tertiary);
  }

  .star-icon-stack {
    position: relative;
    display: inline-flex;
  }

  .star-icon.half-fill {
    position: absolute;
    inset: 0;
    color: var(--accent);
  }
</style>
