<script lang="ts">
  import { backend } from '$lib/backend';
  import Logo from '$lib/ui/Logo.svelte';
</script>

<div class="about">
  <Logo size={64} />
  <h4>Mirrorsight</h4>
  <p>Versão 0.1.0 · Rust + Tauri 2 + Svelte 5</p>
  {#if backend.kind === 'demo'}<span class="chip">Modo demonstração (fora do Tauri)</span>{/if}
  <p class="fine">
    Projeto independente, não afiliado à Grinding Gear Games. Ícones de itens © Grinding Gear Games,
    extraídos da instalação local do jogo.
  </p>
</div>

<style>
  .about { display: flex; flex-direction: column; align-items: center; text-align: center; gap: 10px; padding: 30px; }
  h4 { font-size: 22px; }
  p { color: var(--ms-text-2); }
  .fine { font-size: 11px; color: var(--ms-text-3); max-width: 380px; }
</style>
