use ndarray::{Array2, Axis};
use std::error::Error;

use crate::csv_file::Record;

// Normalize data using min-max normalization
pub fn normalize_data(records: &[Record]) -> Result<Array2<f32>, Box<dyn Error>> {
    let mut data: Vec<Vec<f32>> = records
        .iter()
        .map(|record| {
            vec![
                record.male as f32,
                record.age as f32,
                record.currentSmoker as f32,
                record.cigsPerDay as f32,
                record.BPMeds as f32,
                record.prevalentStroke as f32,
                record.prevalentHyp as f32,
                record.diabetes as f32,
                record.totChol as f32,
                record.sysBP as f32,
                record.diaBP as f32,
                record.BMI as f32,
                record.heartRate as f32,
                record.glucose as f32,
                record.TenYearCHD as f32,
            ]
        })
        .collect();

    let mut array_data =
        Array2::from_shape_vec((records.len(), 15), data.into_iter().flatten().collect())?;

    for feature_idx in 0..15 {
        let column = array_data.column(feature_idx);
        let min = column.fold(column[0], |min, &val| min.min(val));
        let max = column.fold(column[0], |max, &val| max.max(val));

        if (max - min) > 0.0 {
            let _ = array_data
                .column_mut(feature_idx)
                .mapv_inplace(|value| (value - min) / (max - min));
        }
    }

    Ok(array_data)
}
