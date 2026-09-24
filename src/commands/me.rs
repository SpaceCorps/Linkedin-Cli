//! `linkedin me`. Displays current authenticated identity and token status.

use crate::account::{self, identity};
use crate::cli::MeArgs;
use crate::error::Result;
use crate::obj;
use crate::output;

pub fn run(args: MeArgs) -> Result<()> {
    let account = account::resolve(args.account.as_deref(), args.api_key.as_deref())?;
    let me = account.client().get("users/me")?;
    let ident = identity::describe(&me);
    let uname = identity::username(&me);

    output::write(&obj! {
        "account" => account.name,
        "identity" => ident,
        "username" => uname,
        "keyStatus" => "valid",
        "user" => me,
    });
    Ok(())
}
