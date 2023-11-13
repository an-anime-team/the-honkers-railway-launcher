use relm4::prelude::*;

use relm4::factory::{
    AsyncFactoryVecDeque,
    AsyncFactoryComponent,
    AsyncFactorySender
};

use gtk::prelude::*;
use adw::prelude::*;

use anime_launcher_sdk::wincompatlib::prelude::*;

use anime_launcher_sdk::anime_game_core::prelude::*;
use anime_launcher_sdk::anime_game_core::star_rail::consts::GameEdition;

use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::star_rail::config::Config;
use anime_launcher_sdk::star_rail::config::schema::launcher::{LauncherStyle, LauncherBehavior};

pub mod components;

use components::*;

use crate::ui::migrate_installation::MigrateInstallationApp;
use crate::ui::preferences::main::PreferencesAppMsg;

use crate::i18n::*;
use crate::*;

#[derive(Debug)]
struct VoicePackageComponent {
    locale: VoiceLocale,
    installed: bool,
    sensitive: bool
}

#[relm4::factory(async)]
impl AsyncFactoryComponent for VoicePackageComponent {
    type Init = (VoiceLocale, bool);
    type Input = GeneralAppMsg;
    type Output = GeneralAppMsg;
    type CommandOutput = ();
    type ParentWidget = adw::ExpanderRow;

    view! {
        root = adw::ActionRow {
            set_title: &tr!(&self.locale.to_name().to_ascii_lowercase()),

            add_suffix = &gtk::Button {
                #[watch]
                set_visible: self.installed,

                #[watch]
                set_sensitive: self.sensitive,

                set_icon_name: "user-trash-symbolic",
                add_css_class: "flat",
                set_valign: gtk::Align::Center,

                connect_clicked[sender, index] => move |_| {
                    sender.input(GeneralAppMsg::RemoveVoicePackage(index.clone()));
                }
            },

            add_suffix = &gtk::Button {
                #[watch]
                set_visible: !self.installed,

                #[watch]
                set_sensitive: self.sensitive,

                set_icon_name: "document-save-symbolic",
                add_css_class: "flat",
                set_valign: gtk::Align::Center,

                connect_clicked[sender, index] => move |_| {
                    sender.input(GeneralAppMsg::AddVoicePackage(index.clone()));
                }
            }
        }
    }

    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            locale: init.0,
            installed: init.1,
            sensitive: true
        }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncFactorySender<Self>) {
        self.installed = !self.installed;

        sender.output(msg)
            .unwrap();
    }
}

pub struct GeneralApp {
    voice_packages: AsyncFactoryVecDeque<VoicePackageComponent>,
    migrate_installation: Controller<MigrateInstallationApp>,
    components_page: AsyncController<ComponentsPage>,

    game_diff: Option<VersionDiff>,
    main_patch: Option<(Version, JadeitePatchStatusVariant)>,

    style: LauncherStyle,
    languages: Vec<String>
}

#[derive(Debug, Clone)]
pub enum GeneralAppMsg {
    /// Supposed to be called automatically on app's run when the latest game version
    /// was retrieved from the API
    SetGameDiff(Option<VersionDiff>),

    /// Supposed to be called automatically on app's run when the latest UnityPlayer patch version
    /// was retrieved from remote repos
    SetMainPatch(Option<(Version, JadeitePatchStatusVariant)>),

    UpdateDownloadedWine,
    UpdateDownloadedDxvk,

    OpenMigrateInstallation,

    OpenMainPage,
    OpenComponentsPage,

    UpdateLauncherStyle(LauncherStyle),

