use anotify::watcher;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let _ = match watcher::run().await {
        Ok(()) => (),
        Err(err) => {
            println!("Error: {}", err);
            std::process::exit(1);
        }
    };
}
