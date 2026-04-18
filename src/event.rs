use crossterm::event::{Event, EventStream, KeyEvent, KeyEventKind, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
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
                        // Translate the terminal event into an AppEvent. On Windows/WSL,
                        // Key Release/Repeat arrive alongside Press — filter to Press only.
                        let app_event = match event {
                            Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                                AppEvent::Key(key)
                            }
                            Some(Ok(Event::Mouse(mouse))) => AppEvent::Mouse(mouse),
                            Some(Ok(Event::Resize(w, h))) => AppEvent::Resize(w, h),
                            Some(Err(_)) | None => break,
                            _ => continue,
                        };
                        if tx.send(app_event).is_err() {
                            break;
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
