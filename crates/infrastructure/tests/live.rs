//! Testes contra as APIs reais (pathofexile.com e poe.ninja). Ignorados por padrão:
//!
//!     cargo test -p mirrorsight-infrastructure --test live -- --ignored --nocapture
//!
//! Consomem rate limit real — rode com moderação.

use std::sync::Arc;

use mirrorsight_application::use_cases::PriceCheckItem;
use mirrorsight_domain::ports::{ItemParser, LeaguesPort};
use mirrorsight_domain::settings::Settings;
use mirrorsight_domain::GameVersion;
use mirrorsight_infrastructure::http;
use mirrorsight_infrastructure::ninja::PoeNinjaIndex;
use mirrorsight_infrastructure::trade::PoeTradeApi;
use mirrorsight_parser::ClipboardItemParser;

const GLOVES: &str = "Item Class: Gloves
Rarity: Rare
Doom Grip
Sorcerer Gloves
--------
Item Level: 86
--------
+70 to maximum Life
+40% to Fire Resistance
";

#[tokio::test]
#[ignore = "acessa a rede"]
async fn ninja_prices_a_unique() {
    use mirrorsight_domain::ports::PriceIndexPort;
    use mirrorsight_domain::League;

    let text = "Item Class: Body Armours\nRarity: Unique\nKaom's Heart\nGlorious Plate\n--------\nItem Level: 80\n";
    let item = ClipboardItemParser::default().parse(GameVersion::Poe1, text).expect("parse");
    let ninja = PoeNinjaIndex::new(http::client());
    let price = ninja.lookup(&League::new("Covens", GameVersion::Poe1), &item).await.expect("poe.ninja");
    println!("Kaom's Heart: {price:?}");
    assert!(price.is_some_and(|p| p.chaos_value > 0.0));
}

#[tokio::test]
#[ignore = "acessa a rede"]
async fn price_check_against_real_apis() {
    let parser: Arc<dyn ItemParser> = Arc::new(ClipboardItemParser::default());
    let client = http::client();
    let trade = Arc::new(PoeTradeApi::new(client.clone(), parser.clone()));

    let leagues = trade.leagues(GameVersion::Poe1).await.expect("ligas");
    println!("ligas PoE1: {leagues:?}");
    let league = leagues.first().cloned().expect("ao menos uma liga");

    let use_case = PriceCheckItem::new(parser, trade, Arc::new(PoeNinjaIndex::new(client)));
    let settings = Settings { league: league.clone(), ..Settings::default() };
    let out = use_case.execute(&settings, GLOVES, None).await.expect("price check");

    let trade = out.trade.expect("resultado do trade");
    println!("liga={league} total={} listagens={} url={:?}", trade.total, trade.listings.len(), trade.web_url);
    for l in trade.listings.iter().take(5) {
        let mods = l.item.as_ref().map_or(0, |i| i.modifiers.len());
        println!("  {} {} — {} (online={}, mods parseados={mods})", l.price.amount, l.price.currency.0, l.account, l.online);
    }
    println!("resumo: {:?}", out.summary);

    assert!(trade.total > 0, "esperava listagens para luvas com life+fire res");
    assert!(!trade.listings.is_empty());
    assert!(trade.listings.iter().any(|l| l.item.as_ref().is_some_and(|i| !i.modifiers.is_empty())),
        "itens listados deveriam ser parseados com mods");
    let summary = out.summary.expect("resumo com cotações do poe.ninja");
    assert!(summary.divine_chaos.is_some_and(|d| d > 1.0), "cotação do divine");
}

