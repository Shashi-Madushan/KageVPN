<script lang="ts">
	export type Mood = 'sleep' | 'happy' | 'busy' | 'worried';

	let { mood, size = 132 }: { mood: Mood; size?: number } = $props();
</script>

<svg class="mascot {mood}" width={size} height={size} viewBox="0 0 256 256" aria-hidden="true">
	<ellipse class="shadow" cx="128" cy="226" rx="62" ry="8" />
	{#if mood === 'sleep'}
		<g class="zzz"><text x="176" y="62">z</text><text x="196" y="40">Z</text></g>
	{/if}
	{#if mood === 'happy'}
		<g class="sparkles">
			<path d="M52 70l4 10 10 4-10 4-4 10-4-10-10-4 10-4z" />
			<path d="M206 96l3 7 7 3-7 3-3 7-3-7-7-3 7-3z" />
		</g>
	{/if}

	<g class="body-group">
		<circle class="moon" cx="128" cy="112" r="78" />
		<!-- ninja body -->
		<path class="ninja" d="M92 214 L104 142 Q128 128 152 142 L164 214 Z" />
		<!-- arms -->
		<path class="ninja" d="M104 150 Q86 168 92 190 L104 186 Q104 170 116 160 Z" />
		<path class="ninja" d="M152 150 Q170 168 164 190 L152 186 Q152 170 140 160 Z" />
		<!-- scarf -->
		<rect class="scarf" x="106" y="140" width="44" height="9" rx="4.5" />
		<path class="scarf" d="M146 144 q18 6 22 22 q-12 -6 -26 -8 z" />
		<!-- head -->
		<circle class="head" cx="128" cy="104" r="30" />
		<rect class="mask" x="102" y="94" width="52" height="19" rx="9.5" />

		<g class="eyes">
			{#if mood === 'sleep'}
				<path class="lid" d="M110 105 q6 5 12 0 M134 105 q6 5 12 0" />
			{:else if mood === 'happy'}
				<g class="blink">
					<path class="lid" d="M110 107 q6 -9 12 0 M134 107 q6 -9 12 0" />
				</g>
				<circle class="cheek" cx="104" cy="118" r="5" />
				<circle class="cheek" cx="152" cy="118" r="5" />
			{:else if mood === 'busy'}
				<g class="look">
					<ellipse class="eye" cx="116" cy="104" rx="5" ry="6.5" />
					<ellipse class="eye" cx="140" cy="104" rx="5" ry="6.5" />
				</g>
			{:else}
				<ellipse class="eye" cx="116" cy="106" rx="4.5" ry="5.5" />
				<ellipse class="eye" cx="140" cy="106" rx="4.5" ry="5.5" />
				<path class="brow" d="M108 93 l16 5 M148 93 l-16 5" />
			{/if}
		</g>
	</g>

	{#if mood === 'worried'}
		<path class="sweat" d="M172 78 q9 14 0 20 q-9 -6 0 -20z" />
	{/if}
</svg>

<style>
	.mascot {
		--moon: #e9e3ff;
		--ink: #0b0a16;
		--accent: #8b6dff;
		overflow: visible;
		display: block;
	}
	.shadow {
		fill: rgb(0 0 0 / 0.14);
	}
	.moon {
		fill: var(--moon);
		transition: fill 0.3s;
	}
	.ninja,
	.head {
		fill: var(--ink);
	}
	.scarf {
		fill: var(--accent);
		transition: fill 0.3s;
	}
	.mask {
		fill: var(--moon);
	}
	.eye {
		fill: var(--ink);
	}
	.lid,
	.brow {
		fill: none;
		stroke: var(--ink);
		stroke-width: 4;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.cheek {
		fill: #ff8fb0;
		opacity: 0.75;
	}
	.sweat {
		fill: #7fd6ff;
		animation: drip 1.6s ease-in infinite;
	}
	.sparkles path {
		fill: #ffd86b;
		transform-box: fill-box;
		transform-origin: center;
		animation: twinkle 2.4s ease-in-out infinite;
	}
	.sparkles path:last-child {
		animation-delay: 1.1s;
	}
	.zzz text {
		fill: var(--accent);
		font: 700 30px system-ui, sans-serif;
		animation: rise 3s ease-in-out infinite;
	}
	.zzz text:last-child {
		font-size: 38px;
		animation-delay: 1.2s;
	}

	/* mood colours */
	.sleep {
		--moon: #c9c6d6;
		--accent: #9a98ab;
	}
	.busy {
		--moon: #ffe9bf;
		--accent: #f29d0e;
	}
	.worried {
		--moon: #ffd3cf;
		--accent: #e5453b;
	}
	.happy .moon {
		filter: drop-shadow(0 0 14px rgb(40 180 99 / 0.55));
	}
	.busy .moon {
		filter: drop-shadow(0 0 12px rgb(242 157 14 / 0.5));
	}
	.worried .moon {
		filter: drop-shadow(0 0 12px rgb(229 69 59 / 0.45));
	}

	/* motion */
	.body-group {
		transform-box: fill-box;
		transform-origin: 50% 100%;
	}
	.happy .body-group {
		animation: float 3.2s ease-in-out infinite;
	}
	.sleep .body-group {
		animation: breathe 4s ease-in-out infinite;
	}
	.busy .body-group {
		animation: float 1.2s ease-in-out infinite;
	}
	.worried .body-group {
		animation: shiver 0.5s linear infinite;
	}
	.blink {
		transform-box: fill-box;
		transform-origin: center;
		animation: blink 5s infinite;
	}
	.look {
		animation: look 2s ease-in-out infinite;
	}
	@keyframes float {
		50% {
			transform: translateY(-7px);
		}
	}
	@keyframes breathe {
		50% {
			transform: scale(1.025);
		}
	}
	@keyframes shiver {
		25% {
			transform: translateX(-1.5px);
		}
		75% {
			transform: translateX(1.5px);
		}
	}
	@keyframes blink {
		92%,
		100% {
			transform: scaleY(1);
		}
		96% {
			transform: scaleY(0.1);
		}
	}
	@keyframes look {
		25% {
			transform: translateX(-4px);
		}
		75% {
			transform: translateX(4px);
		}
	}
	@keyframes twinkle {
		50% {
			transform: scale(0.5);
			opacity: 0.5;
		}
	}
	@keyframes rise {
		0% {
			opacity: 0;
			transform: translateY(10px);
		}
		40% {
			opacity: 1;
		}
		100% {
			opacity: 0;
			transform: translateY(-14px);
		}
	}
	@keyframes drip {
		0% {
			opacity: 0;
			transform: translateY(-6px);
		}
		30% {
			opacity: 1;
		}
		100% {
			opacity: 0;
			transform: translateY(14px);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.mascot *,
		.mascot {
			animation: none !important;
		}
	}
</style>
