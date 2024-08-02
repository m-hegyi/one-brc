use std::{fs, io::{BufReader, Read}};

const PATH: &str = "/Users/mariohegyi/Work/rust/one-brc/data/measurements.txt";

//type MyMap = std::collections::HashMap<String, Vec<String>>;

fn with_buff(path: &str) {
    //let mut map: MyMap = std::collections::HashMap::new();
    let mut buffer = [0; 10240];

    let input = fs::File::open(path).unwrap();
    let mut reader = BufReader::new(input);
    
    let count = reader.read(&mut buffer).unwrap();

    println!("Count {count}");
    println!("{:?}", buffer);
    println!("{}", reader.capacity());
}

fn main() {
    //let mut map: MyMap = std::collections::HashMap::new();

    with_buff(PATH);

    //for line in fs::read_to_string(PATH).unwrap().lines() {
    //    let raw_split: Vec<&str> = line.split(";").collect();
    //
    //    let name = raw_split.get(0).unwrap().to_string();
    //    let data = raw_split.get(1).unwrap().to_string();
    //
    //    if map.contains_key(&name) {
    //        if let Some(entry_value) = map.get_mut(&name) {
    //            entry_value.push(data);
    //        }
    //    } else {
    //        map.insert(name, vec![data]);
    //    }
    //}

    //println!("Content length: {:?}", map);
}

