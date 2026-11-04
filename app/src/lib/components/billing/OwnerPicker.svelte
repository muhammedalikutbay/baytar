<script lang="ts">
  import { Button, Input } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import * as owners from '$lib/api/owners';
  import { search } from '$lib/api/search';
  import type { Owner } from '$lib/api/types';
  let { onselect }: {onselect:(owner:Owner)=>void}=$props();
  let query=$state('');
  let rows=$state<Owner[]>([]);
  let loading=$state(true);
  let failed=$state(false);
  let refresh=$state(0);
  $effect(()=>{
    const text=query.trim();refresh;let cancelled=false;loading=true;failed=false;
    const timer=setTimeout(()=>{
      (text?search(text,50).then(hits=>hits.map(h=>h.owner)):owners.list({limit:50})).then(list=>{if(!cancelled)rows=list;}).catch(()=>{if(!cancelled)failed=true;}).finally(()=>{if(!cancelled)loading=false;});
    },150);
    return()=>{cancelled=true;clearTimeout(timer);};
  });
</script>
<div class="grid gap-3">
  <Input label={tr.billingSearchOwner} bind:value={query} type="search" />
  {#if loading}<p role="status">{tr.loading}</p>{:else if failed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={()=>refresh++}>{tr.retry}</Button>{:else if !rows.length}<p>{tr.billingNoOwners}</p><a href="/patients" class="text-accent underline">{tr.newOwner}</a>{:else}
    <ul class="max-h-64 divide-y divide-border overflow-y-auto">{#each rows as owner (owner.id)}<li><Button class="w-full justify-start" variant="ghost" onclick={()=>onselect(owner)}>{owner.fullName}</Button></li>{/each}</ul>
  {/if}
</div>
