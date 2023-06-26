use anotify::handler;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anotify::Result<()> {
    let anotify = anotify::app::parse()?;
    match handler::run(anotify, None, tokio::signal::ctrl_c()).await {
        Ok(()) => return Ok(()),
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    };
}
