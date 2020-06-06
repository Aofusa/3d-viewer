#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Id {
    pub id: u128,
}

pub trait IdRepository {
    fn generate(&self) -> Id;
}

pub struct IdInteractor<T>
where
    T: IdRepository,
{
    pub id_repository: T,
}

impl<T: IdRepository> IdInteractor<T> {
    pub fn generate(&self) -> Id {
        self.id_repository.generate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockIdRepository;

    impl IdRepository for MockIdRepository {
        fn generate(&self) -> Id {
            Id { id: 0 }
        }
    }

    #[test]
    fn id_generate() {
        let id_repository = MockIdRepository {};
        let id_interactor = IdInteractor { id_repository };
        let id = id_interactor.generate();
        assert_eq!(id.id, 0);
    }
}

