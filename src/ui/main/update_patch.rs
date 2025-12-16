use relm4::{
    prelude::*,
    Sender
};

use gtk::glib::clone;

use crate::*;
use crate::ui::components::*;

use super::{App, AppMsg};

pub fn update_patch(sender: ComponentSender<App>, progress_bar_input: Sender<ProgressBarMsg>) {
    sender.input(AppMsg::SetDownloading(true));

    let config = Config::get().unwrap();

    std::thread::spawn(move || {
        let result = jadeite::get_latest()
            .and_then(|patch| patch.install(config.patch.path, clone!(
                #[strong]
                sender,

                move |state| {
                    match &state {
                        InstallerUpdate::DownloadingError(err) => {
                            tracing::error!("Downloading failed: {err}");

                            sender.input(AppMsg::Toast {
                                title: tr!("downloading-failed"),
                                description: Some(err.to_string())
                            });
                        }

                        InstallerUpdate::UnpackingError(err) => {
                            tracing::error!("Unpacking failed: {err}");

                            sender.input(AppMsg::Toast {
                                title: tr!("unpacking-failed"),
                                description: Some(err.clone())
                            });
                        }

                        _ => ()
                    }

                    #[allow(unused_must_use)] {
                        progress_bar_input.send(ProgressBarMsg::UpdateFromState(
                            DiffUpdate::Installer(state)
                        ));
                    }
                }
            )));

        if let Err(err) = result {
            tracing::error!("Failed to download latest patch version");

            sender.input(AppMsg::Toast {
                title: tr!("main-patch-update-failed"),
                description: Some(err.to_string())
            });
        }

        sender.input(AppMsg::SetDownloading(false));
        sender.input(AppMsg::UpdateLauncherState {
            perform_on_download_needed: false,
            show_status_page: true
        });
    });
}
