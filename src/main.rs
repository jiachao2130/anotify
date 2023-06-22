use anotify::handler;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let _ = match handler::run().await {
        Ok(()) => (),
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    };
}
