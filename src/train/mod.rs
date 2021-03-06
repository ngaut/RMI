// < begin copyright > 
// Copyright Ryan Marcus 2020
// 
// See root directory of this project for license terms.
// 
// < end copyright > 
 

use crate::models::*;

mod two_layer;
mod multi_layer;
mod lower_bound_correction;

pub struct TrainedRMI {
    pub model_avg_error: f64,
    pub model_avg_l2_error: f64,
    pub model_avg_log2_error: f64,
    pub model_max_error: u64,
    pub model_max_error_idx: usize,
    pub model_max_log2_error: f64,
    pub last_layer_max_l1s: Vec<u64>,
    pub rmi: Vec<Vec<Box<dyn Model>>>,
    pub models: String,
    pub branching_factor: u64
}

impl TrainedRMI {
    #[allow(dead_code)]
    fn test_predict(&self, lookup_key: u64) -> (u64, u64) {
        assert_eq!(self.rmi.len(), 2);
        let top_model = &self.rmi[0][0];
        let leaf_models = &self.rmi[1];
        let num_leaf_models = leaf_models.len() as u64;
        
        let leaf_idx = top_model.predict_to_int(lookup_key.into());
        let target = u64::min(num_leaf_models - 1, leaf_idx) as usize;
        println!("Target leaf: {}", target);
        let pred = leaf_models[target].predict_to_int(lookup_key.into());
        return (pred, self.last_layer_max_l1s[target]);
    }
}

fn train_model(model_type: &str, data: &ModelDataWrapper) -> Box<dyn Model> {
    let model: Box<dyn Model> = match model_type {
        "linear" => Box::new(LinearModel::new(data)),
        "robust_linear" => Box::new(RobustLinearModel::new(data)),
        "linear_spline" => Box::new(LinearSplineModel::new(data)),
        "cubic" => Box::new(CubicSplineModel::new(data)),
        "loglinear" => Box::new(LogLinearModel::new(data)),
        "normal" => Box::new(NormalModel::new(data)),
        "lognormal" => Box::new(LogNormalModel::new(data)),
        "radix" => Box::new(RadixModel::new(data)),
        "radix8" => Box::new(RadixTable::new(data, 8)),
        "radix18" => Box::new(RadixTable::new(data, 18)),
        "radix22" => Box::new(RadixTable::new(data, 22)),
        "radix26" => Box::new(RadixTable::new(data, 26)),
        "radix28" => Box::new(RadixTable::new(data, 28)),
        "bradix" => Box::new(BalancedRadixModel::new(data)),
        "histogram" => Box::new(EquidepthHistogramModel::new(data)),
        "plr" => Box::new(BottomUpPLR::new(data)),
        "pgm" => Box::new(PGM::new(data)),
        _ => panic!("Unknown model type: {}", model_type),
    };

    return model;
}

fn validate(model_spec: &[String]) {
    let num_layers = model_spec.len();
    let empty_data = ModelData::empty();
    let empty_container = ModelDataWrapper::new(&empty_data);

    for (idx, model) in model_spec.iter().enumerate() {
        let restriction = train_model(model, &empty_container).restriction();

        match restriction {
            ModelRestriction::None => {}
            ModelRestriction::MustBeTop => {
                assert_eq!(
                    idx, 0,
                    "if used, model type {} must be the root model",
                    model
                );
            }
            ModelRestriction::MustBeBottom => {
                assert_eq!(
                    idx,
                    num_layers - 1,
                    "if used, model type {} must be the bottommost model",
                    model
                );
            }
        }
    }
}

/*fn test_rmi_input(test_key: u64, data: &ModelDataWrapper, rmi: &TrainedRMI) {
    let correct = data.lower_bound(test_key);
    println!("Predicting {}", test_key);
    let (guess, err) = rmi.test_predict(test_key);
    println!("Model prediction for lookup {}: {} with error {}",
             test_key, guess, err);
    
    println!("({}, {}), {}",
             guess - err,
             guess + err,
             correct);
}*/

pub fn train(data: &mut ModelDataWrapper,
             model_spec: &str, branch_factor: u64) -> TrainedRMI {
    let (model_list, last_model): (Vec<String>, String) = {
        let mut all_models: Vec<String> = model_spec.split(',').map(String::from).collect();
        validate(&all_models);
        let last = all_models.pop().unwrap();
        (all_models, last)
    };

    if model_list.len() == 1 && data.len() > 1_000_000 {
        let res = two_layer::train_two_layer(data, &model_list[0],
                                             &last_model, branch_factor);
        return res;
    }

    // it is not a simple, two layer rmi
    return multi_layer::train_multi_layer(data, &model_list, last_model, branch_factor);
}
