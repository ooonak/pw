pub mod machine;

mod utils;

pub mod pw {
    pub mod messages {
        include!(concat!(env!("OUT_DIR"), "/pw.messages.rs"));
    }
}
