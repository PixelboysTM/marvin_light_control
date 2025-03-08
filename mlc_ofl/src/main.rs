use std::path::Path;
use mlc_ofl::create_lib;


#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    create_lib(Path::new("./lib_test.json"), true).await.unwrap();
}