use serde::de::DeserializeOwned;
use structopt::StructOpt;

mod insight;
mod text;
mod vcs;
use vcs::Vcs;

#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
    #[structopt(long = "vcs", default_value = "gh")]
    vcs: Vcs,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Command {
    Print {
        slug: String,
        #[structopt(long = "sort")]
        sort: bool,
    },
}

async fn get<T: DeserializeOwned>(token: &str, path: &str) -> surf::Result<T> {
    let uri = format!("https://circleci.com/api/v2/{}", &path);
    let value = format!("Basic {}", base64::encode(format!("{}:", &token)));
    let mut res = surf::get(&uri).header("Authorization", value).await?;
    Ok(res.body_json().await?)
}

#[async_std::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::from_args();
    match opt.command {
        Command::Print { slug, sort } => text::print_all(&opt.vcs, &slug, sort).await?,
    }
    Ok(())
}
