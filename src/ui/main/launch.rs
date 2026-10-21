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

use gtk::prelude::*;

use anime_launcher_sdk::config::ConfigExt;
use anime_launcher_sdk::star_rail::config::Config;
use anime_launcher_sdk::star_rail::config::schema::prelude::LauncherBehavior;

use crate::*;

use super::{App, AppMsg};

pub fn launch(sender: ComponentSender<App>) {
    let config = Config::get().unwrap();

    match config.launcher.behavior {
        // Disable launch button and show kill game button if behavior set to "Nothing" to prevent
        // sussy actions
        LauncherBehavior::Nothing => {
            sender.input(AppMsg::DisableButtons(true));
            sender.input(AppMsg::SetKillGameButton(true));
        }

        // Hide launcher window if behavior set to "Hide" or "Close"
        LauncherBehavior::Hide | LauncherBehavior::Close => sender.input(AppMsg::HideWindow)
    }

    std::thread::spawn(move || {
        // Workaround of an issue appeared in 1.2 game update with lowercased
        // telemetry.dll file This block will eventually be removed
        let result = Config::get()
            .and_then(|config| {
                let telemetry = config
                    .game
                    .path
                    .for_edition(config.launcher.edition)
                    .join("StarRail_Data/Plugins/x86_64/Telemetry.dll");

                if telemetry.exists() {
                    std::fs::remove_file(telemetry)?;
                }

                Ok(())
            })
            .and_then(|_| anime_launcher_sdk::star_rail::game::run());

        // if let Err(err) = anime_launcher_sdk::star_rail::game::run() {
        if let Err(err) = result {
            tracing::error!("Failed to launch game: {err}");

            sender.input(AppMsg::Toast {
                title: i18n!("game-launching-failed"),
                description: Some(err.to_string())
            });
        }

        match config.launcher.behavior {
            // Enable launch button and hide kill game button if behavior set to "Nothing" after the
            // game has closed
            LauncherBehavior::Nothing => {
                sender.input(AppMsg::DisableButtons(false));
                sender.input(AppMsg::SetKillGameButton(false));
            }

            // Show back launcher window if behavior set to "Hide" and the game has closed
            LauncherBehavior::Hide => sender.input(AppMsg::ShowWindow),

            // Otherwise close the launcher if behavior set to "Close" and the game has closed
            // We're calling quit method from the main context here because otherwise app won't be
            // closed properly
            LauncherBehavior::Close => gtk::glib::MainContext::default().invoke(|| {
                relm4::main_application().quit();
            })
        }
    });
}
