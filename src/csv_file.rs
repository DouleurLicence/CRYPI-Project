use ndarray::Array1;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;

use csv::ReaderBuilder;
use csv::WriterBuilder;

#[derive(Serialize, Deserialize, Debug, Clone)]
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

pub fn write_csv_file(records: Vec<Record>, path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = WriterBuilder::new().from_writer(file);

    // Write the records
    for record in records {
        writer.serialize(&record)?;
    }

    // Flush the writer to ensure all data is written to the file
    writer.flush()?;

    Ok(())
}

pub fn read_file_to_array1(filename: &str) -> Result<Array1<f64>, Box<dyn std::error::Error>> {
    let path = Path::new(filename);
    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let mut data = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let value: f64 = line.trim().parse()?;
        data.push(value);
    }

    Ok(Array1::from(data))
}
