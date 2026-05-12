use relm4::prelude::*;
use relm4::Sender;
use anime_launcher_sdk::anime_game_core::reqwest::blocking::Client;
use anime_launcher_sdk::anime_game_core::sophon::installer::{
    SophonInstaller, Update as SophonRepairerUpdate
};
use anime_launcher_sdk::anime_game_core::sophon;

use crate::*;
use crate::ui::components::*;
use super::{App, AppMsg};

#[allow(unused_must_use)]
pub fn repair_game(sender: ComponentSender<App>, progress_bar_input: Sender<ProgressBarMsg>) {
    let config = Config::get().expect("failed to read launcher config");

    progress_bar_input.send(ProgressBarMsg::UpdateCaption(Some(tr!("verifying-files"))));
    sender.input(AppMsg::SetDownloading(true));

    std::thread::spawn(move || {
        let client = Client::new();

        let game_branches_info =
            sophon::api::get_game_branches_info(&client, &config.launcher.edition.into())
                .expect("failed to get game branches info");

        let game_branch_info = game_branches_info
            .get_game_branch_by_id_or_biz_latest(config.launcher.edition.api_game_id())
            .expect("failed to get latest game version info");

        let downloads = sophon::api::get_game_download_sophon_info(
            &client,
            game_branch_info
                .main
                .as_ref()
                .expect("`None` case would've been caught earlier"),
            &config.launcher.edition.into()
        )
        .expect("failed to get game info");

        let game_download_info = downloads
            .manifests
            .iter()
            .find(|download_info| download_info.matching_field == "game")
            .cloned()
            .expect("failed to get game download info");

        let mut manifests = vec![game_download_info];

        let game_path = config.game.path.for_edition(config.launcher.edition);

        let game = Game::new(game_path, config.launcher.edition);

        if let Ok(voiceovers) = game.get_voice_packages() {
            for package in voiceovers {
                let locale = package.locale();
                let locale = locale.to_code();

                let download_info = downloads
                    .manifests
                    .iter()
                    .find(|download_info| download_info.matching_field == locale)
                    .cloned();

                if let Some(download_info) = download_info {
                    manifests.push(download_info);
                }
            }
        }

        let repairer_temp = config.launcher.temp.unwrap_or_else(std::env::temp_dir);

        for manifest in manifests {
            let mut repairer = SophonInstaller::new(client.clone(), &manifest, &repairer_temp)
                .expect("failed to initialize sophon repairer");
            repairer.mode_repair = true;

            let updater = |msg: SophonRepairerUpdate| match msg {
                SophonRepairerUpdate::CheckingFilesProgress {
                    total,
                    passed
                } => {
                    tracing::trace!(passed, total, "Verification progress");

                    progress_bar_input.send(ProgressBarMsg::UpdateProgressCounter(passed, total));
                }

                SophonRepairerUpdate::DownloadingProgressFiles {
                    total_files,
                    downloaded_files
                } => {
                    tracing::trace!(downloaded_files, total_files, "Repairing progress");

                    progress_bar_input.send(ProgressBarMsg::UpdateProgressCounter(
                        downloaded_files,
                        total_files
                    ));
                }

                SophonRepairerUpdate::CheckingFiles {
                    ..
                } => {
                    tracing::trace!("Verification started");
                }

                SophonRepairerUpdate::DownloadingStarted {
                    ..
                } => {
                    tracing::trace!("Repairing started");

                    progress_bar_input
                        .send(ProgressBarMsg::UpdateCaption(Some(tr!("repairing-files"))));
                }

                SophonRepairerUpdate::DownloadingFinished => {
                    tracing::trace!("Repair finished");
                }

                SophonRepairerUpdate::DownloadingError(err) => {
                    tracing::error!(?err, "Error during repairing")
                }

                _ => {}
            };

            repairer.install(
                game_path,
                config.launcher.repairer.threads as usize,
                updater
            );

            let _ = std::fs::remove_dir_all(repairer.downloading_temp());
        }

        sender.input(AppMsg::SetDownloading(false));
    });
}
