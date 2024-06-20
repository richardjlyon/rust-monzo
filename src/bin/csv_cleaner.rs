//! A script for cleaning up CSV data from Monzo bank statement produced by scanning a PDF.
//!
use chrono::NaiveDate;
use regex::Regex;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::num::ParseFloatError;

#[derive(Debug)]
struct Transaction {
    date: NaiveDate,
    description: String,
    amount: f64,
}

#[derive(Debug, Serialize)]
struct TransactionForCsv {
    date: NaiveDate,
    description: String,
    amount: f64,
    local_currency: Option<String>,
    local_amount: Option<f64>,
    category: Option<String>,
}

#[derive(Debug)]
struct LocalCurrency {
    currency: String,
    amount: f64,
    description: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_names = vec![
        "monzo-discretionary",
        "monzo-essential-fixed",
        "monzo-essential-variable",
        "monzo-savings",
    ];

    for file_name in file_names {
        let file_path = format!("src/bin/csv_data/{file_name}.csv");
        let csv_path = format!("src/bin/csv_data/processed/{file_name}-processed.csv");
        let error_path = format!("src/bin/csv_data/processed/{file_name}-error.txt");

        println!("Processing file: {file_path}...");

        let joined_lines = join_lines(&file_path)?;
        let records = split_string_by_date(&joined_lines);
        let (transactions_for_csv, failures) = parse_records(records);

        println!(
            "  ->> Got {} transactions for csv",
            transactions_for_csv.len()
        );
        println!("  ->> Failed to parse {} transactions", failures.len());

        let csv_file = File::create(csv_path)?;
        let mut wtr = csv::Writer::from_writer(csv_file);

        for tx in transactions_for_csv {
            wtr.serialize(tx)?;
        }
        wtr.flush()?;

        if failures.len() > 0 {
            let mut error_file = File::create(error_path)?;
            for failure in failures {
                writeln!(error_file, "{}", failure)?;
            }
        }
    }

    Ok(())
}

impl Transaction {
    fn parse_local_currency(&self) -> Option<LocalCurrency> {
        let re = Regex::new(r"Amount : (\w{3}) (-?\d+\.\d{2})").unwrap();

        match re.captures(&self.description) {
            Some(cap) => {
                let currency = cap.get(1).unwrap().as_str().to_string();
                let amount = cap.get(2).unwrap().as_str().parse::<f64>().unwrap();

                let description = if let Some(keyword_index) = &self.description.find("Amount :") {
                    // Extract the substring from the start to the keyword index
                    let extracted_substring = &self.description[..*keyword_index];
                    Some(extracted_substring.trim().to_string()) // Trim any leading/trailing whitespace
                } else {
                    None
                };

                Some(LocalCurrency {
                    currency,
                    amount,
                    description,
                })
            }
            None => None,
        }
    }
}

// Creates one large string from the lines of a file
fn join_lines(file_path: &str) -> Result<String, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut strings: Vec<String> = vec![];

    for line in reader.lines() {
        let line = line?;
        strings.push(line);
    }

    let result = strings.join(" ");

    Ok(result)
}

// Splits a string in to lines starting with a date
fn split_string_by_date(input: &str) -> Vec<String> {
    let re = Regex::new(r"(\d{2}/\d{2}/\d{4})").unwrap();
    let mut results = Vec::new();
    let mut last_index = 0;

    for cap in re.captures_iter(input) {
        let date_index = cap.get(0).unwrap().start();

        if date_index > last_index {
            results.push(input[last_index..date_index].trim().to_string());
        }

        last_index = date_index;
    }

    if last_index < input.len() {
        results.push(input[last_index..].trim().to_string());
    }

    results.into_iter().filter(|s| !s.is_empty()).collect()
}

