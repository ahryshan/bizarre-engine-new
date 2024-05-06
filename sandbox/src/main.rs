use std::{thread::sleep, time::Duration};

use bizarre_engine::window::{Window, WindowCreateInfo, WindowTrait};

fn main() {
    let create_info = WindowCreateInfo::normal_window("Bizarre Window".into(), [600, 400].into());

    let mut window = Window::new(&create_info).unwrap();
    window.map().unwrap();
    sleep(Duration::from_secs(3));
    drop(window);

    let create_info = WindowCreateInfo::splash_window("Bizarre Splash".into(), [1000, 500].into());
    let mut window = Window::new(&create_info).unwrap();
    window.map().unwrap();
    sleep(Duration::from_secs(3));

    let create_info = WindowCreateInfo::fullscreen_window("Bizarre Fullscreen".into());
    let mut window = Window::new(&create_info).unwrap();
    window.map().unwrap();
    sleep(Duration::from_secs(3));
}
