extern crate uuid;

mod component;
mod driver;

use component::id::{IdInteractor};
use driver::uuid_driver::UuidRepository;

fn main() {
    let id_repository = UuidRepository {};
    let id_interactor = IdInteractor { id_repository };
    let id = id_interactor.generate();

    println!("{:?}", id);
}
