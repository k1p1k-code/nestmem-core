pub mod storage;

use storage::group::Group;

pub struct NestMem{
    root: Group
}

impl NestMem {
    fn new() -> Self{
        Self { 
            root: Group::new_root()
         }
    }
}