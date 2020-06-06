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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_generate() {
        let id_repository = UuidRepository {};
        let id = id_repository.generate();
        assert_eq!(id.id > 0, true);
    }
}

