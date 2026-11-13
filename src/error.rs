#[derive(Debug, PartialEq, Eq)]
pub enum ErrorNM {
    NoPermissionCreateGroup,
    ObjectNotFound,
    UnhandledError,
    ParentNotExist,
    GroupExists,
    ItemExists,
    InvalidObject,
}
