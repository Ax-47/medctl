use chrono::Local;
use google_sheets4 as sheets4;
use sheets4::api::ValueRange;
pub async fn log(hub: Sheets<HttpsConnector<HttpConnector>>,medicine: &str, doze: &str, note: &str) {
    let now = Local::now();
    let date = now.format("%d/%m/%Y").to_string();
    let time = now.format("%H:%M:%S").to_string();

    let values = vec![vec![
        date.into(),
        time.into(),
        medicine.into(),
        doze.to_string().into(),
        "auto log".into(),
        note.into(),
    ]];

    let vr = ValueRange {
        values: Some(values),
        ..Default::default()
    };

    hub.spreadsheets()
        .values_append(vr, &spreadsheet_id, &sheet_name)
        .value_input_option("USER_ENTERED")
        .doit()
        .await?;
}
