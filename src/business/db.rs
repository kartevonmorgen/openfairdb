pub trait Repo<T> {
    type Id;
    type Error;

    fn get(&self, Self::Id) -> Result<T, Self::Error>;
    fn all(&self) -> Result<Vec<T>, Self::Error>;
    fn save(&mut self, &T) -> Result<(), Self::Error>;
}
