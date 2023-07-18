use std::{path::PathBuf, sync::Arc};

use tokio::{
    fs,
    sync::watch::{self, Receiver, Sender},
};

use crate::structs::ApplicationState;

// Saving only, load is handled by app root
pub struct SaveManager {
    watch_sender: Sender<()>,
}

impl SaveManager {
    pub fn new(state: Arc<ApplicationState>, location: PathBuf) -> Self {
        let (send, recv) = watch::channel(());
        tokio::spawn(saver(recv, location, state));

        Self { watch_sender: send }
    }

    pub fn save(&self) {
        let _ = self.watch_sender.send(());
    }
}

async fn saver(
    mut recv: Receiver<()>,
    location: PathBuf,
    state: Arc<ApplicationState>,
) -> anyhow::Result<()> {
    loop {
        recv.changed().await?;

        let serialized = serde_json::to_string_pretty(&state);

        let serialized = match serialized {
            Ok(serialized) => serialized,
            Err(_) => {
                println!("Could not serialize application state for saving.");
                continue;
            }
        };

        match fs::write(location.clone(), serialized).await {
            Ok(_) => {}
            Err(_) => {
                println!("Could not save application state.");
            }
        };
    }
}
