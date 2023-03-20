use std::env;
use std::fs::File;
use std::io::prelude::*;
use hex;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum RecType {
        Undefined,
        Almanac,
        Ephemeris,
}

impl Default for RecType {
        fn default() -> Self {
                RecType::Undefined
        }
}

#[derive(Debug, Default)]
struct SatRec {
        satid: u16,
        week: u16,
        toa: u8,
        toe: u32,
        toc: u16,
        rec_type: RecType,
        nmea: String,
}

#[derive(Debug, Default)]
struct  GpsAlmanac{
        satid: u8,
        week: u16,
        toa: u8,
        rem: Vec<u8>,
}

impl GpsAlmanac {
        fn fill(bytes: Vec<u8> ) -> GpsAlmanac {
                return Self{
                        satid: bytes[0],
                        week: (u16::from(bytes[2])).checked_shl(8).unwrap_or(0) + u16::from(bytes[1]),
                        toa: bytes[3],
                        rem: bytes[4..].to_vec(),
                }
        }
}

fn fill_gps_almanac( bytes: Vec<u8> ) -> GpsAlmanac {
        let mut data: GpsAlmanac = Default::default();

        data.satid = bytes[0];
        data.week = (u16::from(bytes[2])).checked_shl(8).unwrap_or(0) + u16::from(bytes[1]);
        data.toa = bytes[3];
        data.rem = bytes[4..].to_vec();    

        return data;
}

#[derive(Debug, Default)]
struct GpsEphemeris {
        week: u16,
        toe: u16,
        toc: u16,
        rem: Vec<u8>,
}

impl GpsEphemeris {
        fn fill(bytes: Vec<u8> ) -> GpsEphemeris {
                return Self{
                        week: (u16::from(bytes[1])).checked_shl(8).unwrap_or(0) + u16::from(bytes[0]),
                        toe: (u16::from(bytes[3])).checked_shl(8).unwrap_or(0) + u16::from(bytes[2]),
                        toc: (u16::from(bytes[5])).checked_shl(8).unwrap_or(0) + u16::from(bytes[4]),
                        rem: bytes[6..].to_vec(),
                }
        }
}

fn fill_gps_ephemeris( bytes: Vec<u8> ) -> GpsEphemeris {
        let mut data: GpsEphemeris = Default::default();

        data.week = (u16::from(bytes[1])).checked_shl(8).unwrap_or(0) + u16::from(bytes[0]);
        data.toe = (u16::from(bytes[3])).checked_shl(8).unwrap_or(0) + u16::from(bytes[2]);
        data.toc = (u16::from(bytes[5])).checked_shl(8).unwrap_or(0) + u16::from(bytes[4]);
        data.rem = bytes[5..].to_vec();

        return data;
}

#[derive(Debug, Default)]
struct GlonassAlmanac{
        satid: u8,
        week: u16,
        toa: u8,
        rem: Vec<u8>,
}

fn fill_glonass_alamanc( bytes: Vec<u8> ) -> GlonassAlmanac {
        let mut data: GlonassAlmanac = Default::default();

        data.satid = bytes[0];
        data.week = (u16::from(bytes[2])).checked_shl(8).unwrap_or(0) + u16::from(bytes[1]);
        data.toa = bytes[3];
        data.rem = bytes[4..].to_vec();    

        return data;
}

#[derive(Debug, Default)]
struct GlonassEphemeris{
        week: u16,
        toe: u32,
        rem: Vec<u8>,
}

fn fill_glonass_ephemeris( bytes: Vec<u8> ) -> GlonassEphemeris {
        let mut data: GlonassEphemeris = Default::default();

        data.week = (u16::from(bytes[1])).checked_shl(8).unwrap_or(0) + u16::from(bytes[0]);
        data.toe = (u32::from(bytes[3])).checked_shl(12).unwrap_or(0) + (u32::from(bytes[2])).checked_shl(4).unwrap_or(0) + u32::from(bytes[4]);
        data.rem = bytes[5..].to_vec();

        return data;
}

#[derive(Debug, Default)]
struct GalileoAlmanac {
        satid: u16,
        svid: u8,
        week: u16,
        toa: u8,
        rem: Vec<u8>,
}

