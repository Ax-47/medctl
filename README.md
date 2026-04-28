# medctl

`medctl` is a CLI tool to log and track your medication. It stores data in **Google Sheets**, allows viewing history, statistics, and visualizes your medication intake with graphs.

---

## Installation

  ```bash
  # Install Rust (if not installed)  
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

  # Clone the project

  git clone https://github.com/yourusername/medctl.git
  cd medctl

  # Build the project

  cargo build --release

  # Set environment variables

  export SPREADSHEET_ID="your_google_sheet_id"
  export SHEET_NAME="Sheet1!A:F"

  # Prepare your service-account.json for Google Sheets API access
  ```
## Usage

  ```fish
  medctl <COMMAND> [OPTIONS]
  ```
## Commands

Log

Log a medication entry:

  ```fish
  medctl log --medicine Ritalin --dose 10 --note "after lunch"

  ```
Options:
Flag	Description	Default
-m, --medicine	Medicine name	Ritalin
-d, --dose	Dose in mg	10
-n, --note	Additional note	""
List

Show recent logs:

medctl list --limit 5

You can filter by date:

medctl list --date "22/12/2025"
medctl list --date "*"  # Show all dates

Options:
Flag	Description	Default
-l, --limit	Number of rows to display	10
-d, --date	Date (dd/mm/yyyy) or * for all	""
Stats

View medication statistics and optional graph:

medctl stats --medicines Ritalin --month --graph

Options:
Flag	Description	Default
-m, --medicines	Filter by medicine(s), comma-separated	Ritalin
--month	Show monthly aggregation	false
--graph	Display a graph visualization	false
Example Output
Log

Logged: Ritalin 10 mg (after lunch)

List

┌────────────┬──────────┬──────────┬───────────┬─────────────────┐
│ Date       │ Time     │ Medicine │ Dose (mg) │ Note            │
├────────────┼──────────┼──────────┼───────────┼─────────────────┤
│ 22/12/2025 │ 12:32:00 │ Ritalin │ 10        │ after lunch     │
...

Stats + Graph

Medication statistics
┌──────────┬───────┬──────────────┬──────────┐
│ Medicine │ Times │ Total dose   │ Avg      │
├──────────┼───────┼──────────────┼──────────┤
│ Ritalin  │ 6     │ 60.00 mg     │ 10.00    │
└──────────┴───────┴──────────────┴──────────┘

Graph: (each dot represents the time of intake)

License

MIT License © 2025
