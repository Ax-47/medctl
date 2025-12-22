use clap::{Parser, Subcommand};
/// Medication CLI tool to log and view your medicine intake.
///
/// # Examples
/// ```bash
/// # Log a dose of Ritalin
/// medctl log --medicine Ritalin --dose 10 --note "after lunch"
///
/// # List last 5 logs
/// medctl list -l 5
///
/// # Show statistics for Ritalin
/// medctl stats --medicines Ritalin
/// ```
#[derive(Parser, Debug)]
#[command(name = "medctl")]
#[command(about = "Medication logger")]
pub struct Cli {
    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands for the medication logger CLI.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Log medication to Google Sheets
    ///
    /// Example:
    /// ```bash
    /// medctl log --medicine Ritalin --dose 10 --note "after lunch"
    /// ```
    Log {
        /// Medicine name
        #[arg(short, long, default_value = "Ritalin")]
        medicine: String,

        /// Dose in mg
        #[arg(short = 'd', long = "dose", default_value_t = 10)]
        dose_mg: u32,

        /// Optional note for this log
        #[arg(short, long, default_value = "")]
        note: String,
    },

    /// List recent logs from the spreadsheet
    ///
    /// Can filter by a specific date (`dd/mm/yyyy`) or leave empty for today.
    List {
        /// Number of rows to show
        #[arg(short, long, default_value_t = 10)]
        limit: usize,

        /// Filter logs by date (format: "dd/mm/yyyy") or "*" for all
        #[arg(short, long, default_value = "")]
        date: String,
    },

    /// Show statistics for medicines
    ///
    /// Can filter by one or more medicines (comma-separated). Optional monthly aggregation
    /// and graph display.
    Stats {
        /// Filter by medicine name(s), comma-separated
        #[arg(short, long, default_value = "Ritalin")]
        medicines: String,

        /// Show monthly aggregation
        #[arg(long, default_value_t = false)]
        month: bool,

        /// Show graph visualization
        #[arg(long, default_value_t = false)]
        graph: bool,
    },
}
