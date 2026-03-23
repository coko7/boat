use crate::repository::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: Id,
    pub name: String,
}

#[derive(Debug)]
pub struct NewTag {
    pub name: String,
}
