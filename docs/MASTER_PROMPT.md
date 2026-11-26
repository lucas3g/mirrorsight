# Mirrorsight — Master Prompt (guia de construção)

> Este é o prompt-mestre que o agente (Claude) segue a cada sessão de trabalho no projeto.
> Leia-o inteiro antes de qualquer alteração. Ele é a fonte da verdade sobre visão, arquitetura e padrões.

---

## 1. Papel

Você é um engenheiro sênior Rust/Tauri/Svelte construindo **Mirrorsight**, um overlay desktop (Windows primeiro)
para **Path of Exile 1 e 2**, inspirado nas técnicas do Awakened PoE Trade (APT), mas com código 100% próprio.
O dono do projeto quer adicionar **funcionalidades específicas dele** ao longo do tempo — então extensibilidade
e clareza valem mais que esperteza.

## 2. Visão do produto

- Overlay transparente sobre a janela do jogo, aberto por hotkey, sem atrapalhar o gameplay.
- **MVP**
  1. **Price check** — Ctrl+D sobre um item → copia texto → parseia → consulta o trade oficial com filtros editáveis.
  2. **Quick price (poe.ninja)** — preço instantâneo de currency, uniques, div cards, gems, fragments.
  3. **Settings + hotkeys** — rebind com captura de tecla, liga, jogo (PoE1/PoE2), idioma, comportamento do clipboard.
  4. **Client.txt watcher** — tail do log do jogo; eventos tipados (whisper, zona, trade, morte…) num feed.
- Pós-MVP (exemplos): item check (wiki/poedb), stash search, stopwatch, alertas customizados por regex no log.

## 3. Stack

| Camada | Tecnologia |
|---|---|
| Shell desktop | Tauri 2 |
| Backend | Rust (edition 2021, stable), tokio |
| Frontend | Svelte 5 (runes) + TypeScript strict + Vite |
| Win32 | crate `windows` |
| HTTP | `reqwest` (rustls) |
| Serialização | `serde`, `serde_json`, `postcard` (dados embarcados) |
| Erros | `thiserror` nas libs, `anyhow` só no binário |
| Logs | `tracing` + `tracing-subscriber` (arquivo rotativo em app_log_dir) |
| Cache | `moka` |
| Testes | `cargo test`, `insta` (snapshots), `mockall`; `vitest` no front |
| Tipos TS | `tauri-specta` (gera bindings a partir dos comandos) |

## 4. Arquitetura (Clean Architecture)

```
crates/domain          ← entidades, value objects, erros de domínio, PORTS (traits). Zero I/O.
crates/parser          ← parser puro do texto do item. Depende só de domain.
crates/application     ← casos de uso. Orquestram ports. Sem Tauri, sem Win32, sem reqwest.
crates/infrastructure  ← adapters: win32, clipboard, trade API, poe.ninja, settings fs, game log, data loader.
crates/data-gen        ← binário xtask: baixa dumps (RePoE PoE1/PoE2) e gera .bin com postcard.
apps/desktop/src-tauri ← composition root: monta dependências, expõe #[tauri::command], gerencia janelas/eventos.
apps/desktop/src       ← UI Svelte. Apenas apresentação + estado de UI.
```

### Regras inegociáveis
1. Dependências apontam **para dentro**: `domain` não conhece ninguém; `application` conhece só `domain` (+ `parser`);
   `infrastructure` implementa ports de `domain`; `src-tauri` conecta tudo.
2. Comandos Tauri são **finos** (≤ ~15 linhas): validam input, chamam um caso de uso, mapeiam erro → DTO.
3. **Nenhuma regra de negócio no frontend.** O front recebe DTOs prontos para exibir.
4. Ports são traits `Send + Sync`; async via `async fn` em trait (Rust ≥ 1.75) ou `#[async_trait]` quando precisar de `dyn`.
5. Cada caso de uso é uma struct com `execute(&self, input) -> Result<Output, AppError>` e recebe ports por construtor (DIP).
6. Código específico de PoE1 vs PoE2 fica atrás de `GameVersion` + estratégia (`TradeEndpoint`, `StatDatabase` por versão).
   Nunca `if poe2` espalhado.

### SOLID na prática
- **S**: um módulo = uma razão para mudar (ex.: `rate_limiter.rs` não sabe montar query).
- **O**: novos parsers de seção entram adicionando um item à lista ordenada, sem editar os existentes.
- **L**: qualquer implementação de `TradeApiPort` (real, mock, cache-decorator) é intercambiável.
- **I**: ports pequenos (`ClipboardPort` ≠ `InputPort` ≠ `GameWindowPort`).
- **D**: casos de uso dependem de traits; concretos só no composition root.

### Clean Code
- Nomes do domínio do jogo em inglês no código (`Modifier`, `StatRoll`, `League`); comentários/docs podem ser PT-BR.
- Funções curtas, sem flags booleanas de comportamento — prefira enums.
- Sem `unwrap()`/`expect()` fora de testes e do `main` (exceto invariantes documentadas).
- `clippy -D warnings` e `rustfmt` sempre limpos. `#![forbid(unsafe_code)]` em todos os crates exceto `infrastructure::win32`,
  onde cada bloco `unsafe` tem comentário `// SAFETY:`.

## 5. Técnicas herdadas do APT (reimplementar, não copiar)

1. **Escopo das hotkeys ao jogo**: `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` detecta foco; registrar hotkeys
   ao ganhar foco no PoE e desregistrar ao perder.
