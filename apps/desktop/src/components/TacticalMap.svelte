<script lang="ts">
  import type { TacticalView } from '../tactical-api';
  import type { CharacterView } from '../table-api';
  let { tactical, characters }: { tactical: TacticalView; characters: CharacterView[] } = $props();
  const tokens = $derived(tactical.battlefield ? tactical.participants.map(p=>({id:p.entity_id,label:p.public_label,position:p.position,remembered:false})) : tactical.observers.flatMap(observer=>[
    ...(observer.position ? [{id:observer.observer,label:characters.find(c=>c.entity_id===observer.observer)?.name ?? 'You',position:observer.position,remembered:false}] : []),
    ...observer.contacts.map(contact=>({id:`${observer.observer}-${contact.entity_id}`,label:contact.label ?? (contact.status==='Remembered'?'Last known position':'Located creature'),position:contact.position,remembered:contact.status==='Remembered'}))]));
  const cells = $derived(tactical.observers.flatMap(observer=>observer.cells));
  const points = $derived([...cells.map(c=>c.position),...tokens.map(t=>t.position)]);
  const minX = $derived(tactical.battlefield?.bounds.min.x ?? Math.min(0,...points.map(p=>p.x)));
  const minY = $derived(tactical.battlefield?.bounds.min.y ?? Math.min(0,...points.map(p=>p.y)));
  const width = $derived((tactical.battlefield?.bounds.max.x ?? Math.max(10,...points.map(p=>p.x+10)))-minX);
  const height = $derived((tactical.battlefield?.bounds.max.y ?? Math.max(10,...points.map(p=>p.y+10)))-minY);
</script>
<figure>
  <svg viewBox={`${minX} ${minY} ${width} ${height}`} role="img" aria-label={tactical.battlefield ? 'Host encounter map' : 'Your known surroundings'}>
    <title>{tactical.battlefield ? 'Host encounter map' : 'Your known surroundings'}</title>
    <defs><pattern id="tactical-grid" width="10" height="10" patternUnits="userSpaceOnUse"><path d="M 10 0 L 0 0 0 10" fill="none" stroke="#93a6b4" stroke-width="0.3" /></pattern></defs>
    <rect x={minX} y={minY} width={width} height={height} fill="#142330" />
    {#if tactical.battlefield}
      <rect x={minX} y={minY} width={width} height={height} fill="#edf3ed" />
      {#each tactical.battlefield.terrain as terrain}<rect x={terrain.volume.min.x} y={terrain.volume.min.y} width={terrain.volume.max.x-terrain.volume.min.x} height={terrain.volume.max.y-terrain.volume.min.y} fill={terrain.difficult?'#d8b975':'#abd0d8'} />{/each}
      {#each tactical.battlefield.obstacles as obstacle}<rect x={obstacle.volume.min.x} y={obstacle.volume.min.y} width={obstacle.volume.max.x-obstacle.volume.min.x} height={obstacle.volume.max.y-obstacle.volume.min.y} fill="#64707a" />{/each}
    {:else}
      {#each cells as cell}<rect x={cell.position.x} y={cell.position.y} width="10" height="10" fill={cell.blocked?'#64707a':cell.difficult?'#d8b975':'#edf3ed'} opacity={cell.currently_seen?1:0.45} />{/each}
    {/if}
    <rect x={minX} y={minY} width={width} height={height} fill="url(#tactical-grid)" />
    {#each tokens as token}<g opacity={token.remembered?0.5:1}><title>{token.label}: {token.position.x/2} feet east, {token.position.y/2} feet south</title><circle cx={token.position.x+5} cy={token.position.y+5} r="3.7" fill="#1b6482" stroke="white" stroke-width="0.5" /><text x={token.position.x+5} y={token.position.y+6.3} text-anchor="middle" fill="white" font-size="4">{token.label.slice(0,1)}</text></g>{/each}
  </svg>
  {#if tokens.length}<ul>{#each tokens as token}<li>{token.label}: {token.position.x/2} feet east, {token.position.y/2} feet south{token.remembered?' (last known)':''}</li>{/each}</ul>{:else}<p>No surroundings are currently known.</p>{/if}
  <figcaption>Each square is 5 feet. Gold is difficult ground; gray is blocked. Faded terrain and contacts show remembered information.</figcaption>
</figure>
<style>figure{margin:1rem 0}svg{display:block;width:100%;max-height:32rem;border-radius:0.5rem}figcaption{font-size:0.9rem;margin-top:0.5rem}</style>
