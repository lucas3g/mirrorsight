import { mount } from 'svelte';

import './styles/global.css';
import App from './App.svelte';
import OverlayApp from './OverlayApp.svelte';
import { backend } from '$lib/backend';
import { preloadCurrencyManifest } from '$lib/currency';

// Manifest local e pequeno: carregar antes evita ícones "piscando" no primeiro render.
await preloadCurrencyManifest();

const target = document.getElementById('app');
if (!target) throw new Error('elemento #app não encontrado');

// Mesmo bundle para as duas janelas; cada uma monta seu app.
const isOverlay = backend.windowRole() === 'overlay';
if (isOverlay) document.documentElement.classList.add('overlay');

export default mount(isOverlay ? OverlayApp : App, { target });
