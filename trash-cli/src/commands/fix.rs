use crate::cli::FixArgs;
use anyhow::Context;
use xdg_trash::UnifiedTrash;

pub fn fix(_args: &FixArgs) -> anyhow::Result<()> {
    let trash = UnifiedTrash::new().context("Failed to init trash")?;
    let amount = trash.fix().context("Failed to fix trashinfo files")?;
    println!("Removed {} trashinfo files", amount);

    Ok(())
}
