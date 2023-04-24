use {std::sync::mpsc, tray_item::TrayItem};

enum Message {
    Quit,
}

pub fn run_tray() {
    let mut tray = TrayItem::new("Slippi Discord Rich Presence", "icon").unwrap();
    let (tx, rx) = mpsc::channel();

    tray.add_menu_item("Quit", move || {
        tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    loop {
        match rx.recv() {
            Ok(Message::Quit) => break,
            _ => {}
        }
    }
}