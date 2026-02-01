<script lang="ts">
	import '../styles/custom-bootstrap.scss';
	import 'bootstrap-icons/font/bootstrap-icons.css';
</script>

<svelte:head>
	<title>Arboretum</title>
	<meta name="description" content="A tool for exploring traces and logs" />
</svelte:head>

<slot />

<style global>
	/* Additional customizations beyond Bootstrap SCSS variables */
	:root {
		--brand-cyan: #0dcaf0;
	}
</style>
