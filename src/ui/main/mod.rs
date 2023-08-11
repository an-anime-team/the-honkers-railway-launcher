use relm4::{
    prelude::*,
    component::*,
    actions::*,
    MessageBroker
};

use gtk::prelude::*;
use adw::prelude::*;

use gtk::glib::clone;

mod update_patch;
mod download_wine;
mod create_prefix;
mod download_diff;
mod disable_telemetry;
mod launch;

use anime_launcher_sdk::components::loader::ComponentsLoader;

use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::star_rail::config::Config;

use anime_launcher_sdk::star_rail::config::schema::launcher::LauncherStyle;

use anime_launcher_sdk::star_rail::states::*;
use anime_launcher_sdk::star_rail::consts::*;

use crate::*;
use crate::ui::components::*;

use super::preferences::main::*;
use super::about::*;

relm4::new_action_group!(WindowActionGroup, "win");

relm4::new_stateless_action!(LauncherFolder, WindowActionGroup, "launcher_folder");
relm4::new_stateless_action!(GameFolder, WindowActionGroup, "game_folder");
relm4::new_stateless_action!(ConfigFile, WindowActionGroup, "config_file");
relm4::new_stateless_action!(DebugFile, WindowActionGroup, "debug_file");
relm4::new_stateless_action!(WishUrl, WindowActionGroup, "wish_url");

relm4::new_stateless_action!(About, WindowActionGroup, "about");

pub static mut MAIN_WINDOW: Option<adw::ApplicationWindow> = None;
pub static mut PREFERENCES_WINDOW: Option<AsyncController<PreferencesApp>> = None;
pub static mut ABOUT_DIALOG: Option<Controller<AboutDialog>> = None;

pub struct App {
    progress_bar: AsyncController<ProgressBar>,

    toast_overlay: adw::ToastOverlay,

    loading: Option<Option<String>>,
    style: LauncherStyle,
    state: Option<LauncherState>,

    downloading: bool,
    disabled_buttons: bool,
    kill_game_button: bool,
    disabled_kill_game_button: bool
}

#[derive(Debug)]
pub enum AppMsg {
    UpdateLauncherState {
        /// Perform action when game or voice downloading is required
        /// Needed for chained executions (e.g. update one voice after another)
        perform_on_download_needed: bool,

        /// Show status gathering progress page
        show_status_page: bool
    },

    /// Supposed to be called automatically on app's run when the latest game version
    /// was retrieved from the API
    SetGameDiff(Option<VersionDiff>),

    /// Supposed to be called automatically on app's run when the latest main patch version
    /// was retrieved from remote repos
    SetMainPatch(Option<(Version, JadeitePatchStatusVariant)>),

    /// Supposed to be called automatically on app's run when the launcher state was chosen
    SetLauncherState(Option<LauncherState>),

    SetLauncherStyle(LauncherStyle),
    SetLoadingStatus(Option<Option<String>>),

    SetDownloading(bool),
    DisableButtons(bool),
    SetKillGameButton(bool),
    DisableKillGameButton(bool),

    OpenPreferences,

    PredownloadUpdate,
    PerformAction,

    HideWindow,
    ShowWindow,

