import { isTauri } from '@tauri-apps/api/core';

import type { Backend } from './port';
import { TauriBackend } from './tauri';
import { DemoBackend } from './demo';

export type { Backend, PeekSide, PriceCheckRequest, WindowRole } from './port';
export { toAppError, describeError } from './port';

/** Composition root do frontend: dentro do Tauri usa o backend real; no navegador, o demo. */
export const backend: Backend = isTauri() ? new TauriBackend() : new DemoBackend();
