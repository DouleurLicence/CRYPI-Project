use crate::csv_file::Record;
use ndarray::s;
use ndarray::{Array1, Array2, Axis};
use std::error::Error;

// Normalize data using min-max normalization
pub fn normalize_data(records: &[Record]) -> Result<Array2<f32>, Box<dyn Error>> {
    let data: Vec<Vec<f32>> = records
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

fn impute_nan_with_mean(column: &mut Array1<f64>) {
    let mean = column
        .iter()
        .filter_map(|&x| if x.is_nan() { None } else { Some(x) })
        .sum::<f64>()
        / (column.len() - column.iter().filter(|&&x| x.is_nan()).count()) as f64;

    column.mapv_inplace(|x| if x.is_nan() { mean } else { x });
}

pub fn clean_dataset(records: Vec<Record>) -> (Array2<f64>, Array1<f64>, Array2<f64>, Array1<f64>) {
    // Convert the records to an ndarray Array2<f64>
    let data = Array2::from_shape_fn((records.len(), 15), |(i, j)| match j {
        0 => records[i].male as f64,
        1 => records[i].age as f64,
        2 => records[i].currentSmoker as f64,
        3 => records[i].cigsPerDay,
        4 => records[i].BPMeds,
        5 => records[i].prevalentStroke as f64,
        6 => records[i].prevalentHyp as f64,
        7 => records[i].diabetes as f64,
        8 => records[i].totChol,
        9 => records[i].sysBP,
        10 => records[i].diaBP,
        11 => records[i].BMI,
        12 => records[i].heartRate,
        13 => records[i].glucose,
        14 => records[i].TenYearCHD as f64,
        _ => unreachable!(),
    });

    // Split the dataset into X and y
    let mut X = data.slice(s![.., ..-1]).to_owned();
    let y = data.column(data.ncols() - 1).to_owned();

    // Impute missing values with mean
    for i in 0..X.ncols() {
        let mut column = X.column_mut(i).to_owned();
        impute_nan_with_mean(&mut column);
        X.column_mut(i).assign(&column);
    }

    // Split the dataset into train and test sets
    let n = X.nrows();
    let train_indices: Vec<usize> = (0..n).filter(|i| i % 5 != 0).collect();
    let test_indices: Vec<usize> = (0..n).filter(|i| i % 5 == 0).collect();

    let X_train = X.select(Axis(0), &train_indices);
    let y_train = y.select(Axis(0), &train_indices);
    let X_test = X.select(Axis(0), &test_indices);
    let y_test = y.select(Axis(0), &test_indices);

    return (X_train, y_train, X_test, y_test);
}
