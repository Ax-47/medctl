use chrono::{Datelike, Local, NaiveDate, NaiveTime, Timelike};
use comfy_table::{ContentArrangement, Table, presets::UTF8_FULL};
use google_sheets4::{
    Sheets, api::ValueRange, hyper_rustls::HttpsConnector,
    hyper_util::client::legacy::connect::HttpConnector,
};

pub struct MedicationLogger {
    hub: Sheets<HttpsConnector<HttpConnector>>,
    spreadsheet_id: String,
    sheet_name: String,
}

impl MedicationLogger {
    pub fn new(
        hub: Sheets<HttpsConnector<HttpConnector>>,
        spreadsheet_id: String,
        sheet_name: String,
    ) -> Self {
        Self {
            hub,
            spreadsheet_id,
            sheet_name,
        }
    }

    pub async fn log(
        &self,
        medicine: &str,
        dose: u32,
        note: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = Local::now();
        let date = now.format("%d/%m/%Y").to_string();
        let time = now.format("%H:%M:%S").to_string();

        // normalize medicine name: ritalin -> Ritalin
        let medicine = {
            let mut c = medicine.chars();
            match c.next() {
                None => "Unknown".to_string(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        };

        let values = vec![vec![
            date.into(),
            time.clone().into(),
            medicine.clone().into(),
            (dose as f64).into(),
            "auto log".into(),
            note.into(),
        ]];

        let vr = ValueRange {
            values: Some(values),
            ..Default::default()
        };

        self.hub
            .spreadsheets()
            .values_append(vr, &self.spreadsheet_id, &self.sheet_name)
            .value_input_option("USER_ENTERED")
            .doit()
            .await?;

        if note.is_empty() {
            println!("✓ Logged {} — {} mg at {}", medicine, dose, time);
        } else {
            println!("✓ Logged {} — {} mg ({}) at {}", medicine, dose, note, time);
        }
        Ok(())
    }

    pub async fn list(&self, datef: &str) -> Result<(), Box<dyn std::error::Error>> {
        let resp = self
            .hub
            .spreadsheets()
            .values_get(&self.spreadsheet_id, &self.sheet_name)
            .doit()
            .await?;

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Date", "Time", "Medicine", "Dose (mg)", "Note"]);

        let now = Local::now();
        if let Some(values) = resp.1.values {
            for row in values.iter().skip(1) {
                let date_str = row.first().and_then(|v| v.as_str()).unwrap_or("");
                let time = row.get(1).and_then(|v| v.as_str()).unwrap_or("");
                let med = row.get(2).and_then(|v| v.as_str()).unwrap_or("");
                let dose = row.get(3).and_then(|v| v.as_str()).unwrap_or("");
                let note = row.get(5).and_then(|v| v.as_str()).unwrap_or("");
                if datef == "*" {
                    table.add_row(vec![date_str, time, med, dose, note]);
                } else if datef.is_empty()
                    && let Ok(date) = NaiveDate::parse_from_str(date_str, "%d/%m/%Y")
                    && date.day() == now.day()
                {
                    table.add_row(vec![date_str, time, med, dose, note]);
                } else if let (Ok(date), Ok(dateff)) = (
                    NaiveDate::parse_from_str(date_str, "%d/%m/%Y"),
                    NaiveDate::parse_from_str(datef, "%d/%m/%Y"),
                ) && date.day() == dateff.day()
                {
                    table.add_row(vec![date_str, time, med, dose, note]);
                }
            }
        }

        println!("{table}");
        Ok(())
    }

    pub async fn stats(&self) -> Result<(), Box<dyn std::error::Error>> {
        let resp = self
            .hub
            .spreadsheets()
            .values_get(&self.spreadsheet_id, &self.sheet_name)
            .doit()
            .await?;

        use std::collections::HashMap;
        let mut count: HashMap<String, u32> = HashMap::new();
        let mut total: HashMap<String, f64> = HashMap::new();
        let mut times_map: HashMap<String, Vec<f64>> = HashMap::new(); // เก็บชั่วโมง

        if let Some(values) = resp.1.values {
            for row in values.iter().skip(1) {
                let med_raw = row.get(2).and_then(|v| v.as_str()).unwrap_or("unknown");

                let med = {
                    let mut c = med_raw.chars();
                    match c.next() {
                        None => "Unknown".to_string(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                };

                let dose: f64 = row
                    .get(3)
                    .and_then(|v| v.as_str())
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);

                // parse เวลา
                let hour: f64 = row
                    .get(1)
                    .and_then(|v| v.as_str())
                    .and_then(|s| NaiveTime::parse_from_str(s, "%H:%M:%S").ok())
                    .map(|t| t.num_seconds_from_midnight() as f64 / 3600.0) // แปลงเป็นชั่วโมง
                    .unwrap_or(0.0);

                *count.entry(med.clone()).or_insert(0) += 1;
                *total.entry(med.clone()).or_insert(0.0) += dose;
                times_map.entry(med.clone()).or_default().push(hour);
            }
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                "Medicine",
                "Times",
                "Total (mg)",
                "Avg (mg)",
                "Time Diff (h)",
            ]);

        for (med, times) in &count {
            let sum = total.get(med).unwrap_or(&0.0);
            let avg = if *times > 0 { sum / *times as f64 } else { 0.0 };

            // เวลาเฉลี่ย
            let hours: &[f64] = times_map.get(med).map(|v| &v[..]).unwrap_or(&[]);

            // ระยะเวลาระหว่างการกินยาเฉลี่ย
            let mut sorted_hours = hours.to_vec();
            sorted_hours.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let time_diff = if sorted_hours.len() > 1 {
                let diffs: Vec<f64> = sorted_hours.windows(2).map(|w| w[1] - w[0]).collect();
                diffs.iter().sum::<f64>() / diffs.len() as f64
            } else {
                0.0
            };

            table.add_row(vec![
                med,
                &times.to_string(),
                &format!("{:.2}", sum),
                &format!("{:.2}", avg),
                &format!("{:.2}", time_diff),
            ]);
        }

        println!("Medication statistics");
        println!("{table}");

        Ok(())
    }
    pub async fn stats_month_graph(&self) -> Result<(), Box<dyn std::error::Error>> {
        use chrono::{Datelike, Local, NaiveDate, NaiveTime};
        use crossterm::{
            event::{self, Event, KeyCode},
            execute,
            terminal::{disable_raw_mode, enable_raw_mode},
        };
        use ratatui::{
            prelude::*,
            widgets::{Axis, Block, Borders, Chart, Dataset},
        };
        use std::collections::HashMap;
        use std::io;
        let resp = self
            .hub
            .spreadsheets()
            .values_get(&self.spreadsheet_id, &self.sheet_name)
            .doit()
            .await?;

        // group per day
        let mut times_per_day: HashMap<NaiveDate, Vec<f64>> = HashMap::new();
        let now = Local::now();
        let year = now.year();
        let month = now.month();
        if let Some(values) = resp.1.values {
            for row in values.iter().skip(1) {
                let date_str = row.first().and_then(|v| v.as_str());
                let time_str = row.get(1).and_then(|v| v.as_str());
                if let (Some(date_str), Some(time_str)) = (date_str, time_str)
                    && let Ok(date) = NaiveDate::parse_from_str(date_str, "%d/%m/%Y")
                    && let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M:%S")
                    && date.year() == year
                    && date.month() == month
                {
                    times_per_day
                        .entry(date)
                        .or_default()
                        .push(time.num_seconds_from_midnight() as f64 / 60f64);
                }
            }
        }
        // หา max number of pills in a day
        let max_pills = times_per_day.values().map(|v| v.len()).max().unwrap_or(0);
        let mut pill_series: Vec<Vec<(f64, f64)>> = vec![vec![]; max_pills];

        // แยกตามเม็ด
        for (date, times) in times_per_day {
            let day = date.day() as f64;
            for (i, t) in times.iter().enumerate() {
                if i < pill_series.len() {
                    pill_series[i].push((day, *t));
                }
            }
        }
        let colors = [
            Color::Green,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::Yellow,
            Color::Red,
        ];

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| {
                let size = f.area();

                let datasets: Vec<_> = pill_series
                    .iter()
                    .enumerate()
                    .map(|(i, data)| {
                        Dataset::default()
                            .name(format!("Pill {}", i + 1))
                            .marker(symbols::Marker::Dot)
                            .style(Style::default().fg(colors[i % colors.len()]))
                            .data(data)
                    })
                    .collect();

                let chart = Chart::new(datasets)
                    .block(
                        Block::default()
                            .title("⏱ Medication timing per day")
                            .borders(Borders::ALL),
                    )
                    .x_axis(
                        Axis::default()
                            .title("Day")
                            .bounds([1.0, 31.0])
                            .labels((1..=31).map(|a| a.to_string())),
                    )
                    .y_axis(
                        Axis::default().title("Time").bounds([0.0, 1440.0]).labels(
                            (0..=24)
                                .map(|h| {
                                    let hour = if h == 0 || h == 24 {
                                        12
                                    } else if h > 12 {
                                        h - 12
                                    } else {
                                        h
                                    };
                                    let suffix = if h < 12 || h == 24 { "AM" } else { "PM" };
                                    Span::styled(
                                        format!("{hour} {suffix}"),
                                        Style::default().add_modifier(Modifier::BOLD),
                                    )
                                })
                                .collect::<Vec<Span>>(),
                        ),
                    );

                f.render_widget(chart, size);
            })?;

            if let Event::Key(key) = event::read()?
                && key.code == KeyCode::Char('q')
            {
                break;
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;

        Ok(())
    }
}
