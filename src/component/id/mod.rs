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

