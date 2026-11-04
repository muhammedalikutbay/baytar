use std::path::{Path, PathBuf};

use baytar_core::backup::{self, BackupKind, BackupPolicy};
use baytar_core::db::Database;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "baytar-cli", about = "Baytar development tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the Baytar core version.
    Version,
    /// Print aggregate statistics for a legacy PatientsCare database.
    Inspect {
        path: std::path::PathBuf,
        /// Print the report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Import a legacy PatientsCare database into a Baytar database.
    ///
    /// The legacy file is only read. Output is aggregate counts and categories; names,
    /// phones, notes and raw values appear only with --show-values.
    Import {
        /// The legacy PatientsCare .db file.
        legacy: PathBuf,
        /// The Baytar database to import into (created if it does not exist).
        #[arg(long)]
        db: PathBuf,
        /// Only preview: write nothing.
        #[arg(long)]
        dry_run: bool,
        /// Print the result as JSON.
        #[arg(long)]
        json: bool,
        /// Include raw legacy values (names, phones, notes) in the output.
        #[arg(long)]
        show_values: bool,
    },
    /// Create, inspect, restore, or prune local database backups.
    Backup {
        #[command(subcommand)]
        command: BackupCommand,
    },
}

#[derive(Subcommand)]
enum BackupCommand {
    /// Create a verified manual backup.
    Create {
        #[arg(long)]
        db: PathBuf,
    },
    /// List recognized backups, newest first.
    List {
        #[arg(long)]
        db: PathBuf,
    },
    /// Restore a backup after preserving the current database.
    Restore {
        #[arg(long)]
        db: PathBuf,
        file: PathBuf,
    },
    /// Prune daily backups using the default retention policy.
    Prune {
        #[arg(long)]
        db: PathBuf,
    },
}

fn backup_dir(db: &Path) -> PathBuf {
    backup::backups_dir(
        db.parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new(".")),
    )
}

fn run_import(
    legacy: &Path,
    db: &Path,
    dry_run: bool,
    json: bool,
    show_values: bool,
) -> anyhow::Result<()> {
    use baytar_core::legacy::import;

    // A mistyped source path must not create an empty database.
    anyhow::ensure!(
        legacy.is_file(),
        "legacy file does not exist: {}",
        legacy.display()
    );
    let work_dir = std::env::temp_dir();
    let prepared = import::prepare(legacy, &work_dir)?;
    let dir = backup_dir(db);

    if dry_run {
        // A dry run never creates the Baytar database: a missing one previews as empty.
        let database = if db.is_file() {
            Database::open(db, &dir)?
        } else {
            Database::open_in_memory()?
        };
        let preview = import::preview(&prepared, &database)?;
        if json {
            let shown = if show_values {
                preview.clone()
            } else {
                preview.without_values()
            };
            println!("{}", serde_json::to_string_pretty(&shown)?);
        } else {
            println!("Dry run: nothing was written.");
            print!("{}", import::format_preview(&preview, show_values));
        }
        return Ok(());
    }

    let mut database = Database::open(db, &dir)?;
    let report = import::execute(&prepared, &mut database, &dir)?;
    if json {
        let shown = if show_values {
            report.clone()
        } else {
            report.without_values()
        };
        println!("{}", serde_json::to_string_pretty(&shown)?);
    } else {
        print!("{}", import::format_report(&report, show_values));
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    match Cli::parse().command {
        Commands::Version => println!("{}", baytar_core::version()),
        Commands::Inspect { path, json } => {
            let report = baytar_core::legacy::inspect::inspect(&path)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_report(&report);
            }
        }
        Commands::Import {
            legacy,
            db,
            dry_run,
            json,
            show_values,
        } => run_import(&legacy, &db, dry_run, json, show_values)?,
        Commands::Backup { command } => match command {
            BackupCommand::Create { db } => {
                // A mistyped source path must not create an empty database.
                anyhow::ensure!(db.is_file(), "database does not exist: {}", db.display());
                let dir = backup_dir(&db);
                let database = Database::open(&db, &dir)?;
                let info = backup::create(&database, &dir, BackupKind::Manual)?;
                println!("{}", info.path.display());
            }
            BackupCommand::List { db } => {
                for info in backup::list(&backup_dir(&db))? {
                    println!(
                        "{}\t{}\t{} bytes\t{}",
                        info.created_at,
                        info.kind.as_str(),
                        info.size_bytes,
                        info.path.display()
                    );
                }
            }
            BackupCommand::Restore { db, file } => {
                let previous = backup::restore(&db, &file, &backup_dir(&db))?;
                println!(
                    "Restored {}. Pre-restore backup: {}",
                    db.display(),
                    previous.path.display()
                );
            }
            BackupCommand::Prune { db } => {
                for path in backup::prune(
                    &backup_dir(&db),
                    &BackupPolicy::default(),
                    chrono::Utc::now(),
                )? {
                    println!("Removed {}", path.display());
                }
            }
        },
    }
    Ok(())
}

fn print_report(report: &baytar_core::legacy::inspect::InspectReport) {
    println!("Legacy database inspection");
    println!("SQLite user_version: {}", report.user_version);
    println!("Page count: {}", report.page_count);
    println!("\nTables");
    println!("{:<24} {:>10}  Columns", "Name", "Rows");
    for table in &report.tables {
        let columns = table
            .columns
            .iter()
            .map(|column| format!("{} ({})", column.name, column.declared_type))
            .collect::<Vec<_>>()
            .join(", ");
        println!("{:<24} {:>10}  {}", table.name, table.row_count, columns);
    }
    println!("\nKnown table checks");
    for (name, details) in &report.known_tables {
        if !details.present {
            println!("{name}: missing");
            continue;
        }
        println!("{name}:");
        if !details.missing_columns.is_empty() {
            println!("  Missing columns: {}", details.missing_columns.join(", "));
        }
        if let Some(phone) = &details.phone {
            println!("  OwnerPhone storage classes: {:?}", phone.storage_classes);
            println!(
                "  OwnerPhone digit-length buckets: {:?}",
                phone.digit_length_buckets
            );
            println!(
                "  Ten digits starting with 5: {}",
                phone.ten_digit_starting_with_five
            );
            println!(
                "  Values containing non-digits: {}",
                phone.containing_non_digits
            );
        }
        for (column, counts) in &details.dates {
            println!("  {column} formats: {counts:?}");
        }
        for (column, counts) in &details.combo_values {
            println!("  {column} known values and other count: {counts:?}");
        }
        if let Some(count) = details.orphan_patients {
            println!("  Orphan patients: {count}");
        }
        if let Some(count) = details.orphan_todos {
            println!("  Orphan to-dos: {count}");
        }
        if let Some(count) = details.duplicate_looking_owners {
            println!("  Owners with duplicate-looking normalized phones: {count}");
        }
    }
}
