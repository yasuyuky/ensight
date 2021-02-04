use crate::get;
use crate::insight::{InsightItem, Insights, Item, Items};
use crate::vcs::Vcs;
use ansi_term::Colour;

pub async fn print_all(vcs: &Vcs, slug: &str, sort: bool) -> anyhow::Result<()> {
    let path = format!("insights/{}/{}/workflows", &vcs, &slug);
    let token = std::env::var("CIRCLECI_TOKEN")?;
    let result = get::<Insights>(&token, &path).await;
    if let Ok(insights) = result {
        let c = colorgrad::warm();
        let l = insights.items.iter().map(|i| i.name.len()).max().unwrap();
        for insight in &insights.items {
            let path = format!("insights/{}/{}/workflows/{}", &vcs, &slug, insight.name);
            let result = get::<Items>(&token, &path).await.unwrap();
            println!();
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
        match item.status.as_deref() {
            Some("success") => print!("{}", success_style.paint(c)),
            Some("failed") => print!("{}", failure_style.paint(c)),
            _ => print!("{}", unknown_style.paint(c)),
        }
    }
}
