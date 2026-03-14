mod cleaner;
mod cli;
mod output;
mod prism;
mod scanner;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Command, Language};
use env_logger::Env;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use serde::Serialize;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:?}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    init_logging(cli.log_level, cli.verbose)?;

    if cli.verbose {
        debug!("arguments: {:?}", cli.command);
    }

    let root = prism::resolve_root(cli.path)?;
    let lang = cli.lang;

    if !root.exists() {
        let msg = match lang {
            Language::En => "PrismLauncher root does not exist",
            Language::Ja => "PrismLauncher rootが存在しません",
        };
        anyhow::bail!("{msg}: {}", root.display());
    }

    if cli.verbose {
        let prefix = match lang {
            Language::En => "root",
            Language::Ja => "ルート",
        };
        eprintln!("{prefix}: {}", root.display());
    }

    info!("root={}", root.display());

    match cli.command {
        Command::Scan => {
            let targets = prism::collect_cleanup_targets(&root);
            let summary = with_spinner(scan_cleanup_msg(lang), !cli.json, || {
                Ok(scanner::scan_cleanup_targets(&root, &targets))
            })?;

            let unused_libraries =
                with_spinner(scan_unused_libraries_msg(lang), !cli.json, || {
                    Ok(scanner::scan_unused_libraries(&root))
                })?;

            let unused_assets = with_spinner(scan_unused_assets_msg(lang), !cli.json, || {
                Ok(scanner::scan_unused_assets(&root))
            })?;

            if cli.json {
                let report = ScanJsonReport {
                    cleanup: summary,
                    unused_libraries,
                    unused_assets,
                };
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                output::print_cleanup(&summary, &unused_libraries, &unused_assets, lang);
            }
        }
        Command::Clean {
            dry_run,
            apply,
            yes,
            include_unused_libraries,
            include_unused_assets,
        } => {
            let mode = cli::CleanMode {
                dry_run: dry_run || !apply,
                yes,
                include_unused_libraries,
                include_unused_assets,
            };

            let mut targets = prism::collect_cleanup_targets(&root);

            if mode.include_unused_libraries {
                let libs = with_spinner(scan_unused_libraries_msg(lang), !cli.json, || {
                    Ok(scanner::scan_unused_libraries(&root))
                })?;
                targets.extend(scanner::cleanup_targets_from_unused_libraries(&libs, 2000));
            }

            if mode.include_unused_assets {
                let assets = with_spinner(scan_unused_assets_msg(lang), !cli.json, || {
                    Ok(scanner::scan_unused_assets(&root))
                })?;
                targets.extend(scanner::cleanup_targets_from_unused_assets(&assets, 5000));
            }

            let summary = with_spinner(clean_targets_msg(lang), !cli.json, || {
                cleaner::run_clean(&root, &targets, mode.dry_run, mode.yes, lang)
            })?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                output::print_clean(&summary, lang);
            }
        }
        Command::Mods => {
            let summary = with_spinner(scan_duplicate_mods_msg(lang), !cli.json, || {
                Ok(scanner::scan_duplicate_mods(&root))
            })?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                output::print_mods(&summary, lang);
            }
        }
        Command::Worlds => {
            let summary = with_spinner(scan_worlds_msg(lang), !cli.json, || {
                Ok(scanner::scan_world_sizes(&root))
            })?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                output::print_worlds(&summary, lang);
            }
        }
        Command::Usage => {
            let summary = with_spinner(scan_usage_msg(lang), !cli.json, || {
                Ok(scanner::scan_instance_usage(&root))
            })?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                output::print_usage(&summary, lang);
            }
        }
    }

    Ok(())
}

fn with_spinner<T, F>(msg: &str, enabled: bool, f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    if !enabled {
        return f();
    }

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .context("failed to initialize spinner style")?,
    );
    spinner.set_message(msg.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(120));

    let result = f();
    match &result {
        Ok(_) => spinner.finish_with_message(format!("{msg} done")),
        Err(_) => spinner.abandon_with_message(format!("{msg} failed")),
    }

    result
}

fn init_logging(level: cli::LogLevel, verbose: bool) -> Result<()> {
    let effective = if verbose && level < cli::LogLevel::Debug {
        cli::LogLevel::Debug
    } else {
        level
    };

    let mut builder = env_logger::Builder::from_env(Env::default().default_filter_or("info"));
    builder.filter_level(effective.as_filter());
    builder.format_timestamp_millis();
    builder.try_init().context("failed to initialize logger")?;
    Ok(())
}

fn scan_cleanup_msg(lang: Language) -> &'static str {
    match lang {
        Language::En => "Scanning cleanup targets...",
        Language::Ja => "クリーン対象をスキャン中...",
    }
}

fn scan_unused_libraries_msg(lang: Language) -> &'static str {
    match lang {
        Language::En => "Scanning unused libraries...",
        Language::Ja => "未使用librariesをスキャン中...",
    }
}

fn scan_unused_assets_msg(lang: Language) -> &'static str {
    match lang {
        Language::En => "Scanning unused assets...",
        Language::Ja => "未使用assetsをスキャン中...",
    }
}

fn clean_targets_msg(lang: Language) -> &'static str {
    match lang {
        Language::En => "Cleaning targets...",
        Language::Ja => "対象を削除中...",
    }
}

fn scan_duplicate_mods_msg(lang: Language) -> &'static str {
    match lang {
        Language::En => "Scanning duplicate mods...",
        Language::Ja => "重複modをスキャン中...",
    }
}

fn scan_worlds_msg(lang: Language) -> &'static str {
    match lang {
        Language::En => "Scanning worlds...",
        Language::Ja => "ワールドをスキャン中...",
    }
}

fn scan_usage_msg(lang: Language) -> &'static str {
    match lang {
        Language::En => "Scanning instance usage...",
        Language::Ja => "インスタンス使用量をスキャン中...",
    }
}

#[derive(Serialize)]
struct ScanJsonReport {
    cleanup: scanner::CleanupSummary,
    unused_libraries: scanner::UnusedLibrariesSummary,
    unused_assets: scanner::UnusedAssetsSummary,
}
