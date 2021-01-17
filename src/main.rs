use ansi_term::Colour;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize};
use std::fmt;
use structopt::StructOpt;

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

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "lower-case")]
enum Vcs {
    #[structopt(alias = "gh")]
    GitHub,
    #[structopt(alias = "bb")]
    BitBucket,
}

impl fmt::Display for Vcs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::GitHub => write!(f, "gh"),
            Self::BitBucket => write!(f, "bb"),
        }
    }
}

impl std::str::FromStr for Vcs {
    type Err = std::string::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gh" => Ok(Self::GitHub),
            "GitHub" => Ok(Self::GitHub),
            "bb" => Ok(Self::BitBucket),
            "BitBucket" => Ok(Self::BitBucket),
            _ => Ok(Self::GitHub),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Insights {
    items: Vec<InsightItem>,
}

#[derive(Debug, Deserialize)]
struct InsightItem {
    name: String,
    metrics: Metrics,
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct Metrics {
    success_rate: f64,
    total_runs: usize,
    failed_runs: usize,
    successful_runs: usize,
    throughput: f64,
    duration_metrics: DurationMetrics,
    total_credits_used: usize,
}

#[derive(Debug, Deserialize)]
struct DurationMetrics {
    min: usize,
    max: usize,
    median: usize,
    mean: usize,
    p95: usize,
    standard_deviation: f64,
}

#[derive(Debug, Deserialize)]
struct Items {
    items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    id: Option<String>,
    // created_at: DateTime<Utc>,
    // stopped_at: DateTime<Utc>,
    duration: usize,
    status: Option<String>,
    credits_used: usize,
}

async fn get<T: DeserializeOwned>(token: &str, path: &str) -> surf::Result<T> {
    let uri = format!("https://circleci.com/api/v2/{}", &path);
    let value = format!("Basic {}", base64::encode(format!("{}:", &token)));
    let mut res = surf::get(&uri).header("Authorization", value).await?;
    Ok(res.body_json().await?)
}

async fn print_all(vcs: &Vcs, slug: &str, sort: bool) -> anyhow::Result<()> {
    let path = format!("insights/{}/{}/workflows", &vcs, &slug);
    let token = std::env::var("CIRCLECI_TOKEN")?;
    let result = get::<Insights>(&token, &path).await;
    if let Ok(insights) = result {
        let c = colorgrad::warm();
        let l = insights.items.iter().map(|i| i.name.len()).max().unwrap();
        for insight in &insights.items {
            let path = format!("insights/{}/{}/workflows/{}", &vcs, &slug, insight.name);
            let result = get::<Items>(&token, &path).await.unwrap();
            println!("");
            print!("W:");
            print_gr(l, &result.items, &insight.name);
            print_insight(&c, insight);
            print_jobs(&vcs, slug, &insight.name, sort).await?;
        }
    }
    Ok(())
}

async fn print_jobs(vcs: &Vcs, slug: &str, workflow: &str, sort: bool) -> anyhow::Result<()> {
    let path = format!("insights/{}/{}/workflows/{}/jobs", &vcs, &slug, workflow);
    let token = std::env::var("CIRCLECI_TOKEN")?;
    let result = get::<Insights>(&token, &path).await;
    if let Ok(mut insights) = result {
        if sort {
            insights.items.sort_by(|a, b| {
                a.metrics
                    .success_rate
                    .partial_cmp(&b.metrics.success_rate)
                    .unwrap()
            });
        }
        let c = colorgrad::warm();
        let l = insights.items.iter().map(|i| i.name.len()).max().unwrap();
        for insight in insights.items {
            let path = format!(
                "insights/{}/{}/workflows/{}/jobs/{}",
                &vcs, &slug, workflow, insight.name
            );
            let result = get::<Items>(&token, &path).await.unwrap();
            print!("J:");
            print_gr(l, &result.items, &insight.name);
            print_insight(&c, &insight);
        }
    } else {
        println!("{:#?}", result);
    }
    Ok(())
}

fn print_insight(c: &colorgrad::Gradient, insight: &InsightItem) {
    let (r, g, b, _) = c.at(insight.metrics.success_rate).rgba_u8();
    let style = Colour::RGB(31, 31, 31).on(Colour::RGB(r, g, b));
    let s = format!(
        " {:3}/{:3} {:7.3}% ",
        insight.metrics.successful_runs,
        insight.metrics.total_runs,
        insight.metrics.success_rate * 100f64,
    );
    let t = format!(
        "avg.{:4}sec. {:7}credits ${:8.4}",
        insight.metrics.duration_metrics.mean,
        insight.metrics.total_credits_used,
        insight.metrics.total_credits_used as f64 * 0.0006,
    );
    println!("{} {}", style.paint(s), t);
}

fn print_gr(l: usize, items: &[Item], s: &str) {
    let success_style = Colour::Black.on(Colour::Green);
    let failure_style = Colour::Black.on(Colour::Red);
    let unknown_style = Colour::Black.on(Colour::Yellow);
    for (i, item) in items.iter().take(l).enumerate() {
        let c = s.get(i..i + 1).unwrap_or(" ");
        match item.status.as_ref().and_then(|s| Some(s.as_str())) {
            Some("success") => print!("{}", success_style.paint(c)),
            Some("failed") => print!("{}", failure_style.paint(c)),
            _ => print!("{}", unknown_style.paint(c)),
        }
    }
}

#[async_std::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::from_args();
    match opt.command {
        Command::Print { slug, sort } => print_all(&opt.vcs, &slug, sort).await?,
    }
    Ok(())
}
