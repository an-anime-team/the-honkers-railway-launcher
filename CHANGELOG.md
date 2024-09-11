# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added support for new game installation format
- Added 7z binary requirement

## [1.8.1] - 02.09.2024

### Fixed

- Fixed gamescope config file layout

## [1.8.0] - 01.09.2024

### Added

- Apply chmod 755 to extracted files if 7z was used

### Changed

- Reworked gamescope settings

### Fixed

- Create cache folder if it doesn't exist
- (potentially) fixed a bug with pre-download button
- Fixed calculation of unpacked files size due to API changes
- Respect downloaded file size in free space check

## [1.7.1] - 03.08.2024

### Removed

- Removed `p7zip` dependency

## [1.7.0] - 02.08.2024

### Added

- Added "Indonesia" wine language option
- Added writing of the game's output to the `game.log` file in the launcher's folder.
  Size of this file is controlled by the `LAUNCHER_GAME_LOG_FILE_LIMIT` environment variable.
- Respect root `.version` file for game version parsing
- Added 2.4.0 voiceovers sizes

### Fixed

- Fixed `dwebp` package name for fedora during initial setup
- Fixed Discord RPC updates

### Changed

- Changed background images processing logic
- Prioritize parsed game version over the API response

### Removed

- Removed `xdelta3` dependency
- Removed migrate installation feature

## [1.6.1] - 02.07.2024

### Added

- Handle dwebp re-coding errors
- Added 2.3.0 voiceovers sizes

### Fixed

- Added workaround for wrong pre-downloads API format

## [1.6.0] - 17.06.2024

### Added

- List missing dependencies on non-standard distros during initial setup
- Added 2.2.0 voiceovers sizes

### Fixed

- Fixed Italian localization breaking the launcher

### Changed

- Support new game API
- Improved background pictures processing
- Updated desktop file entry to include "aagl" keyword
- Localized `force-grab-cursor` to Ukrainian

## [1.5.5] - 08.05.2024

### Added

- Added Czech

## [1.5.4] - 27.03.2024

### Changed

- Updated voiceovers sizes
- Updated game version parsing algorithm

## [1.5.3] - 24.03.2024

### Added

- Bundle `applications-system-symbolic` icon to the app
- Added "force grab cursor" option to the gamescope settings
- Added Thai
- Added Ukrainian

### Fixed

- Fixed GtkSwitch UI state representation

### Changed

- Update wish url
- Updated dependencies
- Improved app args parsing
- Updated locales

### Fixed

- Fixed GtkSwitch UI state representation

## [1.5.2] - 29.12.2023

### Fixed

- Fixed "Kill game process" button

### Changed

- Updated Turkish
- Updated German
- Updated Polish
- Updated Chinese

## [1.5.1] - 15.11.2023

### Added

- Added `UpdatingPermissions` installation step
- Downloaders now will skip finished files and truncate them if needed
- Added new fix for the API responses
- Added voiceovers repairing support
- Added special tooltips for concerning patch status

### Fixed

- Fixed launch button color in concerning patches

### Changed

- Increased voiceovers version prediction error
- Updated FPS Unlocker version which fixes new game version integration issue
- Updated Turkish
- Updated German
- Updated Chinese
- Updated Polish

## [1.5.0] - 13.11.2023

### Added

- Added Vietnamese
- Added Korean
- Added Dutch
- Made free space checks resolve symlinks
- Added voiceovers support
- Added new `Concerning` patch status

### Changed

- Updated development libraries versions
- Updated Japanese
- Updated Chinese

## [1.4.0] - 20.08.2023

### Added

- Added feature to map wine drives
- Added `%launch_args%` magic word for game launching command.
  Now you can use `%bash_command% <script> %launch_args%` to run custom script
- Added `--session <name>` flag to switch active session
- Added Portuguese
- Added Polish

### Fixed

- Fixed logo size in the first run window

### Changed

- Updated Turkish
- Updated Italian
- Updated Japanese
- Updated Swedish
- Improved files migration code. In the best case scenarios, it will work immediately now
- Updated wishes url extractor to use new cache storage

## [1.3.0] - 02.08.2023

### Added

- Added new gamescope version compatibility
- Added "launcher behavior" option
- Added "kill game process" button when chosen behavior keeps launcher window open
- Bundled some icons into the app for consistency across the systems
- Added better panics handler
- Added Swedish

### Fixed

- Fixed predownload button sensitivity

### Changed

- Improved pre-downloads state checking
- Replaced translation functions by `tr!` macro
- Reworked app resources structure

