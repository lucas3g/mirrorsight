export type ToastKind = 'info' | 'success' | 'warning' | 'error';

export interface Toast {
  id: number;
  kind: ToastKind;
  message: string;
  leaving: boolean;
}

const VISIBLE_MS = 3200;
const LEAVE_MS = 300;

class ToastStore {
  items = $state<Toast[]>([]);
  private nextId = 1;

  push(kind: ToastKind, message: string): void {
    const id = this.nextId++;
    this.items = [{ id, kind, message, leaving: false }, ...this.items].slice(0, 4);
    setTimeout(() => this.dismiss(id), VISIBLE_MS);
  }

  dismiss(id: number): void {
    const toast = this.items.find((t) => t.id === id);
    if (!toast || toast.leaving) return;
    toast.leaving = true;
    setTimeout(() => {
      this.items = this.items.filter((t) => t.id !== id);
    }, LEAVE_MS);
  }
}

export const toasts = new ToastStore();
