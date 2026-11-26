/**
 * Move o elemento para o `<body>`. Necessário para `position: fixed` quando algum
 * ancestral tem `transform` (ex.: animação de entrada), que viraria o containing block.
 */
export function portal(node: HTMLElement): { destroy(): void } {
  document.body.appendChild(node);
  return {
    destroy() {
      node.remove();
    },
  };
}
