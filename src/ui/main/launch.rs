use relm4::prelude::*;
use gtk::prelude::*;

use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::star_rail::config::Config;
use anime_launcher_sdk::star_rail::config::schema::prelude::LauncherBehavior;

use crate::*;

use super::{App, AppMsg};

pub fn launch(sender: ComponentSender<App>) {
    let config = Config::get().unwrap();

    match config.launcher.behavior {
        // Disable launch button and show kill game button if behavior set to "Nothing" to prevent sussy actions
        LauncherBehavior::Nothing => {
            sender.input(AppMsg::DisableButtons(true));
            sender.input(AppMsg::SetKillGameButton(true));
        }

        // Hide launcher window if behavior set to "Hide" or "Close"
        LauncherBehavior::Hide | LauncherBehavior::Close => sender.input(AppMsg::HideWindow)
    }

    std::thread::spawn(move || {
        // Workaround of an issue appeared in 1.2 game update with lowercased telemetry.dll file
        // This block will eventually be removed
        let result = Config::get().and_then(|config| {
            let telemetry = config.game.path
                .for_edition(config.launcher.edition)
                .join("StarRail_Data/Plugins/x86_64/Telemetry.dll");

            if telemetry.exists() {
                std::fs::remove_file(telemetry)?;
            }

            Ok(())
        }).and_then(|_| anime_launcher_sdk::star_rail::game::run());

        // if let Err(err) = anime_launcher_sdk::star_rail::game::run() {
        if let Err(err) = result {
            tracing::error!("Failed to launch game: {err}");

            sender.input(AppMsg::Toast {
                title: tr!("game-launching-failed"),
                description: Some(err.to_string())
            });
        }

        match config.launcher.behavior {
            // Enable launch button and hide kill game button if behavior set to "Nothing" after the game has closed
            LauncherBehavior::Nothing => {
                sender.input(AppMsg::DisableButtons(false));
                sender.input(AppMsg::SetKillGameButton(false));
            }

            // Show back launcher window if behavior set to "Hide" and the game has closed
            LauncherBehavior::Hide => sender.input(AppMsg::ShowWindow),

            // Otherwise close the launcher if behavior set to "Close" and the game has closed
            // We're calling quit method from the main context here because otherwise app won't be closed properly
            LauncherBehavior::Close => gtk::glib::MainContext::default().invoke(|| {
                relm4::main_application().quit();
            })
        }
    });
}
