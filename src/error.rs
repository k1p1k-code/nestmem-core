#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum ErrorNM {
    NoPermissionCreateGroup,
    NoFindObject,
    UnhandledError,
}