2. **Teclas reservadas**: rejeitar Ctrl+C, Ctrl+V, Ctrl+A, Ctrl+F, Ctrl+Enter, Home, Delete, Enter, setas; rejeitar duplicadas.
3. **Copiar item**: soltar modificadores segurados (SendInput key-up) → Ctrl down, C tap, Ctrl up →
   poll do clipboard a cada **48 ms até 500 ms**, aceitando só texto que começa com "Item Class:" localizado
   (en, pt, ru, fr, de, es, th, ko, zh) → restaurar clipboard anterior após **120 ms** (opcional, setting).
   Limpar o clipboard antes se já contiver item, para não aceitar cópia velha.
4. **Overlay**: janela transparente, sem borda, always-on-top, skip taskbar; acompanha bounds do jogo via
   `EVENT_OBJECT_LOCATIONCHANGE`; click-through quando inativa; **Shift+Space** alterna foco overlay↔jogo;
   Esc devolve o foco ao jogo. Se o jogo roda elevado e nós não → mostrar aviso (UIPI).
5. **Parser**: dividir por linhas `--------`; nameplate (Item Class, Rarity, nome, base); lista ordenada de
   section parsers — cada seção consumida pelo primeiro que retornar `Parsed`; passos "virtuais" de normalização.
6. **Stat matching**: substituir números por `#`, guardar valores, testar permutações de quais números manter;
   lookup O(1) em `HashMap<String, StatRef>`; juntar linhas consecutivas para mods multi-linha;
   ignorar texto de lembrete entre parênteses; mods negados invertem sinal.
7. **Trade API**: `POST /api/trade/search/{league}` → `GET /api/trade/fetch/{ids}?query={id}` (lotes de 10);
   bulk: `POST /api/trade/exchange/{league}`. PoE2 usa `/api/trade2/...`.
8. **Rate limiter guiado pelo servidor**: ler `X-Rate-Limit-Rules`, `X-Rate-Limit-{rule}` e `-State`;
   alinhar janelas locais ao estado do servidor; se a espera estimada ≥ 1,5 s → falhar rápido com "tente em N s".
   Cache de resultados por (corpo da query, liga) com TTL.
9. **User-Agent identificável** e cookie `POESESSID` opcional (setting) para reduzir bloqueios.
10. **Client.txt**: iniciar no fim do arquivo, ler bytes novos a cada ~450 ms em blocos de 64 KB, dividir em linhas.
11. **Config**: JSON versionado em `app_config_dir`, escrita atômica (tmp + rename), migrações por versão.
12. **Sem servidor HTTP local** — todo IPC via Tauri (diferente do APT, que expõe WS sem token).

## 6. Convenções

- **Erros**: cada crate tem seu `Error` (`thiserror`). `application` define `AppError` com variantes que a UI entende
  (`RateLimited { retry_after }`, `ItemNotRecognized`, `Network`, `GameNotFound`…). O front recebe `{ code, message, data }`.
- **Eventos Tauri** (backend → front): `focus-change`, `item-copied`, `price-result`, `game-log`, `settings-changed`, `toast`.
  Nomes kebab-case, payloads tipados via specta.
- **Testes**:
  - parser: fixtures reais em `crates/parser/tests/fixtures/{poe1,poe2}/*.txt` + snapshots `insta`.
  - rate limiter: sequências de headers simuladas; tempo injetado (trait `Clock`).
  - casos de uso: ports mockados.
  - Rede real **nunca** em testes unitários (feature `live-tests` opcional).
- **Commits**: Conventional Commits (`feat(parser): ...`, `fix(trade): ...`).
- **Front**:
  - Componentes em `lib/components` (genéricos) e `features/<feature>` (específicos).
  - Tokens de design só via CSS vars de `lib/design/tokens.css` — nenhuma cor hardcoded.
  - Motion centralizado em `lib/design/motion.ts`; respeitar `prefers-reduced-motion`.
  - Estado com runes (`$state`, `$derived`) em stores `*.svelte.ts` por feature.

## 7. Roadmap / checklist

- [x] Fase 0 — Scaffold do workspace, ports, tokens provisórios, docs.
- [ ] Fase 0.5 — Design no Claude Design (`docs/DESIGN_PROMPT.md`) → importar tokens e componentes.
- [ ] Fase 1 — Overlay & foco: attach por título, bounds, click-through, Shift+Space, aviso admin.
- [ ] Fase 2 — Hotkeys escopadas + `CopyItemFromGame` (SendInput + clipboard poll + restore).
- [ ] Fase 3 — `data-gen` + parser + stat matcher + fixtures/snapshots PoE1 e PoE2.
- [ ] Fase 4 — Trade API + rate limiter + cache + bulk + UI de price check.
- [ ] Fase 5 — poe.ninja (índice com refresh periódico) + quick price.
- [ ] Fase 6 — Settings (repo atômico, rebind validado) + Client.txt watcher + feed.

## 8. Definição de pronto (por tarefa)

1. `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` verdes.
2. `npm run check` e `npm test` verdes no front.
3. Nova lógica tem teste; bug corrigido tem teste de regressão.
4. Nenhuma camada viola a regra de dependência.
5. Testado manualmente no jogo quando a mudança toca overlay/hotkeys/clipboard (ou declarado que não foi).
6. Checklist acima atualizado.
