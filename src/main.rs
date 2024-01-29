fn main() {
    fastn_observer::observe();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(outer_main())
}


async fn outer_main() {
    if let Err(e) = async_main().await {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CliftError")]
    CliftError,
}

async fn async_main() -> Result<(), Error> {
    let matches = app(version()).get_matches();

    clift_commands(&matches).await?;

    Ok(())
}



fn app(version: &'static str) -> clap::Command {
    clap::Command::new("clift: fastn Package on ft")
        .version(version)
        .arg_required_else_help(true)
        .subcommand(
            clap::Command::new("upload")
                .about("Uploads fastn package on ft")
        )
}


async fn clift_commands(_matches: &clap::ArgMatches) -> Result<(), Error> {
    todo!()
}




pub fn version() -> &'static str {
    if std::env::args().any(|e| e == "--test") {
        env!("CARGO_PKG_VERSION")
    } else {
        match option_env!("GITHUB_SHA") {
            Some(sha) => {
                Box::leak(format!("{} [{}]", env!("CARGO_PKG_VERSION"), sha).into_boxed_str())
            }
            None => env!("CARGO_PKG_VERSION"),
        }
    }
}
