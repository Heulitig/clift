fn main() {
    fastn_observer::observe();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(outer_main())
}

async fn outer_main() {
    let matches = app(clift::utils::version()).get_matches();
    clift_commands(&matches).await
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

async fn clift_commands(matches: &clap::ArgMatches) {
    if let Some(upload) = matches.subcommand_matches("upload") {
        let site = upload.get_one::<String>("site");

        if let Err(e) = clift::commands::upload(site).await {
            eprintln!("Upload failed: {e}");
            std::process::exit(1);
        }
    }
}
