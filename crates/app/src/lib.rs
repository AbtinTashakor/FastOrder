/* ───────────────────── Models (domain) ───────────────────── */

pub mod models {
    pub mod user;
    pub mod cart;
    pub mod menu;
    pub mod order;
    pub mod operator_state;
}

/* ───────────────────── Repository contracts (policy) ───────────────────── */

pub mod repos {
    pub mod user;
    pub mod cart;
    pub mod menu;
    pub mod order;

    // operator / dispatch
    pub mod operator_state;
    pub mod operator_directory;
    pub mod system_state;
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

    /* operator workflow */

    pub mod operator_state {
        pub mod service;
    }

    pub mod assign {
        pub mod service;
    }
}
