pub trait Repo {
    type Id;
    type Connection;
    type Error;

    fn get(&Self::Connection, Self::Id) -> Result<Self, Self::Error> where Self: Sized;
    fn all(&Self::Connection) -> Result<Vec<Self>, Self::Error> where Self: Sized;
    fn save(&self, &Self::Connection) -> Result<Self, Self::Error> where Self: Sized;
}
