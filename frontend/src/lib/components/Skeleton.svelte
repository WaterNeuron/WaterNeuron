<script lang="ts">
	export let width = '3em';
</script>

<span class="skeleton" style:width></span>

<style>
	.skeleton {
		display: inline-block;
		height: 0.75em;
		border-radius: 3px;
		background: var(--switch-background-color);
		opacity: 0.6;
		vertical-align: middle;
	}
</style>
