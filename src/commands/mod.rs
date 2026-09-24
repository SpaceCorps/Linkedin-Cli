//! Command implementations and entrypoints.

pub mod accounts;
pub mod login;
pub mod me;
pub mod post;
pub mod profile;

use crate::cli::Command;
use crate::error::Result;
use crate::readme;

pub fn run(cmd: Command) -> Result<()> {
    match cmd {
        Command::Profile(args) => profile::run(args),
        Command::Post(args) => post::run(args),
        Command::Me(args) => me::run(args),
        Command::Login(args) => login::run(args),
        Command::Accounts(args) => accounts::run(args),
        Command::AgentReadme => {
            readme::print();
            Ok(())
        }
    }
}
