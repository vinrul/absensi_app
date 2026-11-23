<script lang="ts">
	import { onMount } from 'svelte';

	type Waktu = { epoch_ms: number; palsu?: boolean };

	const BULAN = ['Jan', 'Feb', 'Mar', 'Apr', 'Mei', 'Jun', 'Jul', 'Agu', 'Sep', 'Okt', 'Nov', 'Des'];

	let { ringkas = false }: { ringkas?: boolean } = $props();

	let teks = $state('--:--:--');
	let tanggal = $state('');
	let palsu = $state(false);
	let tersinkron = $state(false);

	let offsetMs = 0;
	let epochTerakhir: number | null = null;
	let sedangSinkron = false;

	function dua(angka: number) {
		return String(angka).padStart(2, '0');
	}

	function perbarui() {
		const wita = new Date(Date.now() + offsetMs + 8 * 60 * 60 * 1000);
		teks = `${dua(wita.getUTCHours())}:${dua(wita.getUTCMinutes())}:${dua(wita.getUTCSeconds())}`;
		tanggal = `${wita.getUTCDate()} ${BULAN[wita.getUTCMonth()]} ${wita.getUTCFullYear()}`;
	}

	async function sinkron() {
		if (sedangSinkron) return;
		sedangSinkron = true;
		const mulai = Date.now();
		try {
			const res = await fetch('/api/v1/waktu', {
				cache: 'no-store',
				signal: AbortSignal.timeout(10_000)
			});
			if (!res.ok) return;
			const data = (await res.json()) as Waktu;
			const selesai = Date.now();
			// Saat jam palsu tidak berubah, pertahankan offset agar jam tetap berdetak
			// alih-alih melompat kembali ke nilai fake yang dibekukan.
			const epochBerubah = epochTerakhir === null || data.epoch_ms !== epochTerakhir;
			if (!data.palsu || epochBerubah) {
				// Asumsi latensi simetris: waktu server berada di tengah rentang permintaan.
				offsetMs = data.epoch_ms - Math.round((mulai + selesai) / 2);
				epochTerakhir = data.epoch_ms;
			}
			palsu = Boolean(data.palsu);
			tersinkron = true;
			perbarui();
		} catch {
			// Biarkan jam lokal tetap berdetak sampai sinkronisasi berikutnya.
		} finally {
			sedangSinkron = false;
		}
	}

	onMount(() => {
		let batal = false;
		let timerSinkron: ReturnType<typeof setTimeout> | undefined;

		perbarui();
		void sinkron();
		const detak = setInterval(perbarui, 1000);

		const jadwalkanSinkron = () => {
			if (batal) return;
			timerSinkron = setTimeout(
				async () => {
					await sinkron();
					jadwalkanSinkron();
				},
				palsu ? 15_000 : 5 * 60_000
			);
		};
		jadwalkanSinkron();

		const saatTerlihat = () => {
			if (document.visibilityState === 'visible') void sinkron();
		};
		document.addEventListener('visibilitychange', saatTerlihat);
		window.addEventListener('focus', saatTerlihat);

		return () => {
			batal = true;
			clearInterval(detak);
			if (timerSinkron) clearTimeout(timerSinkron);
			document.removeEventListener('visibilitychange', saatTerlihat);
			window.removeEventListener('focus', saatTerlihat);
		};
	});
</script>

<div
	class="flex shrink-0 items-center gap-1.5 rounded-lg border border-slate-200 bg-slate-50 px-2 py-1.5 sm:gap-2 sm:px-2.5"
	title="Waktu server: {tanggal} {teks} WITA"
>
	<span
		class="size-1.5 rounded-full {tersinkron ? 'bg-emerald-500' : 'bg-amber-400'}"
		aria-hidden="true"
	></span>
	<span class="font-mono text-[11px] font-semibold tabular-nums text-slate-800 sm:text-xs">
		{teks}
	</span>
	<span class="text-[9px] font-medium text-slate-500 sm:text-[10px]">WITA</span>
	{#if !ringkas}
		<span class="hidden text-[10px] text-slate-400 md:inline">{tanggal}</span>
	{/if}
	{#if palsu}
		<span class="rounded bg-amber-100 px-1 text-[9px] font-bold text-amber-700">PALSU</span>
	{/if}
</div>