/// Categorias que já falharam: Foulborn, gema transfigurada, carta, base "Superior".
#[tokio::test]
#[ignore = "acessa a rede"]
async fn price_check_tricky_items() {
    let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/../parser/tests/fixtures/");
    let read = |f: &str| std::fs::read_to_string(format!("{fixtures}{f}")).expect("fixture");
    let items = [
        ("foulborn", read("poe1_foulborn_unique.txt")),
        ("gema transfigurada", read("poe1_transfigured_gem.txt")),
        ("carta", "Item Class: Divination Cards\nRarity: Divination Card\nThe Doctor\n--------\nStack Size: 1/8\n".into()),
        ("superior", "Item Class: Shields\nRarity: Normal\nSuperior Champion Kite Shield\n--------\nQuality: +20% (augmented)\n--------\nItem Level: 84\n".into()),
        ("mapa blighted", read("poe1_blighted_map.txt")),
        ("mapa blighted sem área", "Item Class: Maps\nRarity: Normal\nBlighted Map (Tier 13)\n--------\nMap Tier: 13\n".into()),
        ("mapa mágico", "Item Class: Maps\nRarity: Magic\nSavage Map (Tier 16) of Fog\n--------\nMap Area: Strand\nMap Tier: 16\n".into()),
    ];

    let parser: Arc<dyn ItemParser> = Arc::new(ClipboardItemParser::default());
    let client = http::client();
    let trade = Arc::new(PoeTradeApi::new(client.clone(), parser.clone()));
    let league = trade.leagues(GameVersion::Poe1).await.expect("ligas").first().cloned().expect("liga");
    let use_case = PriceCheckItem::new(parser, trade, Arc::new(PoeNinjaIndex::new(client)));
    let settings = Settings { league, show_offline: true, ..Settings::default() };

    let mut failures = Vec::new();
    for (label, text) in &items {
        match use_case.execute(&settings, text, None).await {
            Ok(out) => {
                let total = out.trade.as_ref().map(|t| t.total);
                println!("{label}: trade total={total:?} erro={:?} ninja={:?}",
                    out.trade_error, out.indexed.as_ref().map(|i| (&i.name, i.chaos_value)));
                if out.trade.is_none() {
                    failures.push(format!("{label}: {:?}", out.trade_error));
                }
            }
            Err(err) => failures.push(format!("{label}: {err}")),
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Busca + fetch de até `n` itens crus (JSON da trade API) para conferir o formato.
async fn raw_items(client: &reqwest::Client, league: &str, query: serde_json::Value, n: usize) -> Vec<serde_json::Value> {
    let mut url = reqwest::Url::parse("https://www.pathofexile.com/api/trade/search").expect("url");
    url.path_segments_mut().expect("segmentos").push(league);
    let search: serde_json::Value =
        client.post(url).json(&query).send().await.expect("search").json().await.expect("json");
    let ids: Vec<String> = search["result"]
        .as_array()
        .map(|a| a.iter().take(n).filter_map(|v| v.as_str().map(str::to_owned)).collect())
        .unwrap_or_default();
    if ids.is_empty() {
        println!("busca sem resultados: {search}");
        return vec![];
    }
    let id = search["id"].as_str().expect("id");
    let fetch = format!("https://www.pathofexile.com/api/trade/fetch/{}?query={id}", ids.join(","));
    let body: serde_json::Value = client.get(fetch).send().await.expect("fetch").json().await.expect("json");
    body["result"].as_array().cloned().unwrap_or_default().into_iter().filter(|v| !v.is_null()).map(|e| e["item"].clone()).collect()
}

/// Imprime o formato real dos mods de anúncios (PoE1): onde ficam fractured/crafted, o tier,
/// e as magnitudes de mods híbridos (cada linha traz só as do próprio stat). Base do `enrich` do DTO.
#[tokio::test]
#[ignore = "acessa a rede"]
async fn trade_extended_mods_shape() {
    let client = http::client();
    let trade = PoeTradeApi::new(client.clone(), Arc::new(ClipboardItemParser::default()));
    let league = trade.leagues(GameVersion::Poe1).await.expect("ligas").first().cloned().expect("liga");
    let rare = |category: &str, misc: serde_json::Value, stats: serde_json::Value| {
        serde_json::json!({
            "query": {
                "status": {"option": "online"},
                "stats": stats,
                "filters": {
                    "type_filters": {"filters": {"rarity": {"option": "rare"}, "category": {"option": category}}},
                    "misc_filters": {"filters": misc}
                }
            },
            "sort": {"price": "asc"}
        })
    };

    // 1) Fractured + crafted: listas separadas (`fracturedMods`/`craftedMods`) ou dentro de `explicitMods`?
    let q = rare("armour.gloves", serde_json::json!({"crafted": {"option": "true"}, "fractured_item": {"option": "true"}}), serde_json::json!([]));
    for item in raw_items(&client, &league, q, 2).await {
        let keys: Vec<&String> = item.as_object().map(|o| o.keys().collect()).unwrap_or_default();
        println!("item keys: {keys:?}");
        for field in ["explicitMods", "fracturedMods", "craftedMods"] {
            println!("{field}: {}", item[field]);
        }
    }

    // 2) Híbridos: linhas que compartilham o mesmo mod (nome + tier) e suas magnitudes.
    let q = rare("armour.chest", serde_json::json!({}), serde_json::json!([]));
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    for item in raw_items(&client, &league, q, 10).await {
        let lines = item["explicitMods"].as_array().cloned().unwrap_or_default();
        for (i, a) in lines.iter().enumerate() {
            for b in lines.iter().skip(i + 1) {
                let (ma, mb) = (&a["mods"][0], &b["mods"][0]);
                if ma["name"] == mb["name"] && ma["tier"] == mb["tier"] {
                    println!("HÍBRIDO?\n  {a}\n  {b}");
                }
            }
        }
    }
}
