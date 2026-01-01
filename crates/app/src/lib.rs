//! Application layer (business logic)
//!
//! This crate contains:
//! - Domain models
//! - Repository contracts (traits)
//! - Use-cases (services)
//!
//! ❌ No database code
//! ❌ No Telegram / HTTP / UI code
//! ✅ Pure business logic

/* ───────────────────── Models (domain) ───────────────────── */

pub mod models {
    pub mod user;
    pub mod cart;
    pub mod menu;
    pub mod order;
}

/* ───────────────────── Repository contracts ───────────────────── */

pub mod repos {
    pub mod user;
    pub mod cart;
    pub mod menu;
    pub mod order;
}

/* ───────────────────── Use-cases (services) ───────────────────── */

pub mod services {
    pub mod users {
        pub mod service;
        pub mod error;
        pub mod phone;
    }

    pub mod cart {
        pub mod service;
        pub mod error;
    }

    pub mod menu {
        pub mod service;
        pub mod error;
    }

    pub mod order {
        pub mod service;
        pub mod error;
    }
}
