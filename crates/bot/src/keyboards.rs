use teloxide::types::{KeyboardButton, KeyboardMarkup, ButtonRequest};

pub fn request_phone_keyboard() -> KeyboardMarkup {
    let button = KeyboardButton::new("📱 ارسال شماره تلفن")
        .request(ButtonRequest::Contact);

    KeyboardMarkup {
        keyboard: vec![vec![button]],
        resize_keyboard: true,
        one_time_keyboard: true,
        is_persistent: false,
        input_field_placeholder: "ارسال شماره تلفن".to_string(),
        selective: false,
    }
}
