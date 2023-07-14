use relm4::prelude::*;

use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::star_rail::config::{Config, Schema};

use anime_launcher_sdk::star_rail::states::LauncherState;
use anime_launcher_sdk::star_rail::consts::*;

use anime_launcher_sdk::anime_game_core::prelude::*;
use anime_launcher_sdk::anime_game_core::star_rail::prelude::*;

use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::*;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod move_folder;
pub mod i18n;
pub mod background;
pub mod ui;

use ui::main::*;
use ui::first_run::main::*;

pub const APP_ID: &str = "moe.launcher.the-honkers-railway-launcher";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_DEBUG: bool = cfg!(debug_assertions);

/// Sets to `true` when the `App` component is ready (fully initialized)
pub static READY: AtomicBool = AtomicBool::new(false);

// TODO: get rid of using this function in all the components' events
//       e.g. by converting preferences pages into Relm4 Components
/// Check if the app is ready
pub fn is_ready() -> bool {
    READY.load(Ordering::Relaxed)
}

lazy_static::lazy_static! {
    /// Config loaded on the app's start. Use `Config::get()` to get up to date config instead.
    /// This one is used to prepare some launcher UI components on start
    pub static ref CONFIG: Schema = Config::get().expect("Failed to load config");

    pub static ref GAME: Game = Game::new(CONFIG.game.path.for_edition(CONFIG.launcher.edition), CONFIG.launcher.edition);

    /// Path to launcher folder. Standard is `$HOME/.local/share/honkers-railway-launcher`
    pub static ref LAUNCHER_FOLDER: PathBuf = launcher_dir().expect("Failed to get launcher folder");

    /// Path to launcher's cache folder. Standard is `$HOME/.cache/honkers-railway-launcher`
    pub static ref CACHE_FOLDER: PathBuf = cache_dir().expect("Failed to get launcher's cache folder");

    /// Path to `debug.log` file. Standard is `$HOME/.local/share/honkers-railway-launcher/debug.log`
    pub static ref DEBUG_FILE: PathBuf = LAUNCHER_FOLDER.join("debug.log");

    /// Path to `background` file. Standard is `$HOME/.local/share/honkers-railway-launcher/background`
    pub static ref BACKGROUND_FILE: PathBuf = LAUNCHER_FOLDER.join("background");

    /// Path to `.keep-background` file. Used to mark launcher that it shouldn't update background picture
    ///
    /// Standard is `$HOME/.local/share/honkers-railway-launcher/.keep-background`
    pub static ref KEEP_BACKGROUND_FILE: PathBuf = LAUNCHER_FOLDER.join(".keep-background");

    /// Path to `.first-run` file. Used to mark launcher that it should run FirstRun window
    ///
    /// Standard is `$HOME/.local/share/honkers-railway-launcher/.first-run`
    pub static ref FIRST_RUN_FILE: PathBuf = LAUNCHER_FOLDER.join(".first-run");
}

