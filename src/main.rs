mod common;
mod send;

use common::Group;

#[tokio::main]
async fn main() {
    let mut main_group = Group::new_main();
    let test_group = Group::new("order".to_string(), None, true);
    main_group.new_group("order".to_string(), test_group).unwrap();
    println!("{:?}", main_group)
}
