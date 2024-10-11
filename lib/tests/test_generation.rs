use std::error::Error;
use tempfile::tempdir;

#[test]
fn basic_config() -> Result<(), Box<dyn Error>> {
    let dir = tempdir()?;
    ethshadow::generate(include_str!("configs/basic.yaml"), dir.path(), true)?;
    // TODO: add some assertions here
    Ok(())
}
