use std::{thread::sleep, time::Duration};

use bizarre_engine::{
    event::EventQueue,
    window::{window_events::WindowEvent, Window, WindowCreateInfo, WindowTrait},
};

fn main() {
    let create_info = WindowCreateInfo::normal_window("Bizarre Window".into(), [600, 400].into());

    let mut window = Window::new(&create_info).unwrap();
    window.map().unwrap();

    let mut event_queue = EventQueue::default();
    let window_event_reader = event_queue.create_reader();
    event_queue
        .register_reader::<WindowEvent>(window_event_reader)
        .unwrap();

    let mut frame_index = 0;

    while !window.close_requested() {
        event_queue.change_frames();
        let _ = window.drain_events_to_queue(&mut event_queue);

        let count = event_queue
            .pull_events::<WindowEvent>(&window_event_reader)
            .unwrap()
            .map(|ev| ev.len());

        if let Some(count) = count {
            if count > 0 {
                println!("Got {count} Window Events on frame #{frame_index}");
                frame_index += 1;
            }
        }
    }
}
