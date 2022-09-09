mod predict;
use crate::predict::get_doppler_shift;
use std::io::{BufReader, BufWriter, BufRead, Write};
use std::fs::File;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use lazy_static::lazy_static;
use clap::{Parser, crate_version, crate_name};
use regex::Regex;
use chrono::NaiveDateTime;

#[derive(Parser, Debug)]
#[clap(name = crate_name!(), author = "Ariyan Eghbal <ariyan.eghbal@gmail.com>", version = crate_version!(), about = "Fixes Dopller Shift on Satellite Signals", long_about = None)]
struct Args {
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,
    #[clap(short = 'f', long = "freq", help="Frequency")]
    freq: f64,
    #[clap(short = 'r', long = "rate", help="Sample rate")]
    sample_rate: u32,
    #[clap(short = 'o', default_value="-", long = "output", help="Output RAW IQ file path or - for stdout")]
    output_file: String,
    #[clap(short = 'i', default_value="-", long = "input", help="Input RAW IQ file path or - for stdin")]
    input_file: String,
    #[clap(short = 'l', long = "location", help = "Observer location (lat,lon,alt)")]
    location: String,
    #[clap(long = "tle", help = "Satellite TLE (Two lines separated with any character)")]
    tle: String,
    #[clap(short = 't', long = "time", help = "Start time of observation (Unix timestamp or %Y%m%dT%H:%M:%S UTC)")]
    time: String,

}


fn main() {
    let args = Args::parse();
    lazy_static! {
        static ref TLE_RE: Regex = Regex::new(r"(?s)^(1 (?:[^ ]+ +){7}\d+).(2 (?:[^ ]+ +){6}[\d\.]+)$").unwrap();
    }

    let tle = args.tle.trim();
    let cap = TLE_RE.captures(tle).unwrap();
    let tle = (&cap[1], &cap[2]);
    if args.verbose{
        eprintln!("TLE:\n{}\n{}", tle.0, tle.1);
    }
    let file_name = args.input_file;
    let out_file_name = args.output_file;
    let sample_rate = args.sample_rate as f64;
    let start_time = match args.time.parse::<i64>() {
        Ok(v) => v,
        Err(_) => NaiveDateTime::parse_from_str(&args.time, "%Y%m%dT%H%M%S").unwrap().timestamp()
    };
    if args.verbose{
        let d = NaiveDateTime::from_timestamp(start_time, 0);
        eprintln!("Time: {} ({})", d.format("%Y-%m-%d %H:%M:%S UTC"), start_time);
    }
    let freq = args.freq;
    let mut location = args.location.splitn(3, ",").map(|x| x.parse::<f64>().unwrap());
    let latitude = location.next().unwrap();
    let longitude = location.next().unwrap();
    let alt = location.next().unwrap();
    if args.verbose{
        eprintln!("Location: {},{},{}", latitude, longitude, alt);
    }
    
    let mut writer: Box<dyn Write> = match out_file_name.as_str(){
        "-" => Box::new(BufWriter::new(std::io::stdout())),
        f => Box::new(BufWriter::with_capacity(8192, File::create(f).unwrap()))
    };
    
    let mut reader: Box<dyn BufRead> =  match file_name.as_str() {
        "-" => Box::new(BufReader::new(std::io::stdin())),
        f => Box::new(BufReader::with_capacity(8192, File::open(f).unwrap()))
    };

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

}
