<script lang="ts">
	import '../styles/custom-bootstrap.scss';
	import 'bootstrap-icons/font/bootstrap-icons.css';
</script>

<slot />

<style global>
	/* Additional customizations beyond Bootstrap SCSS variables */
	:root {
		--brand-cyan: #0dcaf0;
	}

	.bg-darker {
		background-color: #1a1a1a !important;
	}

	.card-title {
		color: var(--brand-cyan);
	}

	.spinner-border {
		border-color: var(--brand-cyan);
		border-right-color: transparent;
	}
</style>
