use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use clap::Parser;
use serde::de::DeserializeOwned;

mod insight;
mod text;
mod vcs;
use vcs::Vcs;

#[derive(Parser)]
struct Opt {
    #[clap(subcommand)]
    command: Command,
    #[clap(long = "vcs", default_value = "gh")]
    vcs: Vcs,
}

#[derive(Debug, Parser)]
#[clap(rename_all = "kebab-case")]
enum Command {
    Print {
        slug: String,
        #[clap(long = "sort")]
        sort: bool,
        #[clap(short = 'l', long = "length")]
        l: Option<usize>,
    },
}

async fn get<T: DeserializeOwned>(token: &str, path: &str) -> surf::Result<T> {
    let uri = format!("https://circleci.com/api/v2/{}", &path);
    let value = format!("Basic {}", BASE64.encode(format!("{}:", &token)));
    let mut res = surf::get(&uri).header("Authorization", value).await?;
    res.body_json().await
}

#[async_std::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();
    match opt.command {
        Command::Print { slug, sort, l } => text::print_all(&opt.vcs, &slug, sort, l).await?,
    }
    Ok(())
}
