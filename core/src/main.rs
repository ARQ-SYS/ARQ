use arq_components::pluggable::manager::ComponentManager;


#[tokio::main]
async fn main() {
    unsafe {
        let mut manager = ComponentManager::new();
        manager.load_middleware("./target/debug/libsample_middleware.so").unwrap();

        let middlewares = manager.get_middlewares();
        println!("Loaded {} middlewares", middlewares.len());

    }
}