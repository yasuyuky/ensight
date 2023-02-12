use crate::get;
use crate::insight::{InsightItem, Insights, Item, Items};
use crate::vcs::Vcs;
use colored::{Color, Colorize};

pub async fn print_all(vcs: &Vcs, slug: &str, sort: bool, n: Option<usize>) -> anyhow::Result<()> {
    let path = format!("insights/{}/{}/workflows", &vcs, &slug);
    let token = std::env::var("CIRCLECI_TOKEN")?;
    let result = get::<Insights>(&token, &path).await;
    if let Ok(insights) = result {
        let l = n.unwrap_or_else(|| insights.items.iter().map(|i| i.name.len()).max().unwrap());
        for insight in &insights.items {
            let path = format!("insights/{}/{}/workflows/{}", &vcs, &slug, insight.name);
            let result = get::<Items>(&token, &path).await.unwrap();
            println!();
            print!("Workflow:");
            print_gr(l, &result.items, &insight.name);
            print_insight(insight);
            print_jobs(vcs, slug, &insight.name, sort, n).await?;
        }
    }
    Ok(())
}

async fn print_jobs(
    vcs: &Vcs,
    slug: &str,
    workflow: &str,
    sort: bool,
    n: Option<usize>,
) -> anyhow::Result<()> {
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
        let l = n.unwrap_or_else(|| insights.items.iter().map(|i| i.name.len()).max().unwrap());
        for insight in insights.items {
            let path = format!(
                "insights/{}/{}/workflows/{}/jobs/{}",
                &vcs, &slug, workflow, insight.name
            );
            let result = get::<Items>(&token, &path).await.unwrap();
            print!("Job:");
            print_gr(l, &result.items, &insight.name);
            print_insight(&insight);
        }
    } else {
        println!("{result:#?}");
    }
    Ok(())
}

fn print_insight(insight: &InsightItem) {
    let c = colorgrad::warm();
    let [r, g, b, _] = c.at(insight.metrics.success_rate).to_rgba8();
    let runs = format!(
        " {:3}/{:3} {:7.3}% ",
        insight.metrics.successful_runs,
        insight.metrics.total_runs,
        insight.metrics.success_rate * 100f64,
    );
    let credits = format!(
        "avg.{:4} sec. {:7} cr. ${:8.4}",
        insight.metrics.duration_metrics.mean,
        insight.metrics.total_credits_used,
        insight.metrics.total_credits_used as f64 * 0.0006,
    );
    let runtext = runs.truecolor(31, 31, 31).on_truecolor(r, g, b);
    println!("{runtext} {credits}");
}

fn print_gr(l: usize, items: &[Item], s: &str) {
    let size = items.len();
    for i in 0..l {
        // [0123456789]
        //      [01234]
        //    ^      ^
        //    0   i  l
        //        ^ size - l + i (must be positive)
        let idx = if l < size + i { size + i - l } else { size };
        let styles = items
            .get(idx)
            .map(|item| match item.status.as_deref() {
                Some("success") => (Color::Black, Color::Green),
                Some("failed") => (Color::Black, Color::Red),
                _ => (Color::Black, Color::Yellow),
            })
            .unwrap_or_else(|| (Color::Black, Color::White));
        let c = s.get(i..i + 1).unwrap_or(" ");
        print!("{}", c.color(styles.0).on_color(styles.1))
    }
}
