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

use adw::prelude::*;
use anime_launcher_sdk::is_available;

use crate::*;

use super::main::{FirstRunAppMsg, MAIN_WINDOW};

pub struct TosWarningApp;

#[derive(Debug, Clone)]
pub enum TosWarningAppMsg {
    Continue,
    Exit
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for TosWarningApp {
    type Init = ();
    type Input = TosWarningAppMsg;
    type Output = FirstRunAppMsg;

    view! {
        adw::PreferencesPage {
            set_hexpand: true,

            add = &adw::PreferencesGroup {
                set_valign: gtk::Align::Center,
                set_vexpand: true,

                gtk::Label {
                    set_label: &i18n!("tos-violation-warning"),
                    add_css_class: "title-1"
                }
            },

            add = &adw::PreferencesGroup {
                gtk::Label {
                    set_label: &i18n!("tos-violation-warning-message"),
                    set_wrap: true,
                    set_selectable: true
                }
            },

            add = &adw::PreferencesGroup {
                set_valign: gtk::Align::Center,
                set_vexpand: true,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,
                    set_spacing: 8,

                    gtk::Button {
                        set_label: &i18n!("continue"),
                        set_css_classes: &["suggested-action", "pill"],

                        connect_clicked => TosWarningAppMsg::Continue
                    },

                    gtk::Button {
                        set_label: &i18n!("exit"),
                        add_css_class: "pill",

                        connect_clicked => TosWarningAppMsg::Exit
                    }
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>
    ) -> AsyncComponentParts<Self> {
        let model = Self;
        let widgets = view_output!();

        AsyncComponentParts {
            model,
            widgets
        }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            #[allow(unused_must_use, static_mut_refs)]
            TosWarningAppMsg::Continue => {
                let dialog = adw::MessageDialog::new(
                    unsafe { MAIN_WINDOW.as_ref() },
                    Some(&i18n!("tos-dialog-title")),
                    Some(&i18n!("tos-dialog-message"))
                );

                dialog.add_responses(&[("exit", &i18n!("exit")), ("continue", &i18n!("agree"))]);

                dialog.connect_response(None, move |_, response| match response {
                    "exit" => relm4::main_application().quit(),

                    "continue" => {
                        let installed =
                            is_available("git") && (is_available("7z") || is_available("7za"));

                        if installed {
                            sender.output(Self::Output::ScrollToDefaultPaths);
                        }
                        else {
                            sender.output(Self::Output::ScrollToDependencies);
                        }
                    }

                    _ => unreachable!()
                });

                dialog.present();
            }

            TosWarningAppMsg::Exit => relm4::main_application().quit()
        }
    }
}
