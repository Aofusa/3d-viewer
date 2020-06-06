use uuid::Uuid;
use super::super::component::id::{Id, IdRepository};

pub struct UuidRepository;

impl IdRepository for UuidRepository {
    fn generate(&self) -> Id {
        let uuid = Uuid::new_v4();
        let id = uuid.to_u128_le();
        Id { id: id }
    }
}

