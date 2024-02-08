extern crate self as clift;
mod commands;
mod error;

pub use error::{Error, Result};

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

async fn async_main() -> clift::Result<()> {
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
                .arg(clap::arg!(site: <SITE> "The site of the package to upload. Default value is taken from FASTN.ftd").required(false)),
        )
}

async fn clift_commands(matches: &clap::ArgMatches) -> clift::Result<()> {
    if let Some(upload) = matches.subcommand_matches("upload") {
        let site = upload.get_one::<String>("site");
        return Ok(clift::commands::upload(site).await?);
    }

    Ok(())
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

pub(crate) const API_FIFTHTRY_COM: &str = "https://api.fifthtry.com";
