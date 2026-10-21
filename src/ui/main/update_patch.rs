// SPDX-License-Identifier: GPL-3.0-or-later
//
// the-honkers-railway-launcher
// Copyright (C) 2021 - 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
// Copyright (C) 2025 - 2026  @JohnTheCoolingFan <ivan8215145640@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use relm4::Sender;
use gtk::glib::clone;

use crate::*;
use crate::ui::components::*;

use super::{App, AppMsg};

pub fn update_patch(sender: ComponentSender<App>, progress_bar_input: Sender<ProgressBarMsg>) {
    sender.input(AppMsg::SetDownloading(true));

    let config = Config::get().unwrap();

    std::thread::spawn(move || {
        let result = jadeite::get_latest().and_then(|patch| {
            patch.install(
                config.patch.path,
                clone!(
                    #[strong]
                    sender,
                    move |state| {
                        match &state {
                            InstallerUpdate::DownloadingError(err) => {
                                tracing::error!("Downloading failed: {err}");

                                sender.input(AppMsg::Toast {
                                    title: i18n!("downloading-failed"),
                                    description: Some(err.to_string())
                                });
                            }

                            InstallerUpdate::UnpackingError(err) => {
                                tracing::error!("Unpacking failed: {err}");

                                sender.input(AppMsg::Toast {
                                    title: i18n!("unpacking-failed"),
                                    description: Some(err.clone())
                                });
                            }

                            _ => ()
                        }

                        #[allow(unused_must_use)]
                        {
                            progress_bar_input.send(ProgressBarMsg::UpdateFromState(state.into()));
                        }
                    }
                )
            )
        });

        if let Err(err) = result {
            tracing::error!("Failed to download latest patch version");

            sender.input(AppMsg::Toast {
                title: i18n!("main-patch-update-failed"),
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
