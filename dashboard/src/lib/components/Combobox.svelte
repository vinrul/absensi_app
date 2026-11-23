<script lang="ts">
	import { tick } from 'svelte';
	import Icon from './Icon.svelte';

	type Opsi = {
		value: string;
		label: string;
		keterangan?: string;
		grup?: string;
	};

	let {
		id,
		value = $bindable(''),
		options,
		placeholder = 'Pilih',
		labelKosong = null,
		disabled = false,
		onchange
	}: {
		id?: string;
		value?: string;
		options: Opsi[];
		placeholder?: string;
		labelKosong?: string | null;
		disabled?: boolean;
		onchange?: (nilai: string) => void;
	} = $props();

	const idUnik = $props.id();
	const idDaftar = `daftar-${idUnik}`;

	let terbuka = $state(false);
	let kataKunci = $state('');
	let aktif = $state(0);
	let akar = $state<HTMLDivElement | null>(null);
	let tombol = $state<HTMLButtonElement | null>(null);
	let panel = $state<HTMLDivElement | null>(null);
	let inputCari = $state<HTMLInputElement | null>(null);
	let posisi = $state({ kiri: 0, atas: 0, lebar: 0 });

	const labelTerpilih = $derived.by(() => {
		if (!value && labelKosong !== null) return labelKosong;
		return options.find((o) => o.value === value)?.label ?? (value || null);
	});

	const keteranganTerpilih = $derived(
		options.find((o) => o.value === value)?.keterangan ?? ''
	);

	const opsiTampil = $derived.by(() => {
		const daftar: Opsi[] = labelKosong !== null ? [{ value: '', label: labelKosong }] : [];
		daftar.push(...options);
		const dicari = kataKunci.trim().toLowerCase();
		if (!dicari) return daftar;
		return daftar.filter((o) =>
			`${o.label} ${o.value} ${o.keterangan ?? ''} ${o.grup ?? ''}`
				.toLowerCase()
				.includes(dicari)
		);
	});

	const aktifAman = $derived(opsiTampil.length ? Math.min(aktif, opsiTampil.length - 1) : -1);

	function ukur(perkiraan = false) {
		if (!akar) return;
		const kotak = akar.getBoundingClientRect();
		const tinggi = perkiraan ? 280 : (panel?.offsetHeight ?? 280);
		const ruangBawah = window.innerHeight - kotak.bottom - 8;
		const ruangAtas = kotak.top - 8;
		const diAtas = ruangBawah < tinggi && ruangAtas > ruangBawah;
		const lebar = Math.min(kotak.width, window.innerWidth - 16);
		const kiri = Math.max(8, Math.min(kotak.left, window.innerWidth - lebar - 8));
		posisi = {
			kiri,
			lebar,
			atas: diAtas ? Math.max(8, kotak.top - tinggi - 4) : kotak.bottom + 4
		};
	}

	function gulirKeAktif() {
		if (!panel || aktifAman < 0) return;
		panel.querySelector<HTMLElement>(`[data-indeks="${aktifAman}"]`)?.scrollIntoView({
			block: 'nearest'
		});
	}

	function buka() {
		if (disabled || terbuka) return;
		ukur(true);
		kataKunci = '';
		aktif = Math.max(0, opsiTampil.findIndex((o) => o.value === value));
		terbuka = true;
		void tick().then(() => {
			ukur();
			gulirKeAktif();
			inputCari?.focus();
		});
	}

	function tutup(kembalikanFokus = false) {
		if (!terbuka) return;
		terbuka = false;
		kataKunci = '';
		if (kembalikanFokus) void tick().then(() => tombol?.focus());
	}

	function pilih(opsi: Opsi) {
		const berubah = opsi.value !== value;
		value = opsi.value;
		tutup(true);
		if (berubah) onchange?.(value);
	}

	function padaTombolCari(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (opsiTampil.length) {
				aktif = (aktifAman + 1) % opsiTampil.length;
				void tick().then(gulirKeAktif);
			}
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (opsiTampil.length) {
				aktif = (aktifAman - 1 + opsiTampil.length) % opsiTampil.length;
				void tick().then(gulirKeAktif);
			}
		} else if (e.key === 'Enter') {
			e.preventDefault();
			const opsi = opsiTampil[aktifAman];
			if (opsi) pilih(opsi);
		} else if (e.key === 'Escape') {
			e.preventDefault();
			tutup(true);
		} else if (e.key === 'Tab') {
			tutup();
		}
	}

	$effect(() => {
		if (!terbuka) return;
		const ikuti = () => ukur();
		window.addEventListener('scroll', ikuti, true);
		window.addEventListener('resize', ikuti);
		return () => {
			window.removeEventListener('scroll', ikuti, true);
			window.removeEventListener('resize', ikuti);
		};
	});

	$effect(() => {
		if (!terbuka) return;
		void opsiTampil.length;
		void tick().then(() => ukur());
	});

	$effect(() => {
		if (!terbuka) return;
		const padaPointer = (e: PointerEvent) => {
			if (!akar || !(e.target instanceof Node) || akar.contains(e.target)) return;
			tutup();
		};
		document.addEventListener('pointerdown', padaPointer, true);
		return () => document.removeEventListener('pointerdown', padaPointer, true);
	});
