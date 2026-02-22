use crossterm::event::{Event, EventStream, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
    _task: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let task = tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_rate);

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        if tx.send(AppEvent::Tick).is_err() {
                            break;
                        }
                    }
                    event = reader.next() => {
                        match event {
                            Some(Ok(Event::Key(key))) => {
                                if tx.send(AppEvent::Key(key)).is_err() {
                                    break;
                                }
                            }
                            Some(Ok(Event::Resize(w, h))) => {
                                if tx.send(AppEvent::Resize(w, h)).is_err() {
                                    break;
                                }
                            }
                            Some(Err(_)) | None => break,
                            _ => {}
                        }
                    }
                }
            }
        });

        Self { rx, _task: task }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
