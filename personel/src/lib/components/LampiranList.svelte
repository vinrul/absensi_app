<script lang="ts">
	import { onDestroy } from 'svelte';
	import { apiBlobUrl } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import { ukuranBerkas } from '#lib/format';
	import type { Lampiran } from '#lib/types';

	let {
		lampiran,
		onhapus,
		judul = 'Lampiran'
	}: { lampiran: Lampiran[]; onhapus?: (item: Lampiran) => void; judul?: string } = $props();

	let memuatId = $state<string | null>(null);
	let galat = $state<string | null>(null);
	const urls = new Map<string, string>();

	async function buka(item: Lampiran) {
		galat = null;
		const tersimpan = urls.get(item.id);
		if (tersimpan) {
			window.open(tersimpan, '_blank');
			return;
		}
		memuatId = item.id;
		try {
			const url = await apiBlobUrl(`lampiran/raw/${item.storage_key}`);
			urls.set(item.id, url);
			window.open(url, '_blank');
		} catch {
			galat = 'Berkas tidak dapat dibuka. Coba lagi.';
		} finally {
			memuatId = null;
		}
	}

	onDestroy(() => {
		for (const url of urls.values()) URL.revokeObjectURL(url);
		urls.clear();
	});
</script>

{#if lampiran.length > 0}
	<div class="space-y-2">
		<p class="text-[11px] font-bold uppercase tracking-wider text-ink-500">{judul}</p>
		{#each lampiran as item (item.id)}
			<div class="flex items-center gap-2 rounded-xl border border-ink-100 bg-ink-50/60 px-3 py-2">
				<Icon name="file" size={16} class="shrink-0 text-brand-600" />
				<div class="min-w-0 flex-1">
					<p class="truncate text-xs font-semibold text-ink-700">{item.file_name}</p>
					<p class="text-[10px] text-ink-400">{ukuranBerkas(item.size_bytes)}</p>
				</div>
				<button
					type="button"
					class="btn btn-kedua px-2 py-1 text-[11px]"
					disabled={memuatId === item.id}
					onclick={() => buka(item)}
				>
					{memuatId === item.id ? 'Memuat…' : 'Lihat'}
				</button>
				{#if onhapus}
					<button
						type="button"
						class="rounded-lg p-1.5 text-rose-600 hover:bg-rose-50"
						aria-label="Hapus lampiran"
						onclick={() => onhapus?.(item)}
					>
						<Icon name="trash" size={15} />
					</button>
				{/if}
			</div>
		{/each}
		{#if galat}
			<p class="text-xs text-rose-600">{galat}</p>
		{/if}
	</div>
{/if}
