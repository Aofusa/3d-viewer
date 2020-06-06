extern crate uuid;
extern crate graphics;

use graphics::component::id::{IdInteractor};
use graphics::driver::uuid_driver::UuidRepository;

fn main() {
    let id_repository = UuidRepository {};
    let id_interactor = IdInteractor { id_repository };
    let id = id_interactor.generate();

    println!("{:?}", id);
}
