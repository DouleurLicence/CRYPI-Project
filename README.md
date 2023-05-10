# CRYPI-Project
Group project that made us work on Secure Predictions using an Secure Multiparty Computation Rust library.

## Secure Predictions

Secure prediction involves the use of advanced cryptographic techniques to ensure that sensitive data and trainded model remain private and secure while making predictions based on this trained model. This is accomplished using two main techniques: homomorphic encryption and secure multiparty computation. Homomorphic encryption allows encrypted data to be processed without revealing its content, while secure multiparty computation allows multiple parties to jointly compute a function without revealing their inputs. Secure prediction has important applications in fields such as healthcare, finance, and government, where sensitive data needs to be analyzed while protecting individual privacy.

# Subject

## Project 4: Secure Predictions using Secure Multiparty Computation

### Objective:

In this project, you will learn how to perform secure prediction using secure multiparty computation **in the semi-honest settings**.
You will use a secure multiparty computation library to predict the outcome of a new observation while preserving privacy and security.

### Materials:
- some explanations on logistic regression ([link](https://github.com/ConstanceBeguier/epita-projects-2023/tree/main/datasets/logistic_regression))
- a heart disease dataset ([framingham_heart_disease_test.csv](https://github.com/ConstanceBeguier/epita-projects-2023/blob/main/datasets/logistic_regression/framingham_heart_disease_test.csv))
- some explanations on the dataset content ([link](https://github.com/ConstanceBeguier/epita-projects-2023/tree/main/datasets))
- a trained model ([trained_model_coeffs.txt](https://github.com/ConstanceBeguier/epita-projects-2023/blob/main/datasets/logistic_regression/trained_model_coeffs.txt))

### Instructions:

1. Familiarize yourself with secure multiparty computation:
Before you begin, it is essential to understand the basics of secure multiparty computation.
Research and read about secure multiparty computation, its different types, and how it is used in secure prediction.
2. Select and set up the secure multiparty computation library:
Install and configure the secure multiparty computation library you will be using.
3. Predict the outcome:
Use the secure multiparty computation library to predict the outcome on a new observation.
4. Evaluate the model:
Evaluate the accuracy of the predicted outcomes by comparing them with the true outcomes.
Use the encrypted dataset to evaluate the model's accuracy.
Compare the accuracy obtained from the secure prediction model with secure multiparty computation with the accuracy obtained from the insecure prediction model without secure multiparty computation
5. Reflect:
Reflect on the process of performing secure prediction with secure multiparty computation.
Consider the advantages and disadvantages of using secure multiparty computation for prediction.
Think about potential use cases for secure prediction with secure multiparty computation.

### Participants:
- A semi-honest model owner, who possesses a trained model.
- A semi-honest data owner, who seeks a prediction for their data.

### Constraints/Challenges:
- Ensuring model confidentiality: the data owner must not gain any knowledge about the trained model.
- Ensuring data confidentiality: the model owner must not gain any knowledge about the data to predict.
- Ensuring prediction confidentiality: the model owner must not gain any knowledge about the prediction.


### Deliverables:

How to compile and run the code ? First, you'll need to install a few dependencies:

```bash
sudo apt install libssl-dev pkg-config cmake
```

Then, you can compile and run the code:

```bash
cargo build --release
```
To use the server, you'll need to run the following command:

```bash
./target/release/server <port>
```
To use the client, you'll need to run the following command:

```bash
./target/release/client <ip> <port>
```