fn fill_galileo_almanac( bytes: Vec<u8> ) -> GalileoAlmanac {
        let mut data: GalileoAlmanac = Default::default();

        data.satid = (u16::from(bytes[1])).checked_shl(8).unwrap_or(0) + u16::from(bytes[0]);
        data.svid = bytes[2];
        data.week = (u16::from(bytes[4])).checked_shl(8).unwrap_or(0) + u16::from(bytes[3]);
        data.toa = bytes[5];
        data.rem = bytes[6..].to_vec();    

        return data;
}

fn check_checksum( chars: &str, sum: u8 ) -> Result<u8, String> {
        let mut checksum: u8 = 0;

        for c in chars.bytes() {
                checksum ^= c;
        }

        if checksum == sum {
                return Ok(sum)
        } else {
                println!("sum {} {}", checksum, sum);
                return Err("checksum failed".to_string());
        }
}

fn process_line( line: &str ) -> Result<SatRec, &str> {
        let fields = line.split(",");
        let field: Vec<&str> = fields.collect();
        let satid: u16;
        let len: i32;
        let mut sat: SatRec = Default::default();

        if field.len() != 4 {
                return Err("malformed line");
        }

        if field[0].eq("$PSTMALMANAC") {
                println!("reading almanac record\n");
                sat.rec_type = RecType::Almanac;
        } else if field[0].eq("$PSTMEPHEM") {
                println!("reading ephemeris record\n");
                sat.rec_type = RecType::Ephemeris;
        } else {
                return Err("unknown record");
        }

        match field[1].parse::<u16>() {
                Ok(n) => satid = n,
                Err(_) => return Err("No sat id found"), 
        }

        match field[2].parse::<i32>() {
                Ok(n) => len = n,
                Err(_) => return Err("No length found"),
        }

        let record: Vec<&str> = field[3].split("*").collect();
        let sum =  hex::decode(record[1]).expect("failed to decode crc");
        let bytes = hex::decode(record[0]).expect("failed to decode record");
        let chars: Vec<&str> = line.split("*").collect();

        //println!("records {} {:?}", field[3], bytes);

        match check_checksum( &chars[0][1..], sum[0] ) {
                Ok(_) => (),
                Err(_) => return Err("cehcksum failed"),
        }

        if sat.rec_type == RecType::Almanac {
                if len != 40 { 
                        return Err("incorrect length");
                }

                if satid <= 32 {
                        let gps_rec = fill_gps_almanac(bytes);
                        //let gps_rec = GpsAlmanac::fill(bytes);
                        println!("gps {:?}", gps_rec);
                        sat.satid = gps_rec.satid as u16;
                        sat.week = gps_rec.week;
                        sat.toa = gps_rec.toa;
                } else if satid >= 33 && satid <= 96 {
                        let glonass_rec = fill_glonass_alamanc(bytes);
                        println!("glonass {} {:?}", satid, glonass_rec);
                        sat.satid = glonass_rec.satid as u16;
                        sat.week = glonass_rec.week;
                        sat.toa = glonass_rec.toa;
                } else if satid >= 301 && satid <= 336 {
                        let galileo_rec = fill_galileo_almanac(bytes);
                        println!("galileo {} {:?}", satid, galileo_rec);
                        sat.satid = galileo_rec.satid;
                        sat.week = galileo_rec.week;
                        sat.toa = galileo_rec.toa;
                }
        } else if sat.rec_type == RecType::Ephemeris {
                if len != 64 { 
                        return Err("incorrect length");
                }

                if satid <= 32 {
                        let gps_rec = fill_gps_ephemeris(bytes);
                        //let gps_rec = GpsEphemeris::fill(bytes);
                        println!("gps ephemeris {:?}", gps_rec);
                        sat.satid = satid;
                        sat.week = gps_rec.week;
                        sat.toc = gps_rec.toc;
                        sat.toe = gps_rec.toe as u32;
                } else if satid >= 33 && satid <= 96 {
                        let glonass_rec = fill_glonass_ephemeris(bytes);
                        println!("glonass {} {:?}", satid, glonass_rec);
                        sat.satid = satid;
                        sat.week = glonass_rec.week;
                        sat.toe = glonass_rec.toe;
                }
        }

        sat.nmea = line.to_string();

        return Ok(sat);
}

