# Prompt para o Claude Design — Mirrorsight

> Como usar: abra o Claude Design (dentro do Claude), selecione **Opus 5.5** com esforço **max**,
> cole tudo abaixo da linha. Ao final, exporte o handoff (HTML/CSS + tokens) para `design/` neste repositório.

---

Crie o design system e as telas de **Mirrorsight**, um overlay desktop para o jogo **Path of Exile (1 e 2)**
que roda por cima do jogo em tela cheia sem borda. O usuário aperta uma hotkey sobre um item e um painel aparece
ao lado do cursor mostrando o preço do item. Ele usa isso dezenas de vezes por hora, no meio do gameplay —
então o painel precisa ser **instantaneamente legível, denso sem ser poluído, bonito e rápido**.

## Identidade visual

- **Conceito**: "forja" — ferramenta profissional de trader, grafite e brasa. Elegância sombria de ARPG, mas
  **moderno e limpo** (pense Linear / Raycast encontrando o universo de Wraeclast), nada de pergaminho,
  texturas pesadas ou fontes góticas ilegíveis. **NÃO usar estética de espelho/prata/cromado nem vidro translúcido.**
- **Tema**: dark only. Superfícies **sólidas** em grafite (`#0C0D0F` / `#141518` / `#1B1D21`), bordas finas de 1px,
  sombras profundas; legibilidade máxima sobre qualquer cena do jogo.
- **Acentos**: brasa/laranja (`~#FF8A3D`) para interação e foco; dourado quente (`~#FFD27A`) reservado a
  **preços**. Faixa fina no topo dos painéis na cor da raridade do item.
- **Layout do Price Check**: painel largo (~760px) em duas colunas — filtros/mods à esquerda, listagens à direita —
  com cabeçalho que mostra identidade do item à esquerda e **preço estimado grande à direita**, seguido de uma
  faixa de distribuição de preços (histograma mín → máx com marcador da mediana). Barra de comandos no rodapé.
- **Ícones de currency**: usar os ícones reais do jogo (já extraídos em `apps/desktop/public/assets/currency/`).
- **Cores semânticas do jogo** (obrigatórias, ajuste o tom para harmonizar e manter contraste AA sobre o vidro):
  Normal `#C8C8C8`, Magic `#8888FF`, Rare `#FFFF77`, Unique `#AF6025`, Gem `#1BA29B`, Currency `#AA9E82`,
  Divination `#0EBAFF`, Corrupted `#D20000`, Fractured `#A29162`, Crafted/Enchant `#B4B4FF`.
- **Tipografia**: Inter (UI) + JetBrains Mono ou similar para números/preços (tabular nums). Escala compacta
  (11 / 12 / 13 / 15 / 18 / 22).
- **Logo**: monograma "M" dentro de um hexágono em gradiente de brasa; versão 16px para a tray.

## Tokens (entregar como CSS custom properties)

Cores (surface 0–3, border, text primary/secondary/muted, accent, success/warning/danger, rarity.*),
radius (4/8/12/16), spacing (escala 4px), sombras/glow, blur, z-index, durações e easings de motion.
Nomeie como `--ms-color-surface-1`, `--ms-radius-md`, `--ms-motion-fast` etc.

## Motion (fluido, nunca lento)

- Entrada do painel: scale 0.96→1 + fade + slide de 8px **a partir do lado do cursor**, spring ~260 ms.
- Saída: 160 ms ease-in.
- Listas de resultados: stagger de 20–30 ms por linha.
- Loading: skeleton shimmer prateado; nunca spinner genérico no centro.
- Mudança de filtro → resultados fazem crossfade, sem pulo de layout.
- Hover/press: micro-interações de 100–120 ms.
- Tudo com fallback em `prefers-reduced-motion` (só fade).
- Descreva as animações com duração/easing exatos para eu implementar em Svelte.

## Telas e componentes

1. **Price Check (principal, ~460px de largura, altura até 80% da tela)**
   - Header do item: nome colorido pela raridade, base type, ilvl, item class, badges (corrupted, fractured,
     influências, links/sockets). Botão de fixar (pin) e fechar.
   - Barra de ações: toggle Online/Any, seletor de liga, link "abrir no site de trade", estado do rate limit.
   - Lista de mods com checkbox para incluir no filtro, valor rolado, campos min/max editáveis inline,
     tag do tipo (implicit/explicit/crafted/fractured/enchant), roll visualizado como mini-barra de percentil.
   - Filtros extras colapsáveis (ilvl, quality, sockets, gem level…).
   - Resultados: lista de listagens (preço com ícone da moeda, quantidade, tempo listado, conta, online/afk),
     resumo no topo (mín, mediana, nº de resultados), gráfico pequeno de tendência 7 dias (poe.ninja).
   - Variante **bulk** para currency/fragments: tabela "você dá × você recebe" com razão.
   - Estados: loading, vazio, erro de rede, **rate limited** (contador regressivo elegante), item não reconhecido.

2. **Quick Price (compacto, ~280px)**: tooltip flutuante perto do cursor para itens com preço no poe.ninja —
   ícone, nome, preço em chaos/divine, variação 24h (verde/vermelho), sparkline. Some sozinho.

3. **Settings** (janela maior, ~880×620, sidebar à esquerda):
   - Geral: jogo (PoE1/PoE2 com segmented control), liga, idioma, restaurar clipboard, abrir ao iniciar.
   - Hotkeys: lista de ações com **captura de tecla** (estado "pressione a combinação…"), validação inline
     (tecla reservada pelo jogo, duplicada), reset por linha.
   - Price check: filtros padrão, % de range dos mods, mostrar offline, POESESSID (campo secreto).
   - Client.txt: caminho do arquivo com botão procurar, teste de leitura, regras de alerta.
   - Aparência: opacidade do vidro, escala da UI, reduzir animações.
   - Sobre/atualizações.

4. **Feed do Client.txt** (painel lateral estreito): eventos tipados com ícone — whisper recebido/enviado,
   mudança de zona, trade aceito, morte, alerta custom. Filtro por tipo, botão "responder" em whispers.

5. **Componentes de sistema**: toasts (info/sucesso/aviso/erro), badge de hotkey (estilo keycap),
   tooltip, segmented control, toggle, input numérico compacto, select/combobox, botão (primary/ghost/danger,
   sm/md), scrollbar fina customizada, empty states com ilustração mínima.

6. **Onboarding (primeira execução, 3 passos)**: escolher jogo/liga → confirmar hotkeys → teste
   "aperte Ctrl+D em qualquer item". Aviso amigável se o jogo estiver rodando como administrador.

7. **Tray menu** e **ícone** da aplicação.

## Requisitos de UX

- Legibilidade sobre qualquer fundo do jogo (vidro + contraste AA mínimo para texto).
- Tudo operável por teclado; foco visível com o brilho prateado; Esc fecha/volta ao jogo.
- Números alinhados (tabular), preços sempre com ícone de moeda.
- Sem layout shift entre loading → resultado.
- Hierarquia clara: o **preço** é a primeira coisa que o olho encontra.
- Densidade de ferramenta profissional, mas com respiro.

## Entregáveis

1. Design system (tokens + componentes com todos os estados) como páginas/cards.
2. Mockups de alta fidelidade de todas as telas acima, incluindo estados (loading/erro/rate limit/vazio).
3. Especificação de motion por componente.
4. Export: **HTML/CSS com os tokens em CSS custom properties** — será convertido para componentes **Svelte 5**,
   então mantenha a estrutura semântica e evite dependências de framework.
