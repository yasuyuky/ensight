use crate::get;
use crate::insight::{InsightItem, Insights, Item, Items};
use crate::vcs::Vcs;
use colored::{Color, Colorize};
use colorgrad::{preset, Gradient};

pub async fn print_all(vcs: &Vcs, slug: &str, sort: bool, n: Option<usize>) -> anyhow::Result<()> {
    let path = format!("insights/{}/{}/workflows", &vcs, &slug);
    let token = std::env::var("CIRCLECI_TOKEN")?;
    let result = get::<Insights>(&token, &path).await;
    if let Ok(insights) = result {
        let l = n.unwrap_or_else(|| insight_name_width(&insights.items));
        for insight in &insights.items {
            let path = format!("insights/{}/{}/workflows/{}", &vcs, &slug, insight.name);
            let result = get::<Items>(&token, &path)
                .await
                .map_err(|err| anyhow::anyhow!("failed to fetch {path}: {err}"))?;
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
            sort_by_success_rate(&mut insights.items);
        }
        let l = n.unwrap_or_else(|| insight_name_width(&insights.items));
        for insight in insights.items {
            let path = format!(
                "insights/{}/{}/workflows/{}/jobs/{}",
                &vcs, &slug, workflow, insight.name
            );
            let result = get::<Items>(&token, &path)
                .await
                .map_err(|err| anyhow::anyhow!("failed to fetch {path}: {err:?}"))?;
            print!("Job:");
            print_gr(l, &result.items, &insight.name);
            print_insight(&insight);
        }
    } else {
        println!("{result:#?}");
    }
    Ok(())
}

fn insight_name_width(items: &[InsightItem]) -> usize {
    items.iter().map(|item| item.name.len()).max().unwrap_or(0)
}

fn sort_by_success_rate(items: &mut [InsightItem]) {
    items.sort_by(|a, b| a.metrics.success_rate.total_cmp(&b.metrics.success_rate));
}

fn print_insight(insight: &InsightItem) {
    let c = preset::warm();
    let [r, g, b, _] = c.at(insight.metrics.success_rate as f32).to_rgba8();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::insight::{DurationMetrics, Metrics};
    use time::OffsetDateTime;

    fn insight(name: &str, success_rate: f64) -> InsightItem {
        InsightItem {
            name: name.to_string(),
            metrics: Metrics {
                success_rate,
                total_runs: 0,
                failed_runs: 0,
                successful_runs: 0,
                throughput: 0.0,
                duration_metrics: DurationMetrics {
                    min: 0,
                    max: 0,
                    median: 0,
                    mean: 0,
                    p95: 0,
                    standard_deviation: 0.0,
                },
                total_credits_used: 0,
            },
            window_start: OffsetDateTime::UNIX_EPOCH,
            window_end: OffsetDateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn insight_name_width_is_zero_for_empty_items() {
        assert_eq!(insight_name_width(&[]), 0);
    }

    #[test]
    fn insight_name_width_uses_longest_name() {
        let items = vec![insight("build", 1.0), insight("integration", 1.0)];

        assert_eq!(insight_name_width(&items), "integration".len());
    }

    #[test]
    fn sort_by_success_rate_handles_nan_without_panic() {
        let mut items = vec![
            insight("ok", 1.0),
            insight("nan", f64::NAN),
            insight("failed", 0.0),
        ];

        sort_by_success_rate(&mut items);

        assert_eq!(items[0].name, "failed");
        assert_eq!(items[1].name, "ok");
        assert_eq!(items[2].name, "nan");
    }
}
