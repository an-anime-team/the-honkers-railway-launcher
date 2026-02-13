use relm4::prelude::*;

use anime_launcher_sdk::wincompatlib::prelude::*;

use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::star_rail::config::Config;

use crate::*;

use super::{App, AppMsg};

pub fn install_mfc140(sender: ComponentSender<App>) {
    let config = Config::get().unwrap();

    match config.get_selected_wine() {
        Ok(Some(wine_config)) => {
            sender.input(AppMsg::DisableButtons(true));

            std::thread::spawn(move || {
                let wine = wine_config.to_wine(config.components.path, Some(config.game.wine.builds.join(&wine_config.name)))
                    .with_prefix(&config.game.wine.prefix)
                    .with_loader(WineLoader::Current);

                let winetricks = Winetricks::from_wine("winetricks", &wine);

                match winetricks.install_args("mfc140", ["-q", "-f"]) {
                    Ok(mut child) => {
                        match child.wait() {
                            Ok(status) => {
                                if !status.success() {
                                    tracing::error!("Winetricks mfc140 installation failed with status: {}", status);

                                    sender.input(AppMsg::Toast {
                                        title: tr!("mfc140-install-failed"),
                                        description: Some(format!("Exit status: {}", status))
                                    });
                                } else {
                                    tracing::info!("MFC140 installed successfully");
                                }
                            }
                            Err(err) => {
                                tracing::error!("Failed to wait for winetricks process: {}", err);

                                sender.input(AppMsg::Toast {
                                    title: tr!("mfc140-install-failed"),
                                    description: Some(err.to_string())
                                });
                            }
                        }
                    }
                    Err(err) => {
                        tracing::error!("Failed to start winetricks: {}", err);

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

        Ok(None) => {
            tracing::error!("Failed to get selected wine executable");

            sender.input(AppMsg::Toast {
                title: tr!("failed-get-selected-wine"),
                description: None
            });
        }

        Err(err) => {
            tracing::error!("Failed to get selected wine executable: {err}");

            sender.input(AppMsg::Toast {
                title: tr!("failed-get-selected-wine"),
                description: Some(err.to_string())
            });
        }
    }
}