    WineOpen(&'static [&'static str]),

    Toast {
        title: String,
        description: Option<String>
    }
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for GeneralApp {
    type Init = ();
    type Input = GeneralAppMsg;
    type Output = PreferencesAppMsg;

    view! {
        #[root]
        adw::PreferencesPage {
            set_title: &tr!("general"),
            set_icon_name: Some("applications-system-symbolic"),

            add = &adw::PreferencesGroup {
                set_title: &tr!("appearance"),

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,

                    set_spacing: 32,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::ToggleButton {
                            add_css_class: "card",

                            set_width_request: 180,
                            set_height_request: 120,

                            #[watch]
                            set_active: model.style == LauncherStyle::Modern,

                            gtk::Image {
                                set_resource: Some(&format!("{APP_RESOURCE_PATH}/images/modern.svg"))
                            },

                            connect_clicked => GeneralAppMsg::UpdateLauncherStyle(LauncherStyle::Modern)
                        },

                        gtk::Label {
                            set_text: &tr!("modern"),

                            set_margin_top: 16
                        }
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::ToggleButton {
                            add_css_class: "card",

                            set_width_request: 180,
                            set_height_request: 120,

                            #[watch]
                            set_active: model.style == LauncherStyle::Classic,

                            gtk::Image {
                                set_resource: Some(&format!("{APP_RESOURCE_PATH}/images/classic.svg"))
                            },

                            connect_clicked => GeneralAppMsg::UpdateLauncherStyle(LauncherStyle::Classic)
                        },

                        gtk::Label {
                            set_text: &tr!("classic"),

                            set_margin_top: 16
                        }
                    }
                }
            },

            add = &adw::PreferencesGroup {
                #[watch]
                set_visible: model.style == LauncherStyle::Classic,

                adw::ActionRow {
                    set_title: &tr!("update-background"),
                    set_subtitle: &tr!("update-background-description"),

                    add_suffix = &gtk::Switch {
                        set_valign: gtk::Align::Center,
                        set_active: !KEEP_BACKGROUND_FILE.exists(),

                        connect_state_notify => |switch| {
                            #[allow(unused_must_use)]
                            if switch.state() {
                                std::fs::remove_file(KEEP_BACKGROUND_FILE.as_path());
                            } else {
                                std::fs::write(KEEP_BACKGROUND_FILE.as_path(), "");
                            }
                        }
                    }
                }
            },

            add = &adw::PreferencesGroup {
                set_title: &tr!("general"),

                adw::ComboRow {
                    set_title: &tr!("launcher-language"),
                    set_subtitle: &tr!("launcher-language-description"),

                    set_model: Some(&gtk::StringList::new(&model.languages.iter().map(|lang| lang.as_str()).collect::<Vec<&str>>())),

                    set_selected: {
                        let selected = crate::i18n::get_lang().language;

                        SUPPORTED_LANGUAGES.iter()
                            .position(|lang| lang.language == selected)
                            .unwrap_or(0) as u32
                    },

                    connect_selected_notify => |row| {
                        if is_ready() {
                            if let Ok(mut config) = Config::get() {
                                config.launcher.language = crate::i18n::format_lang(SUPPORTED_LANGUAGES
                                    .get(row.selected() as usize)
                                    .unwrap_or(&SUPPORTED_LANGUAGES[0]));
    
                                Config::update(config);
                            }
                        }
                    }
                },

                adw::ComboRow {
                    set_title: &tr!("game-edition"),

                    set_model: Some(&gtk::StringList::new(&[
                        &tr!("global"),
                        &tr!("china")
                    ])),

                    set_selected: GameEdition::list().iter()
                        .position(|edition| edition == &CONFIG.launcher.edition)
                        .unwrap() as u32,

                    connect_selected_notify[sender] => move |row| {
                        if is_ready() {
                            #[allow(unused_must_use)]
                            if let Ok(mut config) = Config::get() {
                                config.launcher.edition = GameEdition::list()[row.selected() as usize];

                                Config::update(config);

                                sender.output(PreferencesAppMsg::UpdateLauncherState);
                            }
                        }
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,
                    set_margin_top: 16,

                    gtk::Button {
                        set_label: &tr!("migrate-installation"),
                        set_tooltip_text: Some(&tr!("migrate-installation-description")),

                        connect_clicked => GeneralAppMsg::OpenMigrateInstallation
                    }
                }
            },

            add = &adw::PreferencesGroup {
                set_title: &tr!("status"),

                adw::ActionRow {
                    set_title: &tr!("game-version"),

                    add_suffix = &gtk::Label {
                        #[watch]
                        set_text: &match model.game_diff.as_ref() {
                            Some(diff) => match diff {
                                VersionDiff::Latest { version: current, .. } |
                                VersionDiff::Predownload { current, .. } |
                                VersionDiff::Diff { current, .. } |
                                VersionDiff::Outdated { current, .. } => current.to_string(),

                                VersionDiff::NotInstalled { .. } => tr!("game-not-installed")
                            }

                            None => String::from("?")
                        },

                        #[watch]
                        set_css_classes: match model.game_diff.as_ref() {
                            Some(diff) => match diff {
                                VersionDiff::Latest { .. }       => &["success"],
                                VersionDiff::Predownload { .. }  => &["accent"],
                                VersionDiff::Diff { .. }         => &["warning"],
                                VersionDiff::Outdated { .. }     => &["error"],
                                VersionDiff::NotInstalled { .. } => &[]
                            }

                            None => &[]
                        },

                        #[watch]
                        set_tooltip_text: Some(&match model.game_diff.as_ref() {
                            Some(diff) => match diff {
                                VersionDiff::Latest { .. } => String::new(),

                                VersionDiff::Predownload { current, latest, .. } => tr!("game-predownload-available", {
                                    "old" = current.to_string(),
                                    "new" = latest.to_string()
                                }),

                                VersionDiff::Diff { current, latest, .. } => tr!("game-update-available", {
                                    "old" = current.to_string(),
                                    "new" = latest.to_string()
                                }),

                                VersionDiff::Outdated { latest, ..} => tr!("game-outdated", {
                                    "latest" = latest.to_string()
                                }),

                                VersionDiff::NotInstalled { .. } => String::new()
                            }

                            None => String::new()
                        })
                    }
                },

                adw::ActionRow {
                    set_title: &tr!("player-patch-version"),
                    set_subtitle: &tr!("player-patch-version-description"),

                    add_suffix = &gtk::Label {
                        #[watch]
                        set_text: &match model.main_patch.as_ref() {
                            Some((version, _)) => version.to_string(),
                            None => String::from("?")
                        },

                        #[watch]
                        set_css_classes: match model.main_patch.as_ref() {
                            Some((_, status)) => match status {
                                JadeitePatchStatusVariant::Verified => &["success"],
                                JadeitePatchStatusVariant::Unverified => &["warning"],
                                JadeitePatchStatusVariant::Broken => &["error"],
                                JadeitePatchStatusVariant::Unsafe => &["error"]
                            }

                            None => &[]
                        },

                        #[watch]
                        set_tooltip_text: Some(&match model.main_patch.as_ref() {
                            Some((_, status)) => match status {
                                JadeitePatchStatusVariant::Unverified => tr!("patch-testing-tooltip"),
                                JadeitePatchStatusVariant::Broken => tr!("patch-broken-tooltip"),
                                JadeitePatchStatusVariant::Unsafe => tr!("patch-unsafe-tooltip"),

                                _ => String::new()
                            }

                            None => String::new()
                        })
                    }
                }
            },

            add = &adw::PreferencesGroup {
                set_title: &tr!("options"),

                adw::ComboRow {
                    set_title: &tr!("launcher-behavior"),
                    set_subtitle: &tr!("launcher-behavior-description"),

                    set_model: Some(&gtk::StringList::new(&[
                        &tr!("nothing"),
                        &tr!("hide", { "form" = "verb" }),
                        &tr!("close", { "form" = "verb" }),
                    ])),

                    set_selected: match CONFIG.launcher.behavior {
                        LauncherBehavior::Nothing => 0,
                        LauncherBehavior::Hide    => 1,
                        LauncherBehavior::Close   => 2
                    },

                    connect_selected_notify => |row| {
                        if is_ready() {
                            if let Ok(mut config) = Config::get() {
                                config.launcher.behavior = [
                                    LauncherBehavior::Nothing,
                                    LauncherBehavior::Hide,
                                    LauncherBehavior::Close
                                ][row.selected() as usize];

                                Config::update(config);
                            }
                        }
                    }
                }
            },

            add = &adw::PreferencesGroup {
                adw::ActionRow {
                    set_title: &tr!("components"),
                    set_subtitle: &tr!("components-description"),

                    add_suffix = &gtk::Image {
                        set_icon_name: Some("go-next-symbolic")
                    },

                    set_activatable: true,

                    connect_activated => GeneralAppMsg::OpenComponentsPage
                },

                adw::ExpanderRow {
                    set_title: &tr!("wine-tools"),

                    add_row = &adw::ActionRow {
                        set_title: &tr!("command-line"),
                        set_subtitle: "wineconsole",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["wineconsole"])
                    },

                    add_row = &adw::ActionRow {
                        set_title: &tr!("registry-editor"),
                        set_subtitle: "regedit",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["regedit"])
                    },

                    add_row = &adw::ActionRow {
                        set_title: &tr!("explorer"),
                        set_subtitle: "explorer",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["explorer"])
                    },

                    add_row = &adw::ActionRow {
                        set_title: &tr!("task-manager"),
                        set_subtitle: "taskmgr",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["taskmgr"])
                    },

                    add_row = &adw::ActionRow {
                        set_title: &tr!("configuration"),
                        set_subtitle: "winecfg",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["winecfg"])
                    },

                    add_row = &adw::ActionRow {
                        set_title: &tr!("debugger"),
                        set_subtitle: "start winedbg",

                        set_activatable: true,

                        connect_activated => GeneralAppMsg::WineOpen(&["start", "winedbg"])
                    }
                }
            }
        },

        #[local_ref]
        components_page -> gtk::Box {}
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        tracing::info!("Initializing general settings");

        let mut model = Self {
            voice_packages: AsyncFactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), std::convert::identity),

            migrate_installation: MigrateInstallationApp::builder()
                .launch(())
                .detach(),

            components_page: ComponentsPage::builder()
                .launch(())
                .forward(sender.input_sender(), std::convert::identity),

            game_diff: None,
            main_patch: None,

            style: CONFIG.launcher.style,
            languages: SUPPORTED_LANGUAGES.iter().map(|lang| tr!(format_lang(lang).as_str())).collect()
        };

        let components_page = model.components_page.widget();

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        tracing::debug!("Called general settings event: {:?}", msg);

        match msg {
            GeneralAppMsg::SetGameDiff(diff) => {
                self.game_diff = diff;
            }

            GeneralAppMsg::SetMainPatch(patch) => {
                self.main_patch = patch;
            }

            GeneralAppMsg::UpdateDownloadedWine => {
                self.components_page.sender()
                    .send(ComponentsPageMsg::UpdateDownloadedWine)
                    .unwrap();
            }

            GeneralAppMsg::UpdateDownloadedDxvk => {
                self.components_page.sender()
                    .send(ComponentsPageMsg::UpdateDownloadedDxvk)
                    .unwrap();
            }

            GeneralAppMsg::OpenMigrateInstallation => unsafe {
                if let Some(window) = crate::ui::main::PREFERENCES_WINDOW.as_ref() {
                    self.migrate_installation.widget().set_transient_for(Some(window.widget()));
                }

                self.migrate_installation.widget().present();
            }

            GeneralAppMsg::OpenMainPage => unsafe {
                PREFERENCES_WINDOW.as_ref()
                    .unwrap_unchecked()
                    .widget()
                    .close_subpage();
            }

            GeneralAppMsg::OpenComponentsPage => unsafe {
                PREFERENCES_WINDOW.as_ref()
                    .unwrap_unchecked()
                    .widget()
                    .present_subpage(self.components_page.widget());
            }

            #[allow(unused_must_use)]
            GeneralAppMsg::UpdateLauncherStyle(style) => {
                if style == LauncherStyle::Classic && !KEEP_BACKGROUND_FILE.exists() {
                    if let Err(err) = crate::background::download_background() {
                        tracing::error!("Failed to download background picture");

                        sender.input(GeneralAppMsg::Toast {
                            title: tr!("background-downloading-failed"),
                            description: Some(err.to_string())
                        });

                        return;
                    }
                }

                if let Ok(mut config) = Config::get() {
                    config.launcher.style = style;

                    Config::update(config);
                }

                self.style = style;

                sender.output(Self::Output::SetLauncherStyle(style));
            }

            GeneralAppMsg::WineOpen(executable) => {
                let config = Config::get().unwrap_or_else(|_| CONFIG.clone());

                if let Ok(Some(wine)) = config.get_selected_wine() {
                    let result = wine
                        .to_wine(config.components.path, Some(config.game.wine.builds.join(&wine.name)))
                        .with_prefix(config.game.wine.prefix)
                        .with_loader(WineLoader::Current)
                        .with_arch(WineArch::Win64)
                        .run_args(executable);

                    if let Err(err) = result {
                        sender.input(GeneralAppMsg::Toast {
                            title: tr!("wine-run-error", {
                                "executable" = executable.join(" ")
                            }),
                            description: Some(err.to_string())
                        });

                        tracing::error!("Failed to run {:?} using wine: {err}", executable);
                    }
                }
            }

            #[allow(unused_must_use)]
            GeneralAppMsg::Toast { title, description } => {
                sender.output(Self::Output::Toast { title, description });
            }
        }
    }
}
