#[macro_export]
macro_rules! ctrl {
    ($ch:expr, $key_event:expr) => {
        $key_event.code == KeyCode::Char($ch) && $key_event.modifiers.contains(KeyModifiers::CONTROL)
    };
}

#[macro_export]
macro_rules! alt {
    ($ch:expr, $key_event:expr) => {
        $key_event.code == KeyCode::Char($ch) && $key_event.modifiers.contains(KeyModifiers::ALT)
    };
}

#[macro_export]
macro_rules! alt_ctrl {
    ($ch:expr, $key_event:expr) => {
        $key_event.code == KeyCode::Char($ch)
            && $key_event.modifiers.contains(KeyModifiers::ALT)
            && $key_event.modifiers.contains(KeyModifiers::CONTROL)
    };
}
