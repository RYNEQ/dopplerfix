# Dopplerfix

Simple program which fixes doppler shift on satellite signals  
It uses [libpredict](https://github.com/la1k/libpredict) as doppler shift calculator 

Currently only supports 32bit floating point raw IQ files  


## Compile
- First install [libpredict](https://github.com/la1k/libpredict)  
- Compile 
    - Shared linkage with libpredict:
        ```bash
        cargo build --release
        ```
    - Static linkage with libpredict:
        ```bash
        RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-gnu
        ```

## Usage
```
dopplerfix 0.2.0
Ariyan Eghbal <ariyan.eghbal@gmail.com>
Fixes Dopller Shift on Satellite Signals

USAGE:
    dopplerfix [OPTIONS] --freq <FREQ> --rate <SAMPLE_RATE> --location <LOCATION> --tle <TLE> --time <TIME>

OPTIONS:
    -f, --freq <FREQ>             Frequency
    -h, --help                    Print help information
    -i, --input <INPUT_FILE>      Input RAW IQ file path or - for stdin [default: -]
    -l, --location <LOCATION>     Observer location (lat,lon,alt)
    -o, --output <OUTPUT_FILE>    Output RAW IQ file path or - for stdout [default: -]
    -r, --rate <SAMPLE_RATE>      Sample rate
    -t, --time <TIME>             Start time of observation (Unix timestamp or %Y%m%dT%H:%M:%S UTC)
        --tle <TLE>               Satellite TLE (Two lines separated with any character)
    -v, --verbose                 
    -V, --version                 Print version information

```

before and after doppler fix for NOAA:  
![Spectrum Before Shift](https://github.com/RYNEQ/dopplerfix/blob/master/img/before.png?raw=true)
![Spectrum After Shift](https://github.com/RYNEQ/dopplerfix/blob/master/img/after.png?raw=true)
