use std::{mem::MaybeUninit, sync::{atomic::{AtomicBool, self}, Arc}};

use trayicon::{TrayIconBuilder, MenuBuilder};
use windows::Win32::UI::WindowsAndMessaging::{TranslateMessage, DispatchMessageA, PeekMessageA, PM_REMOVE};

use crate::config::{CONFIG, AppConfig, write_config};

use {std::sync::mpsc};

struct ExtendedMenuBuilder(MenuBuilder<TrayEvents>);
impl ExtendedMenuBuilder {
    fn new() -> ExtendedMenuBuilder {
        ExtendedMenuBuilder(MenuBuilder::<TrayEvents>::new())
    }
    fn checkable(self, name: &str, is_checked: bool, id: TrayEvents) -> Self {
        ExtendedMenuBuilder(self.0.checkable(name, is_checked, id))
    }
    // checkable with enabled check
    fn cwec(self, name: &str, is_checked: bool, id: TrayEvents, enable: &[bool]) -> Self {
        ExtendedMenuBuilder(self.0.with(trayicon::MenuItem::Checkable {
            id,
            name: name.into(),
            disabled: enable.iter().any(|v| !v),
            is_checked,
            icon: None
        }))
    }
    fn submenu(self, name: &str, menu: MenuBuilder<TrayEvents>) -> Self {
        ExtendedMenuBuilder(self.0.submenu(name, menu))
    }
}
impl From<ExtendedMenuBuilder> for MenuBuilder<TrayEvents> {
    fn from(value: ExtendedMenuBuilder) -> Self {
        value.0
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum TrayEvents {
    _Unused,

    // Global
    ShowInGameCharacter,
    ShowInGameTime,

    // Slippi
    EnableSlippi,
    SlippiShowQueueing,

    SlippiEnableRanked,
    SlippiRankedShowRank,
    SlippiRankedShowViewRankedProfileButton,
    SlippiRankedShowScore,

    SlippiEnableUnranked,

    SlippiEnableDirect,

    SlippiEnableTeams,

    // Unclepunch
    EnableUnclePunch,

    // Training Mode
    EnableTrainingMode,

    // Vs. Mode
    EnableVsMode,

    Quit,
}

fn build_menu() -> MenuBuilder<TrayEvents> {
    CONFIG.with_ref(|c| {
        MenuBuilder::new()
        .with(trayicon::MenuItem::Item {
            id: TrayEvents::_Unused,
            name: "Health:".into(),
            disabled: true,
            icon: None
        })
        .with(trayicon::MenuItem::Item {
            id: TrayEvents::_Unused,
            name: "✔️ Connected to Discord".into(),
            disabled: true,
            icon: None
        })
        .with(trayicon::MenuItem::Item {
            id: TrayEvents::_Unused,
            name: "❌ Searching for dolphin process...".into(),
            disabled: true,
            icon: None
        })
        .separator()
        .submenu(
            "Global",
                MenuBuilder::new()
                    .checkable("Show Character", c.global.show_in_game_character, TrayEvents::ShowInGameCharacter)
                    .checkable("Show In-Game Time", c.global.show_in_game_time, TrayEvents::ShowInGameTime)
        )
        .submenu(
            "Slippi Online",
            ExtendedMenuBuilder::new()
                    .checkable("Enabled", c.slippi.enabled, TrayEvents::EnableSlippi)
                    .cwec("Show activity when searching", c.slippi.show_queueing, TrayEvents::SlippiShowQueueing, &[c.slippi.enabled])
                    .submenu(
                        "Ranked",
                    ExtendedMenuBuilder::new()
                            .cwec("Enabled", c.slippi.ranked.enabled, TrayEvents::SlippiEnableRanked, &[c.slippi.enabled])
                            .cwec("Show rank", c.slippi.ranked.show_rank, TrayEvents::SlippiRankedShowRank, &[c.slippi.enabled, c.slippi.ranked.enabled])
                            .cwec("Show \"View Ranked Profile\" button", c.slippi.ranked.show_view_ranked_profile_button, TrayEvents::SlippiRankedShowViewRankedProfileButton, &[c.slippi.enabled, c.slippi.ranked.enabled])
                            .cwec("Show match score", c.slippi.ranked.show_score, TrayEvents::SlippiRankedShowScore, &[c.slippi.enabled, c.slippi.ranked.enabled])
                            .into()
                    )
                    .submenu(
                        "Unranked",
                        ExtendedMenuBuilder::new()
                            .cwec("Enabled", c.slippi.unranked.enabled, TrayEvents::SlippiEnableUnranked, &[c.slippi.enabled])
                            .into()
                    )
                    .submenu(
                        "Direct",
                        ExtendedMenuBuilder::new()
                            .cwec("Enabled", c.slippi.direct.enabled, TrayEvents::SlippiEnableDirect, &[c.slippi.enabled])
                            .into()
                    )
                    .submenu(
                        "Teams",
                        ExtendedMenuBuilder::new()
                            .cwec("Enabled", c.slippi.teams.enabled, TrayEvents::SlippiEnableTeams, &[c.slippi.enabled])
                            .into()
                    )
                    .into()
        )
        .submenu(
            "UnclePunch",
            MenuBuilder::new()
                    .checkable("Enabled", c.uncle_punch.enabled, TrayEvents::EnableUnclePunch)
        )
        .submenu(
            "Training Mode",
            MenuBuilder::new()
                    .checkable("Enabled", c.training_mode.enabled, TrayEvents::EnableTrainingMode)
        )
        .submenu(
            "Vs. Mode",
            MenuBuilder::new()
                    .checkable("Enabled", c.vs_mode.enabled, TrayEvents::EnableVsMode)
        )
        .separator()
        .item("Quit", TrayEvents::Quit)
        .with(trayicon::MenuItem::Item {
            id: TrayEvents::_Unused,
            name: "Made by @FluentCoding".into(),
            disabled: true,
            icon: None
        })
    })
}

pub fn run_tray() {
    let should_end = Arc::new(AtomicBool::new(false));
    let shared_should_end = should_end.clone();

    let (s, r) = mpsc::channel::<TrayEvents>();
    let icon_raw = include_bytes!("../assets/icon.ico");

    let mut tray_icon = TrayIconBuilder::new()
        .sender(s)
        .icon_from_buffer(icon_raw)
        .tooltip("Slippi Discord Integration")
        .menu(
            build_menu()
        )
        .build()
        .unwrap();

    std::thread::spawn(move || {
        let mut toggle_handler = |modifier: fn(&mut AppConfig)| {
            CONFIG.with_mut(|c| { modifier(c); write_config(c); });
            tray_icon.set_menu(&build_menu()).unwrap();
        };
        r.iter().for_each(|m| match m {
            TrayEvents::Quit => {
                should_end.store(true, atomic::Ordering::Relaxed);
            },
            TrayEvents::ShowInGameCharacter => toggle_handler(|f| f.global.show_in_game_character = !f.global.show_in_game_character),
            TrayEvents::ShowInGameTime => toggle_handler(|f| f.global.show_in_game_time = !f.global.show_in_game_time),

            TrayEvents::EnableSlippi => toggle_handler(|f| f.slippi.enabled = !f.slippi.enabled),
            TrayEvents::SlippiShowQueueing => toggle_handler(|f| f.slippi.show_queueing = !f.slippi.show_queueing),

            TrayEvents::SlippiEnableRanked => toggle_handler(|f| f.slippi.ranked.enabled = !f.slippi.ranked.enabled),
            TrayEvents::SlippiRankedShowRank => toggle_handler(|f| f.slippi.ranked.show_rank = !f.slippi.ranked.show_rank),
            TrayEvents::SlippiRankedShowViewRankedProfileButton => toggle_handler(|f| f.slippi.ranked.show_view_ranked_profile_button = !f.slippi.ranked.show_view_ranked_profile_button),
            TrayEvents::SlippiRankedShowScore => toggle_handler(|f| f.slippi.ranked.show_score = !f.slippi.ranked.show_score),

            TrayEvents::SlippiEnableUnranked => toggle_handler(|f| f.slippi.unranked.enabled = !f.slippi.unranked.enabled),

            TrayEvents::SlippiEnableDirect => toggle_handler(|f| f.slippi.direct.enabled = !f.slippi.direct.enabled),

            TrayEvents::SlippiEnableTeams => toggle_handler(|f| f.slippi.teams.enabled = !f.slippi.teams.enabled),

            TrayEvents::EnableUnclePunch => toggle_handler(|f| f.uncle_punch.enabled = !f.uncle_punch.enabled),

            TrayEvents::EnableVsMode => toggle_handler(|f| f.vs_mode.enabled = !f.vs_mode.enabled),

            TrayEvents::EnableTrainingMode => toggle_handler(|f| f.training_mode.enabled = !f.training_mode.enabled),
            _ => {}
        })
    });
    
    // Application message loop
    loop {
        if shared_should_end.load(atomic::Ordering::Relaxed) {
            break;
        }
        unsafe {
            let mut msg = MaybeUninit::uninit();
            let bret = PeekMessageA(msg.as_mut_ptr(), None, 0, 0, PM_REMOVE);
            if bret.as_bool() {
                TranslateMessage(msg.as_ptr());
                DispatchMessageA(msg.as_ptr());
            }
        }
    }
}