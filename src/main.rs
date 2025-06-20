mod blackjack;
mod blackjack_cli;

use blackjack_cli::{BlackjackCli};
use std::io;


fn main() -> io::Result<()> {
    let mut blackjack = BlackjackCli::new();
    blackjack.run()?;

    Ok(())
}
