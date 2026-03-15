use crate::cleaner::CleanSummary;
use crate::cli::Language;
use crate::i18n::{Msg, text};
use crate::scanner::{DuplicateModsSummary, UsageSummary, WorldsSummary};

use super::human_bytes;

pub fn print_mods(summary: &DuplicateModsSummary, lang: Language) {
    if summary.duplicates.is_empty() {
        println!("{}", text(lang, Msg::NoDuplicateMods));
        return;
    }

    println!("{}", text(lang, Msg::DuplicateMods));

    for entry in &summary.duplicates {
        println!("- {} ({})", entry.mod_name, human_bytes(entry.bytes));
        println!("  {}", entry.instances.join(", "));
    }

    println!(
        "{}: {}",
        text(lang, Msg::DuplicateGroups),
        summary.duplicate_groups
    );
    println!(
        "{}: {}",
        text(lang, Msg::PotentialReclaimable),
        human_bytes(summary.potential_reclaim_bytes)
    );
}

pub fn print_worlds(summary: &WorldsSummary, lang: Language) {
    if summary.worlds.is_empty() {
        println!("{}", text(lang, Msg::NoWorldsDetected));
    } else {
        println!("{}", text(lang, Msg::Worlds));
        for row in &summary.worlds {
            println!(
                "- {}/{} ({})",
                row.instance,
                row.world,
                human_bytes(row.bytes)
            );

            if !row.breakdown.is_empty() {
                for part in row.breakdown.iter().take(6) {
                    println!("  - {}: {}", part.bucket, human_bytes(part.bytes));
                }
            }
        }
    }

    println!(
        "{}: {}",
        text(lang, Msg::TotalWorldSize),
        human_bytes(summary.total_world_bytes)
    );
}

pub fn print_usage(summary: &UsageSummary, lang: Language) {
    println!("{}", text(lang, Msg::InstanceUsage));

    for row in &summary.instances {
        println!("- {}: {}", row.instance, human_bytes(row.bytes));
    }

    println!(
        "{}: {}",
        text(lang, Msg::TotalInstanceSize),
        human_bytes(summary.total_bytes)
    );
}

pub fn print_clean(summary: &CleanSummary, lang: Language) {
    if !summary.entries.is_empty() {
        println!("{}", text(lang, Msg::CleanupResult));
        for entry in &summary.entries {
            println!(
                "- {} [{}] {} ({}) {}",
                entry.label,
                entry.action,
                human_bytes(entry.bytes),
                if entry.success { "ok" } else { "ng" },
                entry.message
            );
        }
    }

    println!(
        "{}: {}",
        if summary.dry_run {
            text(lang, Msg::DryRunReclaimable)
        } else {
            text(lang, Msg::Cleaned)
        },
        human_bytes(summary.cleaned_bytes)
    );
}
