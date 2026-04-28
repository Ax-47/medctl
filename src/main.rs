use crate::arg::Commands;
use crate::commands::MedicationLogger;
use clap::Parser;
use google_sheets4 as sheets4;
use sheets4::Sheets;
use sheets4::{hyper_rustls, hyper_util, yup_oauth2};
mod arg;
mod commands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let spreadsheet_id = dotenv::var("SPREADSHEET_ID")?;
    let sheet_name = dotenv::var("SHEET_NAME")?;
    let secret = yup_oauth2::read_service_account_key("service-account.json").await?;
    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(secret)
        .build()
        .await
        .unwrap();
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http1()
                .build(),
        );
    let hub = Sheets::new(client, auth);
    let cli = arg::Cli::parse();
    let logger = MedicationLogger::new(hub, spreadsheet_id, sheet_name);

    match cli.command {
        Commands::Log {
            medicine,
            dose_mg,
            note,
        } => {
            logger.log(&medicine, dose_mg, &note).await?;
        }
        Commands::List { limit: _, date } => {
            logger.list(&date).await?;
        }
        Commands::Stats {
            medicines: _,
            graph,
            month: _,
        } => {
            if graph {
                logger.stats_month_graph().await?;
            } else {
                logger.stats().await?;
            }
        }
    }

    Ok(())
}
