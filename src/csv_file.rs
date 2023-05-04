use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::Read;

use csv::ReaderBuilder;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub male: u32,
    pub age: u32,
    pub currentSmoker: u32,
    pub cigsPerDay: f64,
    pub BPMeds: f64,
    pub prevalentStroke: u32,
    pub prevalentHyp: u32,
    pub diabetes: u32,
    pub totChol: f64,
    pub sysBP: f64,
    pub diaBP: f64,
    pub BMI: f64,
    pub heartRate: f64,
    pub glucose: f64,
    pub TenYearCHD: u32,
}

pub fn read_csv_file(path: String) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut reader = ReaderBuilder::new().from_reader(contents.as_bytes());

    // Put everything into an array of Records
    let mut records: Vec<Record> = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    return Ok(records);
}
