#[derive(Debug)]
pub struct CallbackData {
    pub feature: String,
    pub action: String,
    pub payload: Vec<String>,
}

impl CallbackData {
    pub fn parse(data: &str) -> Option<Self> {
        let mut parts = data.split(':');

        let feature = parts.next()?.to_string();
        let action = parts.next()?.to_string();
        let payload = parts.map(|s| s.to_string()).collect();

        Some(Self {
            feature,
            action,
            payload,
        })
    }
}
