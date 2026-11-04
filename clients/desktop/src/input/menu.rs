use std::collections::HashMap;

use muda::{
    CheckMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, CMD_OR_CTRL, Code, Modifiers},
};
use winit::event_loop::EventLoopProxy;

use crate::{app::UserEvent, emulator::FpsTarget};

#[derive(Clone, Copy, Debug)]
pub(crate) enum MenuAction {
    Settings,
    Mute,
    ChangeFps(FpsTarget),
    Screenshot,
    Reset,
    Pause,
    Quit,
    LoadRom,
    LoadState,
    SaveState,
    ShowFps,
    ShowLogs,
    Palettes,
    Sprites,
    Tiles,
    Maps,
    AudioChannels,
}

pub(crate) struct AppMenu {
    _menu: Menu,
    actions: HashMap<MenuId, MenuAction>,
    screenshot: MenuItem,
    reset: MenuItem,
    pause: MenuItem,
    load_state: MenuItem,
    save_state: MenuItem,
    show_fps: CheckMenuItem,
    fps_items: Vec<(FpsTarget, CheckMenuItem)>,
    palettes: MenuItem,
    sprites: MenuItem,
    tiles: MenuItem,
    maps: MenuItem,
    audio_channels: MenuItem,
}

impl AppMenu {
    pub(crate) fn action(&self, id: &MenuId) -> Option<MenuAction> {
        self.actions.get(id).copied()
    }

    pub(crate) fn set_rom_loaded(&self, loaded: bool) {
        let rom_dependent_items = [
            &self.screenshot,
            &self.reset,
            &self.pause,
            &self.load_state,
            &self.save_state,
            &self.palettes,
            &self.sprites,
            &self.tiles,
            &self.maps,
            &self.audio_channels,
        ];
        for item in rom_dependent_items {
            item.set_enabled(loaded);
        }
        self.show_fps.set_enabled(loaded);
    }

    pub(crate) fn set_show_fps(&self, shown: bool) {
        self.show_fps.set_checked(shown);
    }

    pub(crate) fn select_fps(&self, selected: FpsTarget) {
        for (target, item) in &self.fps_items {
            item.set_checked(*target == selected);
        }
    }
}

fn menu_item(
    actions: &mut HashMap<MenuId, MenuAction>,
    action: MenuAction,
    text: &str,
    enabled: bool,
    accelerator: Option<Accelerator>,
) -> MenuItem {
    let item = MenuItem::new(text, enabled, accelerator);
    actions.insert(item.id().clone(), action);
    item
}

fn check_item(
    actions: &mut HashMap<MenuId, MenuAction>,
    action: MenuAction,
    text: &str,
    enabled: bool,
    checked: bool,
    accelerator: Option<Accelerator>,
) -> CheckMenuItem {
    let item = CheckMenuItem::new(text, enabled, checked, accelerator);
    actions.insert(item.id().clone(), action);
    item
}

pub(crate) fn build_menu(proxy: EventLoopProxy<UserEvent>) -> AppMenu {
    let menu_proxy = proxy.clone();
    muda::MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let mut actions = HashMap::new();

    let screenshot = menu_item(
        &mut actions,
        MenuAction::Screenshot,
        "Screenshot",
        false,
        Some(Accelerator::new(Modifiers::empty(), Code::F4)),
    );
    let reset = menu_item(
        &mut actions,
        MenuAction::Reset,
        "Reset",
        false,
        Some(Accelerator::new(CMD_OR_CTRL, Code::KeyR)),
    );
    let pause = menu_item(
        &mut actions,
        MenuAction::Pause,
        "Pause",
        false,
        Some(Accelerator::new(CMD_OR_CTRL, Code::KeyP)),
    );
    let load_state = menu_item(&mut actions, MenuAction::LoadState, "Load State", false, None);
    let save_state = menu_item(&mut actions, MenuAction::SaveState, "Save State", false, None);
    let show_fps = check_item(
        &mut actions,
        MenuAction::ShowFps,
        "Show FPS",
        false,
        false,
        Some(Accelerator::new(Modifiers::empty(), Code::F3)),
    );
    let palettes = menu_item(&mut actions, MenuAction::Palettes, "Palettes", false, None);
    let sprites = menu_item(&mut actions, MenuAction::Sprites, "Sprites", false, None);
    let tiles = menu_item(&mut actions, MenuAction::Tiles, "Tiles", false, None);
    let maps = menu_item(&mut actions, MenuAction::Maps, "Maps", false, None);
    let audio_channels = menu_item(&mut actions, MenuAction::AudioChannels, "Audio Channels", false, None);

    let fps_items: Vec<(FpsTarget, CheckMenuItem)> = [
        (FpsTarget::Fixed(15), "15"),
        (FpsTarget::Fixed(30), "30"),
        (FpsTarget::Fixed(60), "60"),
        (FpsTarget::Fixed(90), "90"),
        (FpsTarget::Fixed(120), "120"),
        (FpsTarget::Fixed(240), "240"),
        (FpsTarget::Max, "Max"),
        (FpsTarget::Native, "59.7275 (Native)"),
    ]
    .into_iter()
    .map(|(target, text)| {
        let checked = target == FpsTarget::Native;
        let item = check_item(&mut actions, MenuAction::ChangeFps(target), text, true, checked, None);
        (target, item)
    })
    .collect();

    let change_fps = Submenu::new("Change FPS", true);
    for (target, item) in &fps_items {
        if *target == FpsTarget::Native {
            change_fps.append(&PredefinedMenuItem::separator()).unwrap();
        }
        change_fps.append(item).unwrap();
    }

    let submenu = Submenu::with_items(
        "IronBoyAdvance",
        true,
        &[
            &menu_item(
                &mut actions,
                MenuAction::Settings,
                "Settings",
                true,
                Some(Accelerator::new(CMD_OR_CTRL, Code::Comma)),
            ),
            &PredefinedMenuItem::separator(),
            &menu_item(&mut actions, MenuAction::Mute, "Mute", true, None),
            &change_fps,
            &screenshot,
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(Some("Hide")),
            &reset,
            &pause,
            &menu_item(
                &mut actions,
                MenuAction::Quit,
                "Quit",
                true,
                Some(Accelerator::new(CMD_OR_CTRL, Code::KeyQ)),
            ),
        ],
    )
    .unwrap();

    let file = Submenu::with_items(
        "File",
        true,
        &[
            &menu_item(
                &mut actions,
                MenuAction::LoadRom,
                "Load ROM",
                true,
                Some(Accelerator::new(CMD_OR_CTRL, Code::KeyO)),
            ),
            &PredefinedMenuItem::separator(),
            &load_state,
            &save_state,
        ],
    )
    .unwrap();

    let tools = Submenu::with_items(
        "Tools",
        true,
        &[
            &show_fps,
            &menu_item(&mut actions, MenuAction::ShowLogs, "Logs", true, None),
            &PredefinedMenuItem::separator(),
            &palettes,
            &sprites,
            &tiles,
            &maps,
            &audio_channels,
        ],
    )
    .unwrap();

    let menu = Menu::with_items(&[&submenu, &file, &tools]).unwrap();

    #[cfg(target_os = "windows")]
    tracing::warn!("TODO: support windows menu");
    #[cfg(target_os = "linux")]
    tracing::warn!("TODO: support linux menu");
    #[cfg(target_os = "macos")]
    menu.init_for_nsapp();

    AppMenu {
        _menu: menu,
        actions,
        screenshot,
        reset,
        pause,
        load_state,
        save_state,
        show_fps,
        fps_items,
        palettes,
        sprites,
        tiles,
        maps,
        audio_channels,
    }
}
