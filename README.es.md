<div align="center">

<img src="apps/desktop/src-tauri/icons/128x128.png" width="96" alt="Logo de Mirrorsight" />

# Mirrorsight

**Un overlay de consulta de precios rápido y elegante para Path of Exile 1 y 2.**
Pasa el ratón sobre un objeto, pulsa <kbd>Ctrl</kbd> + <kbd>D</kbd> y descubre cuánto vale — sin salir del juego.

![Rust](https://img.shields.io/badge/Rust-1.80+-000000?logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)
![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white)
![Plataforma](https://img.shields.io/badge/plataforma-Windows-0078D6?logo=windows&logoColor=white)
![Licencia](https://img.shields.io/badge/licencia-MIT-green)
[![Download](https://img.shields.io/github/v/release/lucas3g/mirrorsight?label=download&logo=github&color=2ea44f)](https://github.com/lucas3g/mirrorsight/releases/latest)

[English](README.md) · [Português](README.pt-BR.md) · [中文](README.zh-CN.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · **Español**

<img src="docs/screenshots/es/price-check.png" alt="Panel de consulta de precios de Mirrorsight" width="100%" />

</div>

---

## ✨ ¿Qué es Mirrorsight?

Mirrorsight es un overlay ligero que se muestra encima de Path of Exile. Al pulsar la tecla rápida
sobre un objeto, lo lee, busca en el **sitio oficial de comercio** con filtros razonables y te muestra
— junto al cursor — el **precio estimado**, la **distribución de precios**, los **filtros de mods
editables** y los **anuncios en vivo**. Al terminar, <kbd>Esc</kbd> cierra el panel y devuelve el
foco al juego.

Está escrito en **Rust** (rápido, poca memoria, sin Electron) con una interfaz en **Svelte 5** sobre **Tauri 2**.

## 🚀 Funcionalidades

| | |
|---|---|
| 🎯 **Consulta con una tecla** | <kbd>Ctrl</kbd>+<kbd>D</kbd> sobre cualquier objeto abre el panel junto al cursor. |
| 📋 **Ctrl+C también sirve** | Con *Ctrl+C abre el price check* activado (por defecto), el atajo de copiar del propio juego lee el objeto con sus tiers y abre el overlay — el texto se sigue copiando. |
| 🧮 **Estimación inteligente** | Mínimo, mediana y máximo, más un indicador de confianza según la cantidad de anuncios. |
| 💡 **Precio sugerido** | Percentiles *Compra* (p25), *Justo* (p50) y *Venta* (p75), con la nota `~b/o` lista para copiar. |
| 🎛️ **Filtros de mods editables** | Activa/desactiva mods, ve su tier (T1…T*n*) y rango, ajusta mín/máx, usa el control de margen y añade mods por búsqueda de texto (pseudo incluidos). |
| 🧠 **Evaluación de objetos (PoE 1)** | Cada anuncio recibe dos lecturas independientes: **mejora** para tu build (Mejor / Peor / Equivalente, comparando el conjunto de stats, el tope de resistencias y los requisitos de atributos de tu build de Path of Building) y **calidad del objeto** (0–100 según tiers, rolls y potencial de crafteo; los únicos según sus rangos reales de roll). Los mods sin datos suficientes se marcan como *no concluyentes*, nunca como roll perfecto. Los anuncios se agrupan en Mejor / Equivalente / Peor y el icono del ojo muestra exactamente qué stats cambian. |
| 💾 **Presets** | Guarda tus combinaciones de filtros favoritas por juego y reutilízalas. |
| 📊 **Gráfico de distribución** | Histograma de precios con marcadores p25 / p50 / p75. |
| 💰 **Currency al por mayor** | La currency y los objetos apilables se valoran mediante el intercambio masivo (precio por unidad, stock, vendedor). |
| ⚡ **Precio rápido** | Precio instantáneo de **poe.ninja** con tendencia de 7 días — sin búsqueda en el comercio. |
| 🕘 **Historial** | Cada consulta se guarda y se puede reabrir con los mismos filtros. |
| ⌨️ **Paleta de comandos** | <kbd>Ctrl</kbd>+<kbd>K</kbd> para ir a cualquier sitio, cambiar de juego o liga, borrar el historial… |
| 🎮 **PoE 1 y PoE 2** | Cambia de juego y de liga desde la barra de título. |
| 🌐 **7 idiomas** | English, Português, Español, Deutsch, 简体中文, 日本語 y 한국어 — por defecto sigue el idioma del sistema. |
| 🛡️ **Respeta el rate limit** | Sigue las cabeceras `X-Rate-Limit-*` de la API oficial — sin baneos, sin spam. |

## 🔍 Cómo funciona

```mermaid
flowchart LR
    A[🎮 Ratón sobre el objeto<br/>Ctrl + D] --> B[📋 Copia el texto<br/>copia avanzada con tiers]
    B --> C[🧩 Parser<br/>texto → Item tipado]
    C --> D{Tipo de objeto}
    D -->|Equipo, gemas, mapas…| E[🔎 API oficial de comercio<br/>búsqueda + anuncios]
    D -->|Currency / apilables| F[💱 Intercambio masivo]
    D -->|Precio rápido| G[📈 poe.ninja]
    E --> H[🧮 Estadísticas<br/>mín · mediana · máx · p25/p50/p75]
    F --> H
    G --> I
    H --> I[🪟 Panel overlay<br/>junto al cursor]
```

1. **Captura** – con la ventana del juego enfocada, la tecla rápida lanza el atajo de "copiar objeto"
   del propio juego y lee el texto del portapapeles (el contenido anterior puede restaurarse automáticamente).
2. **Análisis** – el texto se convierte en un objeto tipado: rareza, base, nivel de objeto, calidad,
   engarces, influencias y cada modificador emparejado con la lista de stats de la API (con tier y rango).
3. **Búsqueda** – Mirrorsight construye la consulta con los mods relevantes (preseleccionados según tu
   configuración), respetando los límites de la API.
4. **Cálculo** – los anuncios se convierten a una moneda común (con la tasa del divine de poe.ninja)
   y se resumen: mín / mediana / máx, percentiles, confianza.
5. **Resultado** – el overlay se abre junto al cursor. Cambia cualquier filtro y la búsqueda se repite
   sola. <kbd>Esc</kbd> lo cierra y devuelve el foco al juego.

## 📸 Capturas

<table>
  <tr>
    <td width="50%"><img src="docs/screenshots/es/price-check.png" alt="Consulta de precio" /><br/><sub><b>Consulta de precio</b> — estimación, precio sugerido, filtros y anuncios agrupados por veredicto de mejora</sub></td>
    <td width="50%"><img src="docs/screenshots/es/evaluation.png" alt="Evaluación del objeto" /><br/><sub><b>Evaluación del objeto</b> — qué cambia para tu build y la calidad del objeto (icono del ojo)</sub></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/es/distribution.png" alt="Distribución" /><br/><sub><b>Distribución</b> — histograma con percentiles</sub></td>
    <td><img src="docs/screenshots/es/bulk-currency.png" alt="Currency al por mayor" /><br/><sub><b>Currency al por mayor</b> — ofertas con stock</sub></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/es/quick-price.png" alt="Precio rápido" /><br/><sub><b>Precio rápido</b> — valor de poe.ninja y tendencia de 7 días</sub></td>
    <td><img src="docs/screenshots/es/history.png" alt="Historial" /><br/><sub><b>Historial</b> — las últimas 50 consultas, reábrelas con los mismos filtros</sub></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/es/command-palette.png" alt="Paleta de comandos" /><br/><sub><b>Paleta de comandos</b> — <kbd>Ctrl</kbd>+<kbd>K</kbd></sub></td>
    <td><img src="docs/screenshots/es/settings-hotkeys.png" alt="Teclas rápidas" /><br/><sub><b>Teclas rápidas</b> — teclas rápidas reasignables</sub></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/es/settings-general.png" alt="Ajustes" /><br/><sub><b>Ajustes</b> — idioma de la interfaz, juego, liga, actualización automática</sub></td>
    <td><img src="docs/screenshots/es/settings-price-check.png" alt="Build de Path of Building" /><br/><sub><b>Build de Path of Building</b> — item set y skill set para el veredicto de mejora</sub></td>
  </tr>
</table>

> La interfaz está disponible en English, Português, Español, Deutsch, 简体中文, 日本語 y 한국어 (**Ajustes → General → Idioma de la interfaz**; por defecto sigue el idioma del sistema). Las capturas se tomaron en el modo demo integrado.

## 📦 Primeros pasos

1. **Instala** — descarga `Mirrorsight_x.y.z_x64-setup.exe` de la [última release](https://github.com/lucas3g/mirrorsight/releases/latest) y ejecútalo
   (o compílalo tú mismo, ver [Desarrollo](#-desarrollo)). La app vive en la bandeja del sistema.
2. **Configura** — en **Ajustes → General**, elige el juego (PoE 1 / PoE 2) y la liga.
3. **Juega** — pasa el ratón sobre un objeto y pulsa <kbd>Ctrl</kbd>+<kbd>D</kbd>. <kbd>Esc</kbd> cierra.

¿Sin el juego abierto? Pega el texto de un objeto en la ventana principal con <kbd>Ctrl</kbd>+<kbd>V</kbd>
(o <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>V</kbd> para el precio rápido).

### ⌨️ Teclas rápidas por defecto

| Tecla | Acción |
|---|---|
| <kbd>Ctrl</kbd> + <kbd>D</kbd> | Consultar el precio del objeto bajo el cursor |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>D</kbd> | Consulta ya fijada en pantalla |
| <kbd>Ctrl</kbd> + <kbd>C</kbd> | Consulta con tiers; el texto se sigue copiando (se puede desactivar en **Ajustes → General**) |
| <kbd>Shift</kbd> + <kbd>Space</kbd> | Mostrar / ocultar el overlay |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>O</kbd> | Abrir ajustes |
| <kbd>Ctrl</kbd> + <kbd>K</kbd> | Paleta de comandos (dentro de Mirrorsight) |
| <kbd>Esc</kbd> | Cerrar el panel y devolver el foco al juego |

Todas las teclas globales se pueden reasignar en **Ajustes → Hotkeys**. Solo se registran mientras
la ventana del juego está enfocada, así que nunca roban teclas a otras aplicaciones.

> **Consejo:** si Path of Exile se ejecuta como administrador, ejecuta Mirrorsight también como
> administrador — Windows bloquea el envío de teclas de un proceso normal a uno elevado.

### ⚙️ Ajustes

- **General** — idioma de la interfaz, juego, liga, idioma del cliente, moneda de visualización, restaurar portapapeles, Ctrl+C abre el price check, iniciar con Windows, actualizar automáticamente.
- **Consulta de precio** — rango de mods, mostrar vendedores desconectados, filtros marcados por defecto, nivel por defecto del precio sugerido, build de Path of Building (con item set y skill set) para el veredicto de mejora, historial, presets.
- **Apariencia** — escala de la interfaz (90–130 %), densidad, reducir animaciones.

### 🔁 Actualizaciones automáticas

Desde la v0.4.0, Mirrorsight se actualiza solo desde las Releases de GitHub. Al abrir la app, una versión nueva se instala de inmediato (una pequeña barra de progreso y se vuelve a abrir). Si sale una versión mientras juegas, se descarga en segundo plano y aparece el botón **Reiniciar para actualizar** en la barra de estado y en el menú de la bandeja; nada se cierra sin tu clic. Puedes desactivarlo en **Ajustes → General**. Las versiones 0.3.0 y anteriores necesitan una instalación manual.

## 🛠 Desarrollo

Requisitos: **Rust ≥ 1.80**, **Node ≥ 20**, Windows 10/11 (WebView2).

```sh
cd apps/desktop
npm install
npm run dev          # solo la UI, en el navegador, con datos de demo
npx tauri dev        # la app de escritorio completa
npx tauri build      # instalador NSIS en <raíz>/target/release/bundle/nsis
```

Tests y lint:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
cd apps/desktop && npm run check
# contra las APIs reales (consume rate limit — úsalo con moderación):
cargo test -p mirrorsight-infrastructure --test live -- --ignored --nocapture
```

Capturas de los READMEs: `node tools/screenshots/capture.mjs` compila la demo y captura todos los idiomas en `docs/screenshots/<idioma>/` (Node ≥ 22 y Edge o Chrome; `--locales en,es` limita los idiomas).

### 🧱 Arquitectura (clean architecture)

```
crates/domain          entidades, reglas puras y ports (traits). Sin I/O.
crates/parser          texto de Ctrl+C / Ctrl+Alt+C → Item (mods emparejados con los stats de la API)
crates/application     casos de uso (consulta de precio, copiar objeto, ajustes, ligas)
crates/infrastructure  adaptadores: API de comercio + rate limiter, poe.ninja, Win32, portapapeles, ajustes, builds de PoB
crates/gamedata        base de conocimiento de PoE 1 integrada (únicos, tiers, bases, gemas) + reglas de evaluación
apps/desktop/src-tauri composition root, comandos, teclas rápidas, overlay, bandeja
apps/desktop/src       UI en Svelte 5; habla con el backend solo a través del port `Backend`
                       (adaptadores: Tauri real y demo para el navegador)
```

### 🔄 Mantenimiento en cada liga

- **Stats de la API de comercio** (el parser los necesita para reconocer mods):
  `node tools/stats/update-stats.mjs` → actualiza `crates/parser/data/stats-*.tsv`.
- **Base de conocimiento de objetos** (rangos de roll de únicos, variantes, tiers de mods, bases, tags de gemas) desde la última
  versión de Path of Building: `node tools/gamedata/update-gamedata.mjs` → actualiza `crates/gamedata/data/*.json`
  (`--local "<carpeta de PoB>"` lee una instalación local). Los pesos, sinergias y umbrales están en
  `crates/gamedata/data/rules-poe1.json` y se editan a mano. Después ejecuta `cargo test`.
- **Iconos de currency**: extraídos de la instalación local del juego a
  `apps/desktop/public/assets/currency/` (PNG + `manifest.json`).

## ⚠️ Aviso

Mirrorsight es un proyecto independiente y **no está afiliado ni respaldado por Grinding Gear Games**.
Arte de objetos y currency © Grinding Gear Games. Usa las APIs públicas de pathofexile.com y respeta
los límites indicados en las cabeceras `X-Rate-Limit-*`. No lee la memoria del juego ni automatiza el
juego — solo usa el atajo de copiar objeto del propio juego.

## 📄 Licencia

[MIT](LICENSE)
