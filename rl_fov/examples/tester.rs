use std::io::{stdout, Write};

use crossterm::{
    event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand, Result,
};

fn main() -> Result<()> {
    execute!(stdout(),
             SetForegroundColor(Color::Blue),
             SetBackgroundColor(Color::Red),
             Print("Styled text here."),
             ResetColor)?;
    Ok(())
}
