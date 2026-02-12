/// Actions produced by event routing. The shell layer interprets and executes these.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Play a sound from the given category.
    PlaySound { category: String },
    /// Set the terminal tab title.
    SetTabTitle { title: String },
    /// Send a desktop notification.
    Notify {
        message: String,
        title: String,
        color: NotifyColor,
    },
    /// Do nothing — agent session or unknown event.
    Skip,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotifyColor {
    Red,
    Blue,
    Yellow,
}
