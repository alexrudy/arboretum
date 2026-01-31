<script lang="ts">
	import 'bootstrap/dist/css/bootstrap.min.css';
	import 'bootstrap-icons/font/bootstrap-icons.css';
</script>

<slot />

<style global>
	:root {
		--bs-primary: #0dcaf0;
		--bs-primary-rgb: 13, 202, 240;
		--brand-cyan: #0dcaf0;
	}

	body {
		background-color: #1a1a1a;
		color: #e0e0e0;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial,
			sans-serif;
		margin: 0;
		padding: 0;
		min-height: 100vh;
	}

	.bg-dark {
		background-color: #212529 !important;
	}

	.bg-darker {
		background-color: #1a1a1a !important;
	}

	.text-light {
		color: #e0e0e0 !important;
	}

	.border-secondary {
		border-color: #495057 !important;
	}

	.form-control,
	.form-select {
		background-color: #2d2d2d;
		border-color: #495057;
		color: #e0e0e0;
	}

	.form-control:focus,
	.form-select:focus {
		background-color: #2d2d2d;
		border-color: var(--brand-cyan);
		color: #e0e0e0;
		box-shadow: 0 0 0 0.25rem rgba(13, 202, 240, 0.25);
	}

	.form-control::placeholder {
		color: #6c757d;
	}

	.btn-primary {
		background-color: var(--brand-cyan);
		border-color: var(--brand-cyan);
		color: #000;
	}

	.btn-primary:hover {
		background-color: #0bb5d8;
		border-color: #0bb5d8;
		color: #000;
	}

	.card {
		background-color: #2d2d2d;
		border-color: #495057;
		color: #e0e0e0;
	}

	.list-group-item {
		background-color: #2d2d2d;
		border-color: #495057;
		color: #e0e0e0;
	}

	.list-group-item:hover {
		background-color: #3a3a3a;
	}

	.badge.bg-success {
		background-color: #198754 !important;
	}

	.badge.bg-info {
		background-color: #0dcaf0 !important;
		color: #000 !important;
	}

	.badge.bg-warning {
		background-color: #ffc107 !important;
		color: #000 !important;
	}

	.badge.bg-danger {
		background-color: #dc3545 !important;
	}

	.badge.bg-secondary {
		background-color: #6c757d !important;
	}
</style>
