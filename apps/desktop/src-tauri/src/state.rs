use std::path::PathBuf;
use std::sync::Arc;

use mirrorsight_application::use_cases::{
    BuildService, ClipboardPolicy, CopyItemFromGame, HistoryService, ListLeagues, PresetService, PriceCheckItem,
    SaveSettings,
};
use mirrorsight_domain::evaluation::GameKnowledge;
use mirrorsight_domain::ports::{BuildRepository, GameWindowPort, ItemParser, SettingsRepository};
use mirrorsight_gamedata::LazyPoe1;
use mirrorsight_infrastructure::clipboard::SystemClipboard;
use mirrorsight_infrastructure::http;
use mirrorsight_infrastructure::json_file::{JsonHistoryRepository, JsonPresetRepository};
use mirrorsight_infrastructure::ninja::PoeNinjaIndex;
use mirrorsight_infrastructure::pob::PobBuildRepository;
use mirrorsight_infrastructure::settings::JsonSettingsRepository;
use mirrorsight_infrastructure::trade::PoeTradeApi;
use mirrorsight_infrastructure::win32::{Win32GameWindow, Win32Input};
use mirrorsight_parser::nameplate::looks_like_item;
use mirrorsight_parser::ClipboardItemParser;

/// Dependências compartilhadas pelos comandos. Trocar um adapter = mudar só este arquivo.
pub struct AppState {
    pub settings_repo: Arc<dyn SettingsRepository>,
    pub save_settings: SaveSettings,
    pub price_check: PriceCheckItem,
    pub list_leagues: ListLeagues,
    pub copy_item: CopyItemFromGame,
    pub game_window: Arc<dyn GameWindowPort>,
    pub history: HistoryService,
    pub presets: PresetService,
    pub builds: BuildService,
}

impl AppState {
    pub fn build(config_dir: PathBuf) -> Self {
        let settings_repo: Arc<dyn SettingsRepository> = Arc::new(JsonSettingsRepository::new(&config_dir));
        let parser: Arc<dyn ItemParser> = Arc::new(ClipboardItemParser::default());
        let client = http::client();
        // Uma instância só: o rate limiter precisa ver todas as requisições do app.
        let trade = Arc::new(PoeTradeApi::new(client.clone(), parser.clone()));
        // Base de conhecimento (snapshot do PoB) e builds salvos do PoB, para a avaliação de itens.
        // O parse dos JSON (~2,5 MB) fica fora da abertura: começa numa thread de fundo.
        LazyPoe1::warm_up();
        let knowledge: Arc<dyn GameKnowledge> = Arc::new(LazyPoe1);
        let builds: Arc<dyn BuildRepository> = Arc::new(PobBuildRepository::default());

        Self {
            save_settings: SaveSettings::new(settings_repo.clone()),
            price_check: PriceCheckItem::new(parser, trade.clone(), Arc::new(PoeNinjaIndex::new(client)), knowledge.clone())
                .with_builds(builds.clone()),
            list_leagues: ListLeagues::new(trade),
            copy_item: CopyItemFromGame::new(
                Arc::new(Win32Input),
                Arc::new(SystemClipboard),
                looks_like_item,
                ClipboardPolicy::default(),
            ),
            game_window: Arc::new(Win32GameWindow::default()),
            history: HistoryService::new(Arc::new(JsonHistoryRepository::new(&config_dir))),
            presets: PresetService::new(Arc::new(JsonPresetRepository::new(&config_dir))),
            builds: BuildService::new(builds, knowledge),
            settings_repo,
        }
    }
}