</script>

<div class="relative" bind:this={akar}>
	<button
		bind:this={tombol}
		type="button"
		{id}
		class="input flex w-full items-center justify-between gap-2 text-left"
		class:text-slate-400={!labelTerpilih}
		{disabled}
		aria-haspopup="listbox"
		aria-expanded={terbuka}
		onclick={() => (terbuka ? tutup(true) : buka())}
		onkeydown={(e) => {
			if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
				e.preventDefault();
				buka();
			}
		}}
	>
		<span class="flex min-w-0 items-baseline gap-1.5">
			<span class="truncate">{labelTerpilih ?? placeholder}</span>
			{#if keteranganTerpilih}
				<span class="shrink-0 text-xs text-slate-400">· {keteranganTerpilih}</span>
			{/if}
		</span>
		<Icon name="down" size={15} class="shrink-0 text-slate-400" />
	</button>

	{#if terbuka}
		<div
			bind:this={panel}
			class="fixed z-[70] flex flex-col overflow-hidden rounded-lg border border-slate-200 bg-white shadow-xl"
			style:left="{posisi.kiri}px"
			style:top="{posisi.atas}px"
			style:width="{posisi.lebar}px"
			style:max-height="min(20rem, calc(100vh - 1rem))"
		>
			<div class="shrink-0 border-b border-slate-100 p-2">
				<div class="relative">
					<span
						class="pointer-events-none absolute inset-y-0 left-2.5 flex items-center text-slate-400"
					>
						<Icon name="search" size={14} />
					</span>
					<input
						bind:this={inputCari}
						class="input py-1.5 pl-8"
						placeholder="Cari…"
						autocomplete="off"
						spellcheck={false}
						role="combobox"
						aria-expanded="true"
						aria-controls={idDaftar}
						aria-autocomplete="list"
						aria-activedescendant={aktifAman >= 0 ? `${idDaftar}-${aktifAman}` : undefined}
						bind:value={kataKunci}
						oninput={() => (aktif = 0)}
						onkeydown={padaTombolCari}
					/>
				</div>
			</div>
			<ul id={idDaftar} class="min-h-0 flex-1 overflow-y-auto py-1 text-sm" role="listbox">
				{#if opsiTampil.length === 0}
					<li class="px-3 py-2 text-xs text-slate-500">
						Tidak ada pilihan yang cocok dengan “{kataKunci.trim()}”.
					</li>
				{:else}
					{#each opsiTampil as opsi, i (opsi.value)}
						{#if opsi.grup && (i === 0 || opsiTampil[i - 1].grup !== opsi.grup)}
							<li
								class="px-3 pb-1 pt-2 text-[11px] font-semibold uppercase tracking-wide text-slate-400"
								role="presentation"
							>
								{opsi.grup}
							</li>
						{/if}
						<li role="presentation">
							<button
								type="button"
								id="{idDaftar}-{i}"
								data-indeks={i}
								role="option"
								tabindex={-1}
								aria-selected={opsi.value === value}
								class="flex w-full cursor-pointer items-center justify-between gap-2 px-3 py-1.5 text-left text-slate-700"
								class:bg-navy-50={i === aktifAman}
								class:text-navy-800={opsi.value === value}
								onclick={() => pilih(opsi)}
								onmouseenter={() => (aktif = i)}
							>
								<span class="flex min-w-0 items-baseline gap-1.5">
									<span class="truncate">{opsi.label}</span>
									{#if opsi.keterangan}
										<span class="shrink-0 text-xs text-slate-400">{opsi.keterangan}</span>
									{/if}
								</span>
								{#if opsi.value === value}
									<Icon name="check" size={14} class="shrink-0 text-navy-600" />
								{/if}
							</button>
						</li>
					{/each}
				{/if}
			</ul>
		</div>
	{/if}
</div>
