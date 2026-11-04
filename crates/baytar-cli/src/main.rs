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
}

fn main() -> anyhow::Result<()> {
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