fn compare_date( rec1: &SatRec, rec2: &SatRec ) -> Result<i8, String> {
        if rec1.rec_type != rec2.rec_type {
                return Err("can't compare different types".to_string());
        }

        if rec1.rec_type == RecType::Almanac {
                return compare_almanac_date(rec1, rec2);
        }

        if rec1.rec_type == RecType::Ephemeris {
                return compare_ephemeris_date(rec1, rec2);
        }

        return Err("Invalid date type".to_string());
}

fn compare_almanac_date( rec1: &SatRec, rec2: &SatRec ) -> Result<i8, String> {
        if rec1.rec_type != rec2.rec_type {
                return Err("can't compare different types".to_string());
        }

        if rec1.week > rec2.week {
                return Ok(1);
        }

        if rec1.week < rec2.week {
                return Ok(-1);
        }

        if rec1.toa > rec2.toa {
                return Ok(2);
        }

        if rec1.toa < rec2.toa {
                return Ok(-2);
        }

        return Ok(0);
}

fn compare_ephemeris_date( rec1: &SatRec, rec2: &SatRec ) -> Result<i8, String> {
        if rec1.rec_type != rec2.rec_type {
                return Err("can't compare different types".to_string());
        }

        if rec1.week > rec2.week {
                return Ok(1);
        }

        if rec1.week < rec2.week {
                return Ok(-1);
        }

        if rec1.toe > rec2.toe {
                return Ok(2);
        }

        if rec1.toe < rec2.toe {
                return Ok(-2);
        }

        if rec1.toc > rec2.toc {
                return Ok(3);
        }

        if rec1.toc < rec2.toc {
                return Ok(-3);
        }


        return Ok(0);
}

fn process_file(filename: &str, sats: &mut HashMap<u16, SatRec>) {
        let mut f =  File::open(filename).expect("file not found");
        let mut f_contents = String::new();
        let mut sat: SatRec;

        f.read_to_string(&mut f_contents).expect("file read failed");

        println!("in file\n{}", f_contents);

        let lines = f_contents.split('\n');

        for line in lines {
                match process_line(line) {
                        Ok(s) => sat = s,
                        Err(e) => {
                                println!("Failed processing record: {}", e);
                                continue;
                        },
                }

                if sat.satid == 0 {
                        continue;
                }

                if !sats.contains_key(&sat.satid) {
                        println!("Inserting {:?}", sat);
                        sats.insert(sat.satid, sat);
                } else {
                        let old_sat: &SatRec;
                        let cmp: i8;

                        old_sat = sats.get(&sat.satid).unwrap();
                       
                        match compare_date(&sat, &old_sat) {
                                Ok(n) => cmp = n,
                                Err(e) => { println!("{}", e); continue; }
                        }

                        if cmp > 0 {
                                println!("Replacing {:?}", sat);
                                sats.remove(&sat.satid);
                                sats.insert(sat.satid, sat);
                        } else {
                                println!("Skipping {:?} {:?}", sat, old_sat);
                        }
                }
        }
}

fn main() {
        let args: Vec<String> = env::args().collect();

        let infile0 = &args[1];
        let infile1 = &args[2];
        let mut sats: HashMap<u16, SatRec> = HashMap::new();
        let mut sorted_sats: Vec<(&u16, &SatRec)>;
        let mut file: std::fs::File;

        process_file(infile0, &mut sats);
        process_file(infile1, &mut sats);

        sorted_sats = sats.iter().collect();
        sorted_sats.sort_by(|a, b| a.1.satid.cmp(&b.1.satid));

        if args.len() > 3 {
                file = std::fs::File::create(&args[3]).expect("create failed");
                for (_ ,sat) in sorted_sats {
                        file.write_all(sat.nmea.as_bytes()).expect("write failed");
                        file.write_all("\n".as_bytes()).expect("write failed");
                }
        } else {
                for (_, sat) in sats {
                        println!("sat {:?}", sat);
                }
        }

}