## [1.2.5] - 19.07.2023

### Fixed

- Added a workaround to remove outdated `Telemetry.dll` file which fixes login error 400 in game login screen

## [1.2.4] - 17.07.2023

### Fixed

- Fixed jadeite patch state handling from the metadata file
- Fixed game pre-downloading button sensitivity when the update was partially downloaded, but then interrupted
- Fixed game pre-downloading button visibility when jadeite patch state is not "verified"

## [1.2.3] - 14.07.2023

### Fixed

- Fixed telemetry disabling

### Changed

- Updated Italian
- Updated Hungarian
- Updated Japanese
- Updated Indonesian
- Updated Spanish
- Updated Turkish
- Updated Chinese

## [1.2.2] - 18.06.2023

### Added

- Added deletion of old patch files (just in case)
- Added telemetry disabling state support
- Added Discord RPC icons updating

### Changed

- Replaced xlua patch by "disable mhypbase" option
- Returned back old `background` file path

## [1.2.1] - 15.06.2023

### Added

- Added Discord RPC icon selection
- Added Japanese
- Added Hungarian
- Integrated Jadeite patch

### Fixed

- Fixed default launcher language selection at the first start
- Fixed some installer updates reporting (including "checking free space")
- Fixed check button style for newly made sessions
- Fixed repairer's NaN progress
- Fixed game session selection when current one is removed

### Changed

- Reworked game sessions selection
- Updated Indonesian
- Updated French
- Made initial tasks async which has decreased startup time
- Updated Spanish

### Removed

- Removed patch mirror migration

## [1.2.0] - 24.05.2023

### Added

- Added Italian
- Added Indonesian
- Added dynamic main button icon switching
- Set button label as "Resume" when the diff is part downloaded
- Added options to use wine / gstreamer shared libraries from selected wine build.
  These options will configure `LD_LIBRARY_PATH` and `GST_PLUGIN_PATH` environment variables
- Added setting of `LC_ALL` in wine lang setting
- Added `LAUNCHER_REQUESTS_TIMEOUT` environment variable

### Fixed

- Fixed session applying on each launcher start
- Fixed predownload button ui
- Fixed proton builds integration with sandbox
- Fixed compatibility between sessions manager and sandbox
- Fixed sandboxing of inexisting folders

### Changed

- Apply selected session before launching the game.
  This will properly save your game session when you switch between wine prefixes
- Redesigned main button
- Used `whatadistro` to identify recommended package manager in the first run window
- Moved a lot of settings to separate page
- Set fsr quality mode in enhancements settings instead of strength
- Updated fps unlocker data
- Made temporary workaround to the game API changes
- Increased default requests timeout to 8 seconds
- Updated minreq to support `http_proxy`-like variables

### Removed

- Removed Futex2 wine sync option

## [1.1.0] - 06.05.2023

### Added

- Added game settings section
- Added game sessions manager
- Added `LAUNCHER_FOLDER` variable support.
  Using this you can specify root path where the launcher stores `config.json` and other files
- Added patch repository mirror

### Changed

- Improved launcher logo rendering quality
- Reworked entry rows in the settings

### Fixed

- Fixed sandboxed game running (sounds are broken for now)

## [1.0.0] - 01.05.2023

🚀 Initial release

<br>

[unreleased]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.8.1...next
[1.8.1]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.8.0...1.8.1
[1.8.0]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.7.1...1.8.0
[1.7.1]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.7.0...1.7.1
[1.7.0]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.6.1...1.7.0
[1.6.1]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.6.0...1.6.1
[1.6.0]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.5.5...1.6.0
[1.5.5]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.5.4...1.5.5
[1.5.4]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.5.3...1.5.4
[1.5.3]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.5.2...1.5.3
[1.5.2]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.5.1...1.5.2
[1.5.1]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.5.0...1.5.1
[1.5.0]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.4.0...1.5.0
[1.4.0]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.3.0...1.4.0
[1.3.0]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.2.5...1.3.0
[1.2.5]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.2.4...1.2.5
[1.2.4]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.2.3...1.2.4
[1.2.3]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.2.2...1.2.3
[1.2.2]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.2.1...1.2.2
[1.2.1]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.2.0...1.2.1
[1.2.0]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.1.0...1.2.0
[1.1.0]: https://github.com/an-anime-team/the-honkers-railway-launcher/compare/1.0.0...1.1.0
[1.0.0]: https://github.com/an-anime-team/the-honkers-railway-launcher/releases/tag/1.0.0