// Parse a list of records into transactions
fn parse_records(records: Vec<String>) -> (Vec<TransactionForCsv>, Vec<String>) {
    let mut transactions: Vec<Transaction> = Vec::new();
    let mut transactions_for_csv: Vec<TransactionForCsv> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for record in records {
        let cleaned_string = clean_string(&record);
        match parse_string(&cleaned_string) {
            Ok(t) => transactions.push(t),
            Err(_) => failures.push(record),
        }
    }

    for transaction in transactions {
        let tx_for_csv = convert_to_csv_format(transaction);
        transactions_for_csv.push(tx_for_csv);
    }

    (transactions_for_csv, failures)
}

// remove quotes from a string
fn clean_string(line: &str) -> String {
    line.replace("\"", "")
}

// parse a string into Transaction
fn parse_string(string: &str) -> Result<Transaction, ParseFloatError> {
    let format = "%d/%m/%Y";
    let mut parts = string.split(',');
    let date_str = parts.next().unwrap().to_string();
    let date = NaiveDate::parse_from_str(&date_str, format).unwrap();
    let description = parts.next().unwrap().to_string();
    let amount = match parts.next().unwrap().parse::<f64>() {
        Ok(amount) => amount,
        Err(e) => {
            return Err(e);
        }
    };

    Ok(Transaction {
        date,
        description,
        amount,
    })
}

fn convert_to_csv_format(transaction: Transaction) -> TransactionForCsv {
    let local_currency = transaction.parse_local_currency();
    let (local_currency, local_amount, description) = match local_currency {
        Some(local_currency) => (
            Some(local_currency.currency),
            Some(local_currency.amount),
            local_currency.description.unwrap(),
        ),
        None => (None, None, transaction.description.clone()),
    };

    TransactionForCsv {
        date: transaction.date,
        description,
        amount: transaction.amount,
        local_currency,
        local_amount,
        category: None,
    }
}

// -- Tests -------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_string_by_date() {
        let input = "01/01/2020,Description 1,100.00 02/01/2020,Description 2,200.00";
        let result = split_string_by_date(input);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_clean_string() {
        let input = "\"01/01/2020,Description 1,100.00\"";
        let expected = "01/01/2020,Description 1,100.00";
        let result = clean_string(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_string() {
        let input = "01/01/2020,Description 1,100.00".to_string();
        let result = parse_string(&input).unwrap();
        let expected_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        assert_eq!(result.date, expected_date);
        assert_eq!(result.description, "Description 1".to_string());
        assert_eq!(result.amount, 100.0);
    }

    #[test]
    fn parse_convert_to_csv_format() {
        let tx = Transaction {
            date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            description:
                "Navigraph Stockholm SWE Amount : EUR -9.05 . Conversion rate : 1.169251 ."
                    .to_string(),
            amount: -7.74,
        };

        let result = convert_to_csv_format(tx);
        assert_eq!(result.date, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert_eq!(result.description, "Navigraph Stockholm SWE".to_string());
        assert_eq!(result.amount, -7.74);
        assert_eq!(result.local_currency, Some("EUR".to_string()));
        assert_eq!(result.local_amount, Some(-9.05));
    }

    #[test]
    fn test_parse_local_currency_ok() {
        let tx = Transaction {
            date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            description:
                "Navigraph Stockholm SWE Amount : EUR -9.05 . Conversion rate : 1.169251 ."
                    .to_string(),
            amount: -7.74,
        };

        let local_currency = tx.parse_local_currency().unwrap();

        assert_eq!(local_currency.currency, "EUR".to_string());
        assert_eq!(local_currency.amount, -9.05);
        assert_eq!(
            local_currency.description,
            Some("Navigraph Stockholm SWE".to_string())
        );
    }

    #[test]
    fn test_parse_local_currency_is_none() {
        let tx = Transaction {
            date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            description: "AMAZON UK RETAIL WWW.AMAZON.CO LUX".to_string(),
            amount: -34.37,
        };

        let local_currency = tx.parse_local_currency();

        assert!(local_currency.is_none());
    }
}
