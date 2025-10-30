<script lang="ts">
	let name = $state('world');
</script>

<h1>Hello {name}!</h1>
<p>Welcome to your new SvelteKit app with TypeScript and Bun.</p>

<input bind:value={name} placeholder="Enter your name" />