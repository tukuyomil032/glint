use crate::cleaner::CleanSummary;
use crate::cli::Language;
use crate::scanner::{
    CleanupSummary, DuplicateModsSummary, UnusedAssetsSummary, UnusedLibrariesSummary,
    UsageSummary, WorldsSummary,
};
use console::{Emoji, style};
use indicatif::HumanBytes;
use tabled::{Table, Tabled, settings::Style};

pub fn human_bytes(bytes: u64) -> String {
    HumanBytes(bytes).to_string()
}

#[derive(Tabled)]
struct CleanupRow {
    target: String,
    size: String,
    path: String,
}

pub fn print_cleanup(
    summary: &CleanupSummary,
    unused_libs: &UnusedLibrariesSummary,
    unused_assets: &UnusedAssetsSummary,
    lang: Language,
) {
    let spark = Emoji("✨", "*");
    let title = match lang {
        Language::En => "PrismLauncher Scan Report",
        Language::Ja => "PrismLauncher スキャンレポート",
    };
    println!("{} {}\n", spark, style(title).bold().cyan());

    let safe_header = match lang {
        Language::En => "Safe Cleanup Targets",
        Language::Ja => "安全に削除できる対象",
    };
    println!("{}", style(safe_header).bold().underlined());

    let rows: Vec<CleanupRow> = summary
        .entries
        .iter()
        .map(|entry| CleanupRow {
            target: entry.label.clone(),
            size: human_bytes(entry.bytes),
            path: entry.path.display().to_string(),
        })
        .collect();

    let mut safe_table = Table::new(rows);
    safe_table.with(Style::rounded());
    println!("{}", safe_table);

    let safe_total = match lang {
        Language::En => "Safe reclaimable total",
        Language::Ja => "安全対象の合計",
    };
    println!(
        "{}: {}\n",
        style(safe_total).bold(),
        human_bytes(summary.total_bytes)
    );

    print_unused_libraries(unused_libs, lang);
    print_unused_assets(unused_assets, lang);
}

#[derive(Tabled)]
struct ModRow {
    mod_name: String,
    size: String,
    instances: String,
}

pub fn print_mods(summary: &DuplicateModsSummary, lang: Language) {
    let rows: Vec<ModRow> = summary
        .duplicates
        .iter()
        .map(|entry| ModRow {
            mod_name: entry.mod_name.clone(),
            size: human_bytes(entry.bytes),
            instances: entry.instances.join(", "),
        })
        .collect();

    if rows.is_empty() {
        let msg = match lang {
            Language::En => "No duplicate mods detected.",
            Language::Ja => "重複modは見つかりませんでした。",
        };
        println!("{}", msg);
        return;
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);

    let dup_groups = match lang {
        Language::En => "Duplicate groups",
        Language::Ja => "重複グループ数",
    };
    let reclaimable = match lang {
        Language::En => "Potential reclaimable",
        Language::Ja => "削減可能見込み",
    };

    println!("\n{}: {}", dup_groups, summary.duplicate_groups);
    println!(
        "{}: {}",
        reclaimable,
        human_bytes(summary.potential_reclaim_bytes)
    );
}

#[derive(Tabled)]
struct WorldRow {
    instance: String,
    world: String,
    size: String,
}

pub fn print_worlds(summary: &WorldsSummary, lang: Language) {
    let rows: Vec<WorldRow> = summary
        .worlds
        .iter()
        .map(|world| WorldRow {
            instance: world.instance.clone(),
            world: world.world.clone(),
            size: human_bytes(world.bytes),
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);

    let total = match lang {
        Language::En => "Total world size",
        Language::Ja => "ワールド合計サイズ",
    };
    println!("\n{}: {}", total, human_bytes(summary.total_world_bytes));
}

#[derive(Tabled)]
struct UsageRow {
    instance: String,
    size: String,
}

pub fn print_usage(summary: &UsageSummary, lang: Language) {
    let rows: Vec<UsageRow> = summary
        .instances
        .iter()
        .map(|row| UsageRow {
            instance: row.instance.clone(),
            size: human_bytes(row.bytes),
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);

    let total = match lang {
        Language::En => "Total instances size",
        Language::Ja => "インスタンス合計サイズ",
    };
    println!("\n{}: {}", total, human_bytes(summary.total_bytes));
}

#[derive(Tabled)]
struct CleanRow {
    target: String,
    action: String,
    size: String,
    success: String,
    message: String,
}

pub fn print_clean(summary: &CleanSummary, lang: Language) {
    let rows: Vec<CleanRow> = summary
        .entries
        .iter()
        .map(|entry| CleanRow {
            target: entry.label.clone(),
            action: entry.action.clone(),
            size: human_bytes(entry.bytes),
            success: if entry.success { "yes" } else { "no" }.to_string(),
            message: entry.message.clone(),
        })
        .collect();

    if !rows.is_empty() {
        let mut table = Table::new(rows);
        table.with(Style::rounded());
        println!("{}", table);
        println!();
    }

    if summary.dry_run {
        let label = match lang {
            Language::En => "Dry-run reclaimable",
            Language::Ja => "dry-runで削減可能",
        };
        println!("{}: {}", label, human_bytes(summary.cleaned_bytes));
    } else {
        let label = match lang {
            Language::En => "Cleaned",
            Language::Ja => "削除済み",
        };
        println!("{}: {}", label, human_bytes(summary.cleaned_bytes));
    }
}

#[derive(Tabled)]
struct CandidateRow {
    path: String,
    size: String,
}

fn print_unused_libraries(summary: &UnusedLibrariesSummary, lang: Language) {
    let title = match lang {
        Language::En => "Detected Unused Libraries",
        Language::Ja => "未使用の可能性があるlibraries",
    };
    println!("{}", style(title).bold().underlined());

    if summary.candidates.is_empty() {
        let msg = match lang {
            Language::En => "No candidates found (or references unavailable).",
            Language::Ja => "候補は見つかりませんでした（または参照情報が不足）。",
        };
        println!("{}\n", msg);
        return;
    }

    let rows: Vec<CandidateRow> = summary
        .candidates
        .iter()
        .take(20)
        .map(|entry| CandidateRow {
            path: entry.relative_path.clone(),
            size: human_bytes(entry.bytes),
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);

    let footer = match lang {
        Language::En => "Unused libraries reclaimable",
        Language::Ja => "unused librariesの合計",
    };
    println!("{}: {}\n", footer, human_bytes(summary.total_bytes));
}

fn print_unused_assets(summary: &UnusedAssetsSummary, lang: Language) {
    let title = match lang {
        Language::En => "Detected Unused Assets",
        Language::Ja => "未使用の可能性があるassets",
    };
    println!("{}", style(title).bold().underlined());

    if summary.candidates.is_empty() {
        let msg = match lang {
            Language::En => "No candidates found (or index references unavailable).",
            Language::Ja => "候補は見つかりませんでした（またはindex参照が不足）。",
        };
        println!("{}\n", msg);
        return;
    }

    let rows: Vec<CandidateRow> = summary
        .candidates
        .iter()
        .take(20)
        .map(|entry| CandidateRow {
            path: entry.path.display().to_string(),
            size: human_bytes(entry.bytes),
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);

    let footer = match lang {
        Language::En => "Unused assets reclaimable",
        Language::Ja => "unused assetsの合計",
    };
    println!("{}: {}\n", footer, human_bytes(summary.total_bytes));
}
