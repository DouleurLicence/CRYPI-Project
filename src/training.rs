use ndarray::{Array, Array1, Array2};

pub fn sigmoid(z: &Array1<f64>) -> Array1<f64> {
    let one: f64 = 1.0;
    one / (one + (-z).mapv(f64::exp))
}

pub fn logistic_regression(
    X: &Array2<f64>,
    y: &Array1<f64>,
    alpha: f64,
    iterations: usize,
) -> Array1<f64> {
    let m = X.nrows() as f64;
    let mut theta = Array::zeros(X.ncols());

    for _ in 0..iterations {
        let h = sigmoid(&(X.dot(&theta)));
        let gradient = (X.t().dot(&(h - y))) / m;
        theta -= &(gradient * alpha);
    }

    theta
}

pub fn train_log_reg(X_train: &Array2<f64>, y_train: &Array1<f64>) -> Array1<f64> {
    // Implement training a logistic regression model
    let alpha = 0.01; // Learning rate
    let iterations = 1000; // Number of iterations for gradient descent
    let model = logistic_regression(&X_train, &y_train, alpha, iterations);
    model
}

pub fn predict(model: &Array1<f64>, X: &Array2<f64>) -> Array1<f64> {
    let preds = sigmoid(&(X.dot(&model.t())));
    preds.mapv(|p| if p >= 0.5 { 1.0 } else { 0.0 })
}

pub fn model_accuracy(model: &Array1<f64>, X_test: &Array2<f64>, y_test: &Array1<f64>) -> f64 {
    let y_pred = predict(model, X_test);
    let correct_preds = (y_pred - y_test).mapv(|x| (x == 0.0) as u32).sum();
    correct_preds as f64 / y_test.len() as f64
}
