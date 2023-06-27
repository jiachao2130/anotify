use anotify_rs::handler;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anotify_rs::Result<()> {
    let anotify = anotify_rs::app::parse()?;
    match handler::run(anotify, None, tokio::signal::ctrl_c()).await {
        Ok(()) => return Ok(()),
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    };
}
