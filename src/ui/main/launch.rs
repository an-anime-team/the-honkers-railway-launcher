use relm4::prelude::*;

use crate::i18n::*;

use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::star_rail::config::Config;

use super::{App, AppMsg};

pub fn launch(sender: ComponentSender<App>) {
    sender.input(AppMsg::HideWindow);

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
                title: tr("game-launching-failed"),
                description: Some(err.to_string())
            });
        }

        sender.input(AppMsg::ShowWindow);
    });
}
