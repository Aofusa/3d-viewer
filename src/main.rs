use artichoke_backend::prelude::core::*;
use artichoke_backend::prelude::*;

fn example() -> Result<(), Exception> {
    let mut interp = artichoke_backend::interpreter()?;
    let result = interp.eval(b"10 * 10")?;
    let result = result.try_into::<i64>(&interp)?;
    assert_eq!(100, result);
    Ok(())
}

fn main() {
    example();
    println!("Hello, world!");
}
