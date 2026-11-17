<script lang="ts">
	let {
		nilai,
		label,
		warna = '#0b2545',
		ukuran = 116,
		suffix = '%'
	}: { nilai: number; label: string; warna?: string; ukuran?: number; suffix?: string } = $props();

	const r = 46;
	const keliling = 2 * Math.PI * r;
	const bersih = $derived(Math.min(100, Math.max(0, Number.isFinite(nilai) ? nilai : 0)));
	const panjang = $derived((bersih / 100) * keliling);
</script>

<div class="flex flex-col items-center">
	<svg viewBox="0 0 120 120" width={ukuran} height={ukuran} role="img" aria-label="{label}: {bersih}">
		<circle cx="60" cy="60" r={r} fill="none" stroke="#e2e8f0" stroke-width="11" />
		<circle
			cx="60"
			cy="60"
			r={r}
			fill="none"
			stroke={warna}
			stroke-width="11"
			stroke-linecap="round"
			stroke-dasharray="{panjang} {keliling - panjang}"
			transform="rotate(-90 60 60)"
		/>
		<text
			x="60"
			y="63"
			text-anchor="middle"
			font-size="21"
			font-weight="700"
			fill="#1e293b">{bersih.toFixed(1)}{suffix}</text
		>
	</svg>
	<p class="mt-1 text-center text-xs font-medium text-slate-600">{label}</p>
</div>
