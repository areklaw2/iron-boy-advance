use muda::{
    Menu, MenuItem, PredefinedMenuItem, Submenu,
    accelerator::{Accelerator, Code, Modifiers},
};

pub(crate) fn build_menu() -> Menu {
    let menu = Menu::new();
    let menu_item2 = MenuItem::new("Menu item #2", false, None);
    let submenu = Submenu::with_items(
        "Submenu Outer",
        true,
        &[
            &MenuItem::new("Menu item #1", true, Some(Accelerator::new(Modifiers::ALT, Code::KeyD))),
            &PredefinedMenuItem::separator(),
            &menu_item2,
            &MenuItem::new("Menu item #3", true, None),
            &PredefinedMenuItem::separator(),
            &Submenu::with_items(
                "Submenu Inner",
                true,
                &[
                    &MenuItem::new("Submenu item #1", true, None),
                    &PredefinedMenuItem::separator(),
                    &menu_item2,
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let _ = menu.append(&submenu);

    #[cfg(target_os = "windows")]
    tracing::warn!("TODO: support windows menu");
    #[cfg(target_os = "linux")]
    tracing::warn!("TODO: support linux menu");
    #[cfg(target_os = "macos")]
    menu.init_for_nsapp();

    menu
}
