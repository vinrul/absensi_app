<script lang="ts">
	interface Titik {
		nama: string;
		rata_rata: number;
	}

	let { points = [] }: { points?: Titik[] } = $props();

	const lebar = 420;
	const tinggi = 170;
	const pad = { kiri: 36, kanan: 12, atas: 14, bawah: 26 };

	const areaLebar = $derived(lebar - pad.kiri - pad.kanan);
	const areaTinggi = $derived(tinggi - pad.atas - pad.bawah);

	const minY = $derived(
		points.length ? Math.max(0, Math.floor(Math.min(...points.map((p) => p.rata_rata)) / 5) * 5 - 5) : 0
	);
	const maxY = $derived(points.length ? Math.ceil(Math.max(...points.map((p) => p.rata_rata)) / 5) * 5 + 5 : 100);

	function x(index: number) {
		if (points.length <= 1) return pad.kiri + areaLebar / 2;
		return pad.kiri + (index / (points.length - 1)) * areaLebar;
	}

	function y(nilai: number) {
		const rentang = maxY - minY || 1;
		return pad.atas + areaTinggi - ((nilai - minY) / rentang) * areaTinggi;
	}

	const garis = $derived(points.map((p, i) => `${x(i)},${y(p.rata_rata)}`).join(' '));
	const areaPath = $derived(
		points.length
			? `M ${x(0)} ${pad.atas + areaTinggi} L ${points
					.map((p, i) => `${x(i)} ${y(p.rata_rata)}`)
					.join(' L ')} L ${x(points.length - 1)} ${pad.atas + areaTinggi} Z`
			: ''
	);
</script>

<svg viewBox="0 0 {lebar} {tinggi}" class="w-full" role="img" aria-label="Tren capaian kinerja">
	{#each [0, 25, 50, 75, 100] as persen}
		{@const nilai = minY + ((maxY - minY) * persen) / 100}
		<line
			x1={pad.kiri}
			x2={lebar - pad.kanan}
			y1={y(nilai)}
			y2={y(nilai)}
			stroke="#e2e8f0"
			stroke-width="1"
			stroke-dasharray="3 3"
		/>
		<text x={pad.kiri - 6} y={y(nilai) + 3} text-anchor="end" font-size="9" fill="#94a3b8">
			{nilai.toFixed(0)}
		</text>
	{/each}

	{#if points.length}
		<path d={areaPath} fill="#0b2545" opacity="0.08" />
		<polyline points={garis} fill="none" stroke="#0b2545" stroke-width="2.5" stroke-linejoin="round" />

		{#each points as p, i}
			<circle cx={x(i)} cy={y(p.rata_rata)} r="4" fill="#d4a017" stroke="#0b2545" stroke-width="1.5" />
			<text x={x(i)} y={y(p.rata_rata) - 9} text-anchor="middle" font-size="10" font-weight="700" fill="#0b2545">
				{p.rata_rata.toFixed(1)}
			</text>
			<text x={x(i)} y={tinggi - 8} text-anchor="middle" font-size="9" fill="#64748b">
				{p.nama.length > 12 ? p.nama.slice(0, 12) : p.nama}
			</text>
		{/each}
	{:else}
		<text x={lebar / 2} y={tinggi / 2} text-anchor="middle" font-size="11" fill="#94a3b8">
			Belum ada data periode
		</text>
	{/if}
</svg>
