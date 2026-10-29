pub mod error;
pub mod models;
pub mod repo;

pub use error::DriverError;
pub use models::{GitStatus, QualityAudit, TicketDetails};
pub use repo::{
    parse_quality_audit, parse_ticket_creation_output, CccpGitDriver, RepoDriver,
};
