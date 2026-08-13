pub mod app;
pub mod config;
pub mod error;
pub mod middleware;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;

pub use app::build_router;
pub use config::AppConfig;
pub use repositories::{MemoryRepository, Repository};
pub use services::openai::{CloudProvider, FakeProvider, Provider};
pub use services::tutor::{FakeTutorProvider, OpenRouterTutorProvider, TutorProvider};
pub use state::AppState;