    Toast {
        title: String,
        description: Option<String>
    }
}

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    menu! {
        main_menu: {
            section! {
                &tr!("launcher-folder") => LauncherFolder,
                &tr!("game-folder") => GameFolder,
                &tr!("config-file") => ConfigFile,
                &tr!("debug-file") => DebugFile,
            },

            section! {
                &tr!("wish-url") => WishUrl
            },

            section! {
                &tr!("about") => About
            }
        }
    }

    view! {
        main_window = adw::ApplicationWindow {
            set_icon_name: Some(APP_ID),

            #[watch]
            set_default_size: (
                match model.style {
                    LauncherStyle::Modern => 900,
                    LauncherStyle::Classic => 1094 // (w = 1280 / 730 * h, where 1280x730 is default background picture resolution)
                },
                match model.style {
                    LauncherStyle::Modern => 600,
                    LauncherStyle::Classic => 624
                }
            ),

            #[watch]
            set_css_classes: &{
                let mut classes = vec!["background", "csd"];

                if APP_DEBUG {
                    classes.push("devel");
                }

                match model.style {
                    LauncherStyle::Modern => (),
                    LauncherStyle::Classic => {
                        if model.loading.is_none() {
                            classes.push("classic-style");
                        }
                    }
                }

                classes
            },

            #[local_ref]
            toast_overlay -> adw::ToastOverlay {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    adw::HeaderBar {
                        #[watch]
                        set_css_classes: match model.style {
                            LauncherStyle::Modern => &[""],
                            LauncherStyle::Classic => &["flat"]
                        },

                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle {
                            #[watch]
                            set_title: match model.style {
                                LauncherStyle::Modern => "The Honkers Railway Launcher",
                                LauncherStyle::Classic => ""
                            }
                        },

                        pack_end = &gtk::MenuButton {
                            set_icon_name: "open-menu-symbolic",
                            set_menu_model: Some(&main_menu)
                        }
                    },

                    adw::StatusPage {
                        set_title: &tr!("loading-data"),
                        set_icon_name: Some(APP_ID),
                        set_vexpand: true,

                        #[watch]
                        set_description: match &model.loading {
                            Some(Some(desc)) => Some(desc),
                            Some(None) | None => None
                        },

                        #[watch]
                        set_visible: model.loading.is_some()
                    },

                    adw::PreferencesPage {
                        #[watch]
                        set_visible: model.loading.is_none(),

                        add = &adw::PreferencesGroup {
                            set_margin_top: 48,

                            #[watch]
                            set_visible: model.style == LauncherStyle::Modern,

                            gtk::Picture {
                                set_resource: Some(&format!("{APP_RESOURCE_PATH}/icons/hicolor/scalable/apps/{APP_ID}.png")),
                                set_vexpand: true,
                                set_content_fit: gtk::ContentFit::ScaleDown
                            },

                            gtk::Label {
                                set_label: "The Honkers Railway Launcher",
                                set_margin_top: 32,
                                add_css_class: "title-1"
                            }
                        },

                        add = &adw::PreferencesGroup {
                            #[watch]
                            set_valign: match model.style {
                                LauncherStyle::Modern => gtk::Align::Center,
                                LauncherStyle::Classic => gtk::Align::End
                            },

                            #[watch]
                            set_width_request: match model.style {
                                LauncherStyle::Modern => -1,
                                LauncherStyle::Classic => 800
                            },

                            #[watch]
                            set_visible: model.downloading,

                            set_vexpand: true,
                            set_margin_top: 48,
                            set_margin_bottom: 48,

                            add = model.progress_bar.widget(),
                        },

                        add = &adw::PreferencesGroup {
                            #[watch]
                            set_valign: match model.style {
                                LauncherStyle::Modern => gtk::Align::Center,
                                LauncherStyle::Classic => gtk::Align::End
                            },

                            #[watch]
                            set_width_request: match model.style {
                                LauncherStyle::Modern => -1,
                                LauncherStyle::Classic => 800
                            },

                            #[watch]
                            set_visible: !model.downloading,

                            #[watch]
                            set_margin_bottom: match model.style {
                                LauncherStyle::Modern => 48,
                                LauncherStyle::Classic => 0
                            },

                            set_vexpand: true,

                            gtk::Box {
                                #[watch]
                                set_halign: match model.style {
                                    LauncherStyle::Modern => gtk::Align::Center,
                                    LauncherStyle::Classic => gtk::Align::End
                                },

                                #[watch]
                                set_height_request: match model.style {
                                    LauncherStyle::Modern => -1,
                                    LauncherStyle::Classic => 40
                                },

                                set_margin_top: 64,
                                set_spacing: 8,

                                adw::Bin {
                                    set_css_classes: &["background", "round-bin"],

                                    gtk::Button {
                                        set_width_request: 44,

                                        #[watch]
                                        set_tooltip_text: Some(&tr!("predownload-update", {
                                            "version" = match model.state.as_ref() {
                                                Some(LauncherState::PredownloadAvailable { diff, .. }) => diff.latest().to_string(),
                                                _ => String::from("?")
                                            },

                                            "size" = match model.state.as_ref() {
                                                Some(LauncherState::PredownloadAvailable { diff, .. }) => prettify_bytes(diff.downloaded_size().unwrap_or(0)),
                                                _ => String::from("?")
                                            }
                                        })),

                                        #[watch]
                                        set_visible: matches!(model.state.as_ref(), Some(LauncherState::PredownloadAvailable { .. })),

                                        #[watch]
                                        set_sensitive: match model.state.as_ref() {
                                            Some(LauncherState::PredownloadAvailable { diff, .. }) => {
                                                let config = Config::get().unwrap();
                                                let temp = config.launcher.temp.unwrap_or_else(std::env::temp_dir);

                                                !temp.join(diff.file_name().unwrap()).metadata()
                                                    .map(|metadata| Some(metadata.len()) == diff.downloaded_size())
                                                    .unwrap_or(false)
                                            }

                                            _ => false
                                        },

                                        #[watch]
                                        set_css_classes: match model.state.as_ref() {
                                            Some(LauncherState::PredownloadAvailable { diff, .. }) => {
                                                let config = Config::get().unwrap();
                                                let temp = config.launcher.temp.unwrap_or_else(std::env::temp_dir);

                                                let downloaded = temp.join(diff.file_name().unwrap()).metadata()
                                                    .map(|metadata| Some(metadata.len()) == diff.downloaded_size())
                                                    .unwrap_or(false);

                                                if downloaded {
                                                    &["success", "circular"]
                                                } else {
                                                    &["warning", "circular"]
                                                }
                                            }

                                            _ => &["warning", "circular"]
                                        },

                                        set_icon_name: "document-save-symbolic",
                                        set_hexpand: false,

                                        connect_clicked => AppMsg::PredownloadUpdate
                                    }
                                },

                                adw::Bin {
                                    set_css_classes: &["background", "round-bin"],

                                    #[watch]
                                    set_visible: !model.kill_game_button,

                                    gtk::Button {
                                        adw::ButtonContent {
                                            #[watch]
                                            set_icon_name: match &model.state {
                                                Some(LauncherState::Launch) |
                                                Some(LauncherState::PatchNotVerified) |
                                                Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Verified, .. }) |
                                                Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Unverified, .. })
                                                    => "media-playback-start-symbolic",

                                                Some(LauncherState::PatchNotInstalled) |
                                                Some(LauncherState::PatchUpdateAvailable) => "document-save-symbolic",

                                                Some(LauncherState::TelemetryNotDisabled) => "security-high-symbolic",

                                                Some(LauncherState::WineNotInstalled) |
                                                Some(LauncherState::PrefixNotExists) => "document-save-symbolic",

                                                Some(LauncherState::GameUpdateAvailable(_)) |
                                                Some(LauncherState::GameNotInstalled(_)) => "document-save-symbolic",

                                                Some(LauncherState::GameOutdated(_)) |
                                                Some(LauncherState::PatchBroken) |
                                                Some(LauncherState::PatchUnsafe) |
                                                Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Broken, .. }) |
                                                Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Unsafe, .. }) |
                                                None => "window-close-symbolic"
                                            },

                                            #[watch]
                                            set_label: &match &model.state {
                                                Some(LauncherState::Launch) |
                                                Some(LauncherState::PatchNotVerified) |
                                                Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Verified, .. }) |
                                                Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Unverified, .. })
                                                    => tr!("launch"),

                                                Some(LauncherState::PatchNotInstalled) |
                                                Some(LauncherState::PatchUpdateAvailable) => tr!("download-patch"),

                                                Some(LauncherState::PatchBroken) |
                                                Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Broken, .. })
                                                    => tr!("patch-broken"),

                                                Some(LauncherState::PatchUnsafe) |
                                                Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Unsafe, .. }) 
                                                    => tr!("patch-unsafe"),

                                                Some(LauncherState::TelemetryNotDisabled) => tr!("disable-telemetry"),

                                                Some(LauncherState::WineNotInstalled) => tr!("download-wine"),
                                                Some(LauncherState::PrefixNotExists)  => tr!("create-prefix"),

                                                Some(LauncherState::GameUpdateAvailable(diff)) |
                                                Some(LauncherState::GameOutdated(diff)) => {
                                                    match (Config::get(), diff.file_name()) {
                                                        (Ok(config), Some(filename)) => {
                                                            let temp = config.launcher.temp.unwrap_or_else(std::env::temp_dir);

                                                            if temp.join(filename).exists() {
                                                                tr!("resume")
                                                            }

                                                            else {
                                                                tr!("update")
                                                            }
                                                        }

                                                        _ => tr!("update")
                                                    }
                                                },

                                                Some(LauncherState::GameNotInstalled(_)) => tr!("download"),

                                                None => String::from("...")
                                            }
                                        },

                                        #[watch]
                                        set_sensitive: !model.disabled_buttons && match &model.state {
                                            Some(LauncherState::GameOutdated { .. }) |
                                            Some(LauncherState::PatchBroken) |
                                            Some(LauncherState::PatchUnsafe) |
                                            Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Broken, .. }) |
                                            Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Unsafe, .. }) => false,

                                            Some(_) => true,

                                            None => false
                                        },

                                        #[watch]
                                        set_css_classes: match &model.state {
                                            Some(LauncherState::GameOutdated { .. }) |
                                            Some(LauncherState::PatchNotVerified) => &["warning", "pill"],

                                            Some(LauncherState::PatchBroken) |
                                            Some(LauncherState::PatchUnsafe) |
                                            Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Broken, .. }) |
                                            Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Unsafe, .. }) => &["error", "pill"],

                                            Some(_) => &["suggested-action", "pill"],

                                            None => &["pill"]
                                        },

                                        #[watch]
                                        set_tooltip_text: Some(&match &model.state {
                                            Some(LauncherState::GameOutdated { .. }) => tr!("main-window--version-outdated-tooltip"),

                                            Some(LauncherState::PatchNotVerified) => tr!("patch-testing-tooltip"),

                                            Some(LauncherState::PatchBroken) |
                                            Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Broken, .. })
                                                => tr!("patch-broken-tooltip"),

                                            Some(LauncherState::PatchUnsafe) |
                                            Some(LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Unsafe, .. })
                                                => tr!("patch-unsafe-tooltip"),

                                            _ => String::new()
                                        }),

                                        set_hexpand: false,
                                        set_width_request: 200,

                                        connect_clicked => AppMsg::PerformAction
                                    }
                                },

                                adw::Bin {
                                    set_css_classes: &["background", "round-bin"],

                                    #[watch]
                                    set_visible: model.kill_game_button,

                                    gtk::Button {
                                        adw::ButtonContent {
                                            set_icon_name: "violence-symbolic", // window-close-symbolic
                                            set_label: &tr!("kill-game-process")
                                        },

                                        #[watch]
                                        set_sensitive: !model.disabled_kill_game_button,

                                        set_css_classes: &["error", "pill"],

                                        set_hexpand: false,
                                        set_width_request: 200,

                                        connect_clicked[sender] => move |_| {
                                            sender.input(AppMsg::DisableKillGameButton(true));

                                            std::thread::spawn(clone!(@strong sender => move || {
                                                std::thread::sleep(std::time::Duration::from_secs(3));

                                                sender.input(AppMsg::DisableKillGameButton(false));
                                            }));

                                            let result = std::process::Command::new("pkill")
                                                .arg("StarRail\\.exe")
                                                .spawn();

                                            if let Err(err) = result {
                                                sender.input(AppMsg::Toast {
                                                    title: tr!("kill-game-process-failed"),
                                                    description: Some(err.to_string())
                                                });
                                            }

                                            // Doesn't work on all the systems
                                            // e.g. won't work if you didn't install wine system-wide
                                            // there's some reasons for it

                                            // match Config::get() {
                                            //     Ok(config) => {
                                            //         match config.get_selected_wine() {
                                            //             Ok(Some(version)) => {
                                            //                 use anime_launcher_sdk::wincompatlib::prelude::*;

                                            //                 let result = version.to_wine(config.components.path, Some(config.game.wine.builds.join(&version.name)))
                                            //                     .stop_processes(false);

                                            //                 if let Err(err) = result {
                                            //                     sender.input(AppMsg::Toast {
                                            //                         title: tr!("kill-game-process-failed"),
                                            //                         description: Some(err.to_string())
                                            //                     });
                                            //                 }
                                            //             }

                                            //             Ok(None) => {
                                            //                 sender.input(AppMsg::Toast {
                                            //                     title: tr!("failed-get-selected-wine"),
                                            //                     description: None
                                            //                 });
                                            //             }

                                            //             Err(err) => {
                                            //                 sender.input(AppMsg::Toast {
                                            //                     title: tr!("failed-get-selected-wine"),
                                            //                     description: Some(err.to_string())
                                            //                 });
                                            //             }
                                            //         }
                                            //     }

                                            //     Err(err) => {
                                            //         sender.input(AppMsg::Toast {
                                            //             title: tr!("config-file-opening-error"),
                                            //             description: Some(err.to_string())
                                            //         });
                                            //     }
                                            // }
                                        }
                                    }
                                },

                                adw::Bin {
                                    set_css_classes: &["background", "round-bin"],

                                    gtk::Button {
                                        #[watch]
                                        set_sensitive: !model.disabled_buttons,

                                        set_width_request: 44,

                                        add_css_class: "circular",
                                        set_icon_name: "emblem-system-symbolic",

                                        connect_clicked => AppMsg::OpenPreferences
                                    }
                                }
                            }
                        }
                    }
                }
            },

            connect_close_request[sender] => move |_| {
                if let Err(err) = Config::flush() {
                    sender.input(AppMsg::Toast {
                        title: tr!("config-update-error"),
                        description: Some(err.to_string())
                    });
                }

                gtk::Inhibit::default()
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        tracing::info!("Initializing main window");

        let model = App {
            progress_bar: ProgressBar::builder()
                .launch(ProgressBarInit {
                    caption: None,
                    display_progress: true,
                    display_fraction: true,
                    visible: true
                })
                .detach(),

            toast_overlay: adw::ToastOverlay::new(),

            loading: Some(None),
            style: CONFIG.launcher.style,
            state: None,

            downloading: false,
            disabled_buttons: false,
            kill_game_button: false,
            disabled_kill_game_button: false
        };

        model.progress_bar.widget().set_halign(gtk::Align::Center);
        model.progress_bar.widget().set_width_request(360);

        let toast_overlay = &model.toast_overlay;

        let widgets = view_output!();

        let about_dialog_broker: MessageBroker<AboutDialogMsg> = MessageBroker::new();

        unsafe {
            MAIN_WINDOW = Some(widgets.main_window.clone());

            PREFERENCES_WINDOW = Some(PreferencesApp::builder()
                .launch(widgets.main_window.clone().into())
                .forward(sender.input_sender(), std::convert::identity));

            ABOUT_DIALOG = Some(AboutDialog::builder()
                .transient_for(widgets.main_window.clone())
                .launch_with_broker((), &about_dialog_broker)
                .detach());
        }

        let mut group = RelmActionGroup::<WindowActionGroup>::new();

        // TODO: reduce code somehow

        group.add_action::<LauncherFolder>(RelmAction::new_stateless(clone!(@strong sender => move |_| {
            if let Err(err) = open::that(LAUNCHER_FOLDER.as_path()) {
                sender.input(AppMsg::Toast {
                    title: tr!("launcher-folder-opening-error"),
                    description: Some(err.to_string())
                });

                tracing::error!("Failed to open launcher folder: {err}");
            }
        })));

        group.add_action::<GameFolder>(RelmAction::new_stateless(clone!(@strong sender => move |_| {
            let path = match Config::get() {
                Ok(config) => config.game.path.for_edition(config.launcher.edition).to_path_buf(),
                Err(_) => CONFIG.game.path.for_edition(CONFIG.launcher.edition).to_path_buf()
            };

            if let Err(err) = open::that(path) {
                sender.input(AppMsg::Toast {
                    title: tr!("game-folder-opening-error"),
                    description: Some(err.to_string())
                });

                tracing::error!("Failed to open game folder: {err}");
            }
        })));

        group.add_action::<ConfigFile>(RelmAction::new_stateless(clone!(@strong sender => move |_| {
            if let Ok(file) = config_file() {
                if let Err(err) = open::that(file) {
                    sender.input(AppMsg::Toast {
                        title: tr!("config-file-opening-error"),
                        description: Some(err.to_string())
                    });

                    tracing::error!("Failed to open config file: {err}");
                }
            }
        })));

        group.add_action::<DebugFile>(RelmAction::new_stateless(clone!(@strong sender => move |_| {
            if let Err(err) = open::that(crate::DEBUG_FILE.as_os_str()) {
                sender.input(AppMsg::Toast {
                    title: tr!("debug-file-opening-error"),
                    description: Some(err.to_string())
                });

                tracing::error!("Failed to open debug file: {err}");
            }
        })));

        group.add_action::<WishUrl>(RelmAction::new_stateless(clone!(@strong sender => move |_| {
            std::thread::spawn(clone!(@strong sender => move || {
                let config = Config::get().unwrap_or_else(|_| CONFIG.clone());

                let web_cache = config.game.path.for_edition(config.launcher.edition)
                    .join(config.launcher.edition.data_folder())
                    .join("webCaches");

                // Find newest cache folder
                let mut web_cache_id = None;

                if let Ok(entries) = web_cache.read_dir() {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() &&
                           entry.file_name().to_string_lossy().trim_matches(|c| "0123456789.".contains(c)).is_empty() &&
                           Some(entry.file_name()) > web_cache_id
                        {
                            web_cache_id = Some(entry.file_name());
                        }
                    }
                }

                if let Some(web_cache_id) = web_cache_id {
                    let web_cache = web_cache
                        .join(web_cache_id)
                        .join("Cache/Cache_Data/data_2");

                    match std::fs::read(web_cache) {
                        Ok(web_cache) => {
                            let web_cache = String::from_utf8_lossy(&web_cache);

                            // https://webstatic-sea.[ho-yo-ver-se].com/hkrpg/event/e20211215gacha-v2/index.html?......
                            if let Some(url) = web_cache.lines().rev().find(|line| line.contains("gacha-v2/index.html")) {
                                let url_begin_pos = url.find("https://").unwrap();
                                let url_end_pos = url_begin_pos + url[url_begin_pos..].find("\0\0\0\0").unwrap();

                                if let Err(err) = open::that(format!("{}#/log", &url[url_begin_pos..url_end_pos])) {
                                    tracing::error!("Failed to open wishes URL: {err}");

                                    sender.input(AppMsg::Toast {
                                        title: tr!("wish-url-opening-error"),
                                        description: Some(err.to_string())
                                    });
                                }
                            }

                            else {
                                tracing::error!("Couldn't find wishes URL: no url found");

                                sender.input(AppMsg::Toast {
                                    title: tr!("wish-url-search-failed"),
                                    description: None
                                });
                            }
                        }

                        Err(err) => {
                            tracing::error!("Couldn't find wishes URL: failed to open cache file: {err}");

                            sender.input(AppMsg::Toast {
                                title: tr!("wish-url-search-failed"),
                                description: Some(err.to_string())
                            });
                        }
                    }
                }

                else {
                    tracing::error!("Couldn't find wishes URL: cache file doesn't exist");

                    sender.input(AppMsg::Toast {
                        title: tr!("wish-url-search-failed"),
                        description: None
                    });
                }
            }));
        })));

        group.add_action::<About>(RelmAction::new_stateless(move |_| {
            about_dialog_broker.send(AboutDialogMsg::Show);
        }));

        widgets.main_window.insert_action_group("win", Some(&group.into_action_group()));

        tracing::info!("Main window initialized");

        let download_picture = model.style == LauncherStyle::Classic && !KEEP_BACKGROUND_FILE.exists();

        // Initialize some heavy tasks
        std::thread::spawn(move || {
            tracing::info!("Initializing heavy tasks");

            let mut tasks = Vec::new();

            // Download background picture if needed

            if download_picture {
                tasks.push(std::thread::spawn(clone!(@strong sender => move || {
                    if let Err(err) = crate::background::download_background() {
                        tracing::error!("Failed to download background picture: {err}");

                        sender.input(AppMsg::Toast {
                            title: tr!("background-downloading-failed"),
                            description: Some(err.to_string())
                        });
                    }
                })));
            }

            // Update components index

            tasks.push(std::thread::spawn(clone!(@strong sender => move || {
                let components = ComponentsLoader::new(&CONFIG.components.path);

                match components.is_sync(&CONFIG.components.servers) {
                    Ok(Some(_)) => (),

                    Ok(None) => {
                        for host in &CONFIG.components.servers {
                            match components.sync(host) {
                                Ok(changes) => {
                                    sender.input(AppMsg::Toast {
                                        title: tr!("components-index-updated"),
                                        description: if changes.is_empty() {
                                            None
                                        } else {
                                            Some(changes.into_iter()
                                                .map(|line| format!("- {line}"))
                                                .collect::<Vec<_>>()
                                                .join("\n"))
                                        }
                                    });

                                    break;
                                }

                                Err(err) => {
                                    tracing::error!("Failed to sync components index");

                                    sender.input(AppMsg::Toast {
                                        title: tr!("components-index-sync-failed"),
                                        description: Some(err.to_string())
                                    });
                                }
                            }
                        }
                    }

                    Err(err) => {
                        tracing::error!("Failed to verify that components index synced");

                        sender.input(AppMsg::Toast {
                            title: tr!("components-index-verify-failed"),
                            description: Some(err.to_string())
                        });
                    }
                }
            })));

            // Update initial patch status

            tasks.push(std::thread::spawn(clone!(@strong sender => move || {
                // Get main patch status
                sender.input(AppMsg::SetMainPatch(match jadeite::get_metadata() {
                    Ok(metadata) => {
                        let status = GAME.get_version()
                            .map(|version| metadata.games.hsr.global.get_status(version))
                            .unwrap_or(metadata.games.hsr.global.status);

                        Some((metadata.jadeite.version, status))
                    }

                    Err(err) => {
                        tracing::error!("Failed to fetch patch metadata: {err}");

                        sender.input(AppMsg::Toast {
                            title: tr!("patch-info-fetching-error"),
                            description: Some(err.to_string())
                        });

                        None
                    }
                }));

                tracing::info!("Updated patch status");
            })));

            // Update initial game version status

            tasks.push(std::thread::spawn(clone!(@strong sender => move || {
                sender.input(AppMsg::SetGameDiff(match GAME.try_get_diff() {
                    Ok(diff) => Some(diff),
                    Err(err) => {
                        tracing::error!("Failed to find game diff: {err}");

                        sender.input(AppMsg::Toast {
                            title: tr!("game-diff-finding-error"),
                            description: Some(err.to_string())
                        });

                        None
                    }
                }));

                tracing::info!("Updated game version status");
            })));

            // Await for tasks to finish execution
            for task in tasks {
                task.join().expect("Failed to join task");
            }

            // Update launcher state
            sender.input(AppMsg::UpdateLauncherState {
                perform_on_download_needed: false,
                show_status_page: true
            });

            // Mark app as loaded
            crate::READY.store(true, Ordering::Relaxed);

            tracing::info!("App is ready");
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        tracing::debug!("Called main window event: {:?}", msg);

        match msg {
            // TODO: make function from this message like with toast
            AppMsg::UpdateLauncherState { perform_on_download_needed, show_status_page } => {
                if show_status_page {
                    sender.input(AppMsg::SetLoadingStatus(Some(Some(tr!("loading-launcher-state")))));
                } else {
                    self.disabled_buttons = true;
                }

                let updater = clone!(@strong sender => move |state| {
                    if show_status_page {
                        match state {
                            StateUpdating::Game => {
                                sender.input(AppMsg::SetLoadingStatus(Some(Some(tr!("loading-launcher-state--game")))));
                            }

                            StateUpdating::Patch => {
                                sender.input(AppMsg::SetLoadingStatus(Some(Some(tr!("loading-launcher-state--patch")))));
                            }
                        }
                    }
                });

                let state = match LauncherState::get_from_config(updater) {
                    Ok(state) => Some(state),
                    Err(err) => {
                        tracing::error!("Failed to update launcher state: {err}");

                        self.toast(tr!("launcher-state-updating-error"), Some(err.to_string()));
    
                        None
                    }
                };

                sender.input(AppMsg::SetLauncherState(state.clone()));

                if show_status_page {
                    sender.input(AppMsg::SetLoadingStatus(None));
                } else {
                    self.disabled_buttons = false;
                }

                if let Some(state) = state {
                    match state {
                        LauncherState::GameUpdateAvailable(_) |
                        LauncherState::GameNotInstalled(_) if perform_on_download_needed => {
                            sender.input(AppMsg::PerformAction);
                        }

                        _ => ()
                    }
                }
            }

            #[allow(unused_must_use)]
            AppMsg::SetGameDiff(diff) => unsafe {
                PREFERENCES_WINDOW.as_ref().unwrap_unchecked().sender().send(PreferencesAppMsg::SetGameDiff(diff));
            }

            #[allow(unused_must_use)]
            AppMsg::SetMainPatch(patch) => unsafe {
                PREFERENCES_WINDOW.as_ref().unwrap_unchecked().sender().send(PreferencesAppMsg::SetMainPatch(patch));
            }

            AppMsg::SetLauncherState(state) => {
                self.state = state;
            }

            AppMsg::SetLoadingStatus(status) => {
                self.loading = status;
            }

            AppMsg::SetLauncherStyle(style) => {
                self.style = style;
            }

            AppMsg::SetDownloading(state) => {
                self.downloading = state;
            }

            AppMsg::DisableButtons(state) => {
                self.disabled_buttons = state;
            }

            AppMsg::SetKillGameButton(state) => {
                self.kill_game_button = state;
            }

            AppMsg::DisableKillGameButton(state) => {
                self.disabled_kill_game_button = state;
            }

            AppMsg::OpenPreferences => unsafe {
                PREFERENCES_WINDOW.as_ref().unwrap_unchecked().widget().present();
            }

            #[allow(unused_must_use)]
            AppMsg::PredownloadUpdate => {
                if let Some(LauncherState::PredownloadAvailable { mut diff, .. }) = self.state.clone() {
                    let tmp = Config::get().unwrap().launcher.temp.unwrap_or_else(std::env::temp_dir);

                    self.downloading = true;

                    let progress_bar_input = self.progress_bar.sender().clone();

                    progress_bar_input.send(ProgressBarMsg::UpdateCaption(Some(tr!("downloading"))));

                    std::thread::spawn(move || {
                        let result = diff.download_to(&tmp, clone!(@strong progress_bar_input => move |curr, total| {
                            progress_bar_input.send(ProgressBarMsg::UpdateProgress(curr, total));
                        }));

                        if let Err(err) = result {
                            sender.input(AppMsg::Toast {
                                title: tr!("downloading-failed"),
                                description: Some(err.to_string())
                            });

                            tracing::error!("Failed to predownload update: {err}");
                        }

                        sender.input(AppMsg::SetDownloading(false));
                        sender.input(AppMsg::UpdateLauncherState {
                            perform_on_download_needed: false,
                            show_status_page: true
                        });
                    });
                }
            }

            AppMsg::PerformAction => unsafe {
                match self.state.as_ref().unwrap_unchecked() {
                    LauncherState::PatchNotVerified |
                    LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Verified, .. } |
                    LauncherState::PredownloadAvailable { patch: JadeitePatchStatusVariant::Unverified, .. } |
                    LauncherState::Launch => launch::launch(sender),

                    LauncherState::PatchNotInstalled |
                    LauncherState::PatchUpdateAvailable => update_patch::update_patch(sender, self.progress_bar.sender().to_owned()),

                    LauncherState::TelemetryNotDisabled => disable_telemetry::disable_telemetry(sender),

                    LauncherState::WineNotInstalled => download_wine::download_wine(sender, self.progress_bar.sender().to_owned()),
                    LauncherState::PrefixNotExists => create_prefix::create_prefix(sender),

                    LauncherState::GameUpdateAvailable(diff) |
                    LauncherState::GameNotInstalled(diff) =>
                        download_diff::download_diff(sender, self.progress_bar.sender().to_owned(), diff.to_owned()),

                    _ => ()
                }
            }

            AppMsg::HideWindow => unsafe {
                MAIN_WINDOW.as_ref().unwrap_unchecked().set_visible(false);
            }

            AppMsg::ShowWindow => unsafe {
                MAIN_WINDOW.as_ref().unwrap_unchecked().present();
            }

            AppMsg::Toast { title, description } => self.toast(title, description)
        }
    }
}

impl App {
    pub fn toast<T: AsRef<str>>(&mut self, title: T, description: Option<T>) {
        let toast = adw::Toast::new(title.as_ref());

        toast.set_timeout(4);

        if let Some(description) = description {
            toast.set_button_label(Some(&tr!("details")));

            let dialog = adw::MessageDialog::new(
                Some(unsafe { MAIN_WINDOW.as_ref().unwrap_unchecked() }),
                Some(title.as_ref()),
                Some(description.as_ref())
            );

            dialog.add_response("close", &tr!("close", { "form" = "noun" }));
            dialog.add_response("save", &tr!("save"));

            dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);

            dialog.connect_response(Some("save"), |_, _| {
                if let Err(err) = open::that(crate::DEBUG_FILE.as_os_str()) {
                    tracing::error!("Failed to open debug file: {err}");
                }
            });

            toast.connect_button_clicked(move |_| {
                dialog.present();
            });
        }

        self.toast_overlay.add_toast(toast);
    }
}
