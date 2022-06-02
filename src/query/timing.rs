pub struct Timing {
    /// timing in seconds
    pub data: Vec<f64>
}

pub fn get_timing_data(obj: &Timing) -> (usize, usize) {
    let mut min = 100000.;
    let mut max = 0.;
    let mut min_index = 0;
    let mut max_index = 0;

    for (j, &value) in obj.data.iter().enumerate() {
        if value > max {
            max = value;
            max_index = j + 1;
        }
        
        if value < min {
            min = value;
            min_index = j + 1;
        }
    }

    println!("min: {}; max: {}; avg: {}", min, max, obj.data.iter().sum::<f64>() as f64 / obj.data.len() as f64);
    (min_index, max_index)
}