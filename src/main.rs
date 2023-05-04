mod csv_file;
mod normalize;

use csv_file::read_csv_file;
use normalize::normalize_data;

fn main() {
    let records = read_csv_file(String::from(
        "given_files/framingham_heart_disease_test.csv",
    ))
    .unwrap();
    let normalized_data = normalize_data(&records).unwrap();

    println!("{:?}", normalized_data);
}
