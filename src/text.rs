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
    let runs = format!(
        " {:3}/{:3} {:7.3}% ",
        insight.metrics.successful_runs,
        insight.metrics.total_runs,
        insight.metrics.success_rate * 100f64,
    );
    let credits = format!(
        "avg.{:4}sec. {:7}credits ${:8.4}",
        insight.metrics.duration_metrics.mean,
        insight.metrics.total_credits_used,
        insight.metrics.total_credits_used as f64 * 0.0006,
    );
    println!("{} {}", style.paint(runs), credits);
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
        let style = items
            .get(idx)
            .map(|item| match item.status.as_deref() {
                Some("success") => Colour::Black.on(Colour::Green),
                Some("failed") => Colour::Black.on(Colour::Red),
                _ => Colour::Black.on(Colour::Yellow),
            })
            .unwrap_or(Colour::Black.on(Colour::White));
        let c = s.get(i..i + 1).unwrap_or(" ");
        print!("{}", style.paint(c))
    }
}
