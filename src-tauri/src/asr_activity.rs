use std::sync::mpsc;

#[derive(Clone, Default)]
pub(crate) struct AsrActivityReporter {
    tx: Option<mpsc::Sender<()>>,
}

impl AsrActivityReporter {
    pub(crate) fn new(tx: mpsc::Sender<()>) -> Self {
        Self { tx: Some(tx) }
    }

    pub(crate) fn disabled() -> Self {
        Self { tx: None }
    }

    /// Marks effective remote ASR feedback, not local microphone activity.
    ///
    /// Providers should call this only for non-empty text progress that can tell
    /// session-level no-feedback timeout the service is still recognizing.
    pub(crate) fn mark_feedback(&self) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AsrActivityReporter;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn reporter_sends_activity_signal() {
        let (tx, rx) = mpsc::channel();
        AsrActivityReporter::new(tx).mark_feedback();

        assert!(rx.recv_timeout(Duration::from_millis(20)).is_ok());
    }

    #[test]
    fn disabled_reporter_is_noop() {
        AsrActivityReporter::disabled().mark_feedback();
    }
}
