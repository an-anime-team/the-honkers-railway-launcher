use relm4::prelude::*;
use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::star_rail::config::Config;
use anime_launcher_sdk::anime_game_core::patches::prelude::*;

use crate::*;
use super::{App, AppMsg};

pub fn install_mfc140(sender: ComponentSender<App>) {
    let config = Config::get().unwrap();

    sender.input(AppMsg::DisableButtons(true));

    std::thread::spawn(move || {
        match mfc140::install(&config.game.wine.prefix, config.launcher.temp.as_ref()) {
            Ok(()) => {
                tracing::info!("MFC140 installed successfully");
            }
            Err(err) => {
                tracing::error!("Failed to install mfc140: {}", err);

                sender.input(AppMsg::Toast {
                    title: tr!("mfc140-install-failed"),
                    description: Some(err.to_string())
                });
            }
        }

        sender.input(AppMsg::DisableButtons(false));
        sender.input(AppMsg::UpdateLauncherState {
            perform_on_download_needed: false,
            show_status_page: true
        });
    });
}
