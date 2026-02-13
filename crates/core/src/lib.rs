#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod types;
pub mod upgrade;

mod agent;
mod annoyed;
mod pack;
mod routing;
mod sound;
pub mod tab_title;

pub use agent::is_agent_session;
pub use annoyed::check_annoyed;
pub use pack::resolve_pack;
pub use routing::route_event;
pub use sound::pick_sound;
pub use tab_title::build_tab_title;
