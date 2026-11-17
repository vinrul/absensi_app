<script lang="ts">
	let {
		page,
		perPage,
		total,
		onchange,
		label = 'data'
	}: {
		page: number;
		perPage: number;
		total: number;
		onchange: (page: number) => void;
		label?: string;
	} = $props();

	const totalHalaman = $derived(Math.max(1, Math.ceil(total / perPage)));
	const halaman = $derived(Math.min(Math.max(1, page), totalHalaman));
	const mulai = $derived(total === 0 ? 0 : (halaman - 1) * perPage + 1);
	const sampai = $derived(Math.min(halaman * perPage, total));
</script>

<div class="flex flex-wrap items-center justify-between gap-2 border-t border-slate-100 px-4 py-3 text-sm">
	<span class="text-slate-500">
		Menampilkan {mulai}–{sampai} dari {total} {label}
	</span>
	<div class="flex items-center gap-2">
		<button
			class="btn btn-ghost px-2 py-1"
			disabled={halaman <= 1}
			onclick={() => onchange(halaman - 1)}
		>
			‹ Sebelumnya
		</button>
		<span class="text-slate-600">Halaman {halaman} / {totalHalaman}</span>
		<button
			class="btn btn-ghost px-2 py-1"
			disabled={halaman >= totalHalaman}
			onclick={() => onchange(halaman + 1)}
		>
			Berikutnya ›
		</button>
	</div>
</div>
