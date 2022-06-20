use log::{info};
use std::time::{Instant};
mod write_csv;

#[derive(Default)]
pub struct Timing {
    /// timing in seconds
    pub data: Vec<f64>,
    // 
    pub now: Option<Instant>,
    //
    pub avg: u128,
    //
    pub cnt: u64,
    //
    pub writer: Option<csv::Writer<std::fs::File>>,
    //
    pub node_end_point: String,
}

impl Timing {
    pub fn init_write(&mut self, stage: u64) -> Result<(), csv::Error>{
        self.writer = Some(write_csv::create(self.node_end_point.to_string(), stage)?);
        self.data = Vec::new();

        Ok(())
    }

    pub fn init(&mut self) {
        self.avg = 0;
        self.cnt = 0;
    }

    pub fn start(&mut self) {
        self.now = Some(Instant::now());
    }

    pub fn iteration(&mut self) {
        self.avg += self.now.unwrap().elapsed().as_millis();
        self.cnt += 1;
    }

    pub fn end(&mut self, batch_size: usize, concurrency: usize) -> Result<f64, csv::Error> {
        let fin_avg = (self.avg as f64) / (self.cnt as f64);
        self.data.push(fin_avg);

        if let Some(w) = self.writer.as_mut() {
            w.write_record(&[format!("{batch_size}"),
                             format!("{concurrency}"),
                             format!("{fin_avg}")])?;
        }

        Ok(fin_avg)
    }
}

pub fn get_timing_data(mut timer: Timing) {
    let mut min = 100000.;
    let mut max = 0.;
    let mut min_index = 0;
    let mut max_index = 0;

    for (j, &value) in timer.data.iter().enumerate() {
        if value > max {
            max = value;
            max_index = j + 1;
        }
        
        if value < min {
            min = value;
            min_index = j + 1;
        }
    }

    info!("min: {}; max: {}; avg: {}", min, max, timer.data.iter().sum::<f64>() as f64 / timer.data.len() as f64);
    let num = timer.data.len();

    let batch_min = num - min_index + 1;
    let remainder_0 = if num % batch_min == 0 {0} else {1};
    let concurrency_min = num / (num - min_index + 1) + remainder_0;
    info!("Minimum with batch_size={} and concurrency={}", batch_min, concurrency_min);


    let batch_max = num - max_index + 1;
    let remainder_1 = if num % batch_max == 0 {0} else {1};
    let concurrency_max = num / (num - max_index + 1) + remainder_1;
    info!("Maximum with batch_size={} and concurrency={}", batch_max, concurrency_max);

    let time_min = &timer.data[num - batch_min];
    let time_max = &timer.data[num - batch_max];

    if let Some(w) = timer.writer.as_mut() {
        w.write_record(&[format!("{batch_max}"),
                         format!("{concurrency_max}"),
                         format!("{time_max}")]).unwrap();
        w.write_record(&[format!("{batch_min}"),
                         format!("{concurrency_min}"),
                         format!("{time_min}")]).unwrap();
        w.flush().unwrap();
    }
}