fn main() {
    // Create launcher folder if it isn't
    if !LAUNCHER_FOLDER.exists() {
        std::fs::create_dir_all(LAUNCHER_FOLDER.as_path()).expect("Failed to create launcher folder");

        // This one is kinda critical buy well, I can't do something with it
        std::fs::write(FIRST_RUN_FILE.as_path(), "").expect("Failed to create .first-run file");

        // Set initial launcher language based on system language
        // CONFIG is initialized lazily so it will contain following changes as well
        let mut config = Config::get().expect("Failed to get config");

        config.launcher.language = i18n::format_lang(&i18n::get_default_lang());

        Config::update_raw(config).expect("Failed to update config");
    }

    // Force debug output
    let force_debug = std::env::args().any(|arg| &arg == "--debug");

    // Run the game
    let run_game = std::env::args().any(|arg| &arg == "--run-game");

    // Forcely run the game
    let just_run_game = std::env::args().any(|arg| &arg == "--just-run-game");

    // Prepare stdout logger
    let stdout = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter({
            if APP_DEBUG || force_debug {
                LevelFilter::TRACE
            } else {
                LevelFilter::WARN
            }
        })
        .with_filter(filter_fn(|metadata| {
            !metadata.target().contains("rustls")
        }));

    // Prepare debug file logger
    let file = match std::fs::File::create(DEBUG_FILE.as_path()) {
        Ok(file) => file,
        Err(error) => panic!("Failed to create debug.log file: {:?}", error)
    };

    let debug_log = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(false)
        .with_writer(std::sync::Arc::new(file))
        .with_filter(filter_fn(|metadata| {
            !metadata.target().contains("rustls")
        }));

    tracing_subscriber::registry()
        .with(stdout)
        .with(debug_log)
        .init();

    tracing::info!("Starting application ({APP_VERSION})");

    adw::init().expect("Libadwaita initialization failed");

    // Register and include resources
    gtk::gio::resources_register_include!("resources.gresource")
        .expect("Failed to register resources");

    // Set application's title
    gtk::glib::set_application_name("The Honkers Railway Launcher");
    gtk::glib::set_program_name(Some("The Honkers Railway Launcher"));

    // Set global css
    relm4::set_global_css(&format!("
        progressbar > text {{
            margin-bottom: 4px;
        }}

        window.classic-style {{
            background: url(\"file://{}\");
            background-repeat: no-repeat;
            background-size: cover;
        }}

        window.classic-style progressbar {{
            background-color: #00000020;
            border-radius: 16px;
            padding: 8px 16px;
        }}

        window.classic-style progressbar:hover {{
            background-color: #00000060;
            color: #ffffff;
            transition-duration: 0.5s;
            transition-timing-function: linear;
        }}

        .round-bin {{
            border-radius: 24px;
        }}
    ", BACKGROUND_FILE.to_string_lossy()));

    // Set UI language
    let lang = CONFIG.launcher.language.parse().expect("Wrong language format used in config");

    i18n::set_lang(lang).expect("Failed to set launcher language");

    tracing::info!("Set UI language to {}", i18n::get_lang());

    // Run FirstRun window if .first-run file persist
    if FIRST_RUN_FILE.exists() {
        // Create the app
        let app = RelmApp::new(APP_ID);

        // Show first run window
        app.run::<FirstRunApp>(());
    }

    // Run the app if everything's ready
    else {
        // Temporary workaround for old patches which HAVE to be reverted
        // I don't believe to users to read announcements so better do this
        // 
        // There's 2 files which were modified by the old patch, but since the game
        // was updated those files were updated as well, so no need for additional actions
        // 
        // Should be removed in future
        let game_path = CONFIG.game.path.for_edition(CONFIG.launcher.edition);

        if game_path.join("Generated").exists() {
            std::fs::remove_dir_all(game_path.join("Generated"))
                .expect("Failed to delete 'Generated' folder");
        }

        if game_path.join("TVMBootstrap.dll").exists() {
            std::fs::remove_file(game_path.join("TVMBootstrap.dll"))
                .expect("Failed to delete 'TVMBootstrap.dll' file");
        }

        // AC won't say a thing about this file anyway but for consistency I decided
        // to delete it as well
        if game_path.join("launch.bat").exists() {
            std::fs::remove_file(game_path.join("launch.bat"))
                .expect("Failed to delete 'launch.bat' file");
        }

        // Patch was renaming crash reporter to disable it
        if game_path.join("UnityCrashHandler64.exe.bak").exists() {
            if game_path.join("UnityCrashHandler64.exe").exists() {
                std::fs::remove_file(game_path.join("UnityCrashHandler64.exe.bak"))
                    .expect("Failed to delete 'UnityCrashHandler64.exe.bak' file");
            }

            else {
                std::fs::rename(game_path.join("UnityCrashHandler64.exe.bak"), game_path.join("UnityCrashHandler64.exe"))
                    .expect("Failed to rename 'UnityCrashHandler64.exe.bak' file to 'UnityCrashHandler64.exe'");
            }
        }

        // End of temporary workaround ^

        if run_game || just_run_game {
            let state = LauncherState::get_from_config(|_| {})
                .expect("Failed to get launcher state");

            match state {
                LauncherState::Launch => {
                    anime_launcher_sdk::star_rail::game::run().expect("Failed to run the game");

                    return;
                }

                LauncherState::PatchNotVerified |
                LauncherState::PredownloadAvailable { .. } |
                LauncherState::PatchUpdateAvailable => {
                    if just_run_game {
                        anime_launcher_sdk::star_rail::game::run().expect("Failed to run the game");

                        return;
                    }
                }

                _ => ()
            }
        }

        // Create the app
        let app = RelmApp::new(APP_ID);

        // Show main window
        app.run::<App>(());
    }
}
