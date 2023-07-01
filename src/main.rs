use async_inotify::handler;

#[tokio::main(flavor = "current_thread")]
async fn main() -> async_inotify::Result<()> {
    let anotify = async_inotify::app::parse()?;
    match handler::run(anotify, None, tokio::signal::ctrl_c()).await {
        Ok(()) => return Ok(()),
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    };
}
