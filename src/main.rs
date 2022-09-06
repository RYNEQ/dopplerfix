mod predict;
use crate::predict::get_doppler_shift;
use std::io::{BufReader, BufWriter};
use std::fs::File;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use clap::Parser;


#[derive(Parser, Debug)]
#[clap(name = "Doppler Fix", author = "Ariyan Eghbal <ariyan.eghbal@gmail.com>", version = "0.1.1", about = "Fixes Dopller Shift on Satellite Signals", long_about = None)]
struct Args {
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,
    #[clap(short = 'f', long = "freq", help="Frequency")]
    freq: f64,
    #[clap(short = 'r', long = "rate", help="Sample rate")]
    sample_rate: u32,
    #[clap(short = 'o', long = "output", help="Output RAW IQ file path")]
    output_file: String,
    #[clap(short = 'i', long = "input", help="Input RAW IQ file path")]
    input_file: String,
    #[clap(short = 'l', long = "location", help = "Observer location (lat,lon,alt)")]
    location: String,
    #[clap(long = "tle", help = "Satellite TLE")]
    tle: String,
    #[clap(short = 't', long = "time", help = "Start time of observation (Unix timestamp UTC)")]
    time: i64,

}


fn main() {
    let args = Args::parse();

    let tle = args.tle.trim().split_once('\n').unwrap();
    let file_name = args.input_file;
    let out_file_name = args.output_file;
    let sample_rate = args.sample_rate as f64;
    let start_time = args.time;
    let freq = args.freq;
    let mut location = args.location.splitn(3, ",").map(|x| x.parse::<f64>().unwrap());
    let latitude = location.next().unwrap();
    let longitude = location.next().unwrap();
    let alt = location.next().unwrap();

    let file = File::open(file_name).unwrap();
    let sample_count = (file.metadata().unwrap().len() as usize / 4 / 2) as usize;
    let outfile = File::create(out_file_name).unwrap();
    let mut writer = BufWriter::with_capacity(8192, outfile);
    let mut reader = BufReader::with_capacity(8192, file);

    let m = num::complex::Complex64::new(0.0, 1.0)*2.0*std::f64::consts::PI;
    let mut idx = 1usize;
    let mut shift: Option<f64> = None;
    loop {
            let i = reader.read_f32::<LittleEndian>(); 
            let q = reader.read_f32::<LittleEndian>();
            match (i,q){
                (Ok(i), Ok(q)) => {
                    let sample = num::complex::Complex64::new(i as f64, q as f64);
                    if shift.is_none() || idx % sample_rate as usize  == 0 {
                        shift = Some(-1.0*get_doppler_shift(tle, latitude, longitude, alt, start_time+(idx as f64/sample_rate) as i64, freq));
                    }
                    let t = idx as f64/sample_rate;
                    let new_sample = (m*(shift.unwrap() as f64*t)).exp() * sample;
                    writer.write_f32::<LittleEndian>(new_sample.re as f32).unwrap();
                    writer.write_f32::<LittleEndian>(new_sample.im as f32).unwrap();
                },
                _ => {
                    break;
                }
            }
            idx += 1;
    }

    if idx-1 != sample_count {
        panic!("Sample count mismatch");
    }
}
