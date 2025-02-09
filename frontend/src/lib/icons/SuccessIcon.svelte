<script lang="ts">
	export let color: '--main-button-text-color' | '--title-color';
</script>

<svg width={20} height={20} viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
	<rect x="1.25" y="1.25" width="21.5" height="21.5" rx="10.75" fill="none" />
	<path
		d="M7 11L11 15L17 9"
		stroke="var({color})"
		stroke-width="1.5"
		stroke-linecap="round"
		stroke-linejoin="round"
	/>
	<rect
		x="1.25"
		y="1.25"
		width="21.5"
		height="21.5"
		rx="10.75"
		stroke="var({color})"
		stroke-width="1.5"
	/>
</svg>
