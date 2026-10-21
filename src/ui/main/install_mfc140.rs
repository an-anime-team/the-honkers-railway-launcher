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

use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::star_rail::config::Config;

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
                    title: i18n!("mfc140-install-failed"),
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
