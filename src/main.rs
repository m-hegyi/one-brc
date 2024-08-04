use core::panic;
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};

static MAX_VAL: f64 = 100.0;
static MIN_VAL: f64 = -100.0;

type MyMap = std::collections::HashMap<String, Vec<f64>>;
// key, min, mean, max 
type MyResult = std::collections::HashMap<String, (String, String, String)>;
type MyOrderedResult = std::collections::BTreeMap<String, (String, String, String)>;

fn parse_line(my_map: &mut MyMap, line: &str) -> Option<bool> {
    let values: Vec<&str> = line.split(";").collect(); 

    let name = String::from(*values.get(0)?);

    let value = *values.get(1)?;
    let value = if value.contains("\n") {
        value[0..value.len() - 2].parse().expect("Invalid number found!")
    } else {
        value.parse().expect("Invalud number found!")
    };

    if my_map.contains_key(&name) {
        my_map.get_mut(&name)?.push(value);
    } else {
        my_map.insert(name, vec![value]);
    }
    Some(true)
}

fn round_value(value: f64) -> String {
    format!("{:.1}", (value * 10.0).round() / 10.0)
}

fn calculate_values(input: &Vec<f64>) -> (String, String, String) {
    let mut max = MIN_VAL;
    let mut min = MAX_VAL;
    let mut sum = 0.0;

    for value in input.iter() {
        if value > &max {
            max = *value;
        }

        if value < &min {
            min = *value;
        }

        sum += *value;
    }

    let mean = sum / input.len() as f64;

    (round_value(min), round_value(mean), round_value(max))
}

fn calculate_result(my_map: &MyMap) -> MyResult {
    let mut result: MyResult = std::collections::HashMap::with_capacity(my_map.len());

    for (key, value) in my_map.iter() {
       let number_results = calculate_values(value); 

       result.insert(key.clone(), number_results);
    }

    result
}

fn order_result(result: MyResult) -> MyOrderedResult {
    let mut map = std::collections::BTreeMap::new();

    for (key, values) in result {
        map.insert(key.clone(), values.clone());
    }

    map
}

fn display_result(result: &MyOrderedResult, max_line: usize) {
    for (city, (min, mean, max)) in result.iter().take(max_line) {
        println!("{city};{min};{mean};{max}");
    } 
}

fn get_file_name() -> PathBuf {
    let path = Path::new("data/");

    let path = match env::args().len() {
        2 => {
            let file_name = env::args().last().unwrap();

            path.join(file_name)
        }
        _ => path.join("measurements.txt"),
    };

    if let Err(_) = path.try_exists() {
        panic!("Invalid file path: {:?}", path.to_str());
    }

    path 
}

fn run(path: &Path) -> (usize, MyOrderedResult) {
    let mut line_counter = 0;
    let mut my_map: MyMap = std::collections::HashMap::new();
    
    let file = fs::File::open(path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut content = String::new();

    loop {
        match buf_reader.read_line(&mut content) {
            Ok(0) => break,
            Ok(_) => {
                line_counter += 1;
                parse_line(&mut my_map, &content);

                if line_counter % 10_000_000 == 0 {
                    println!("{line_counter}");
                }

                content.clear();
            },
            _ => panic!("Something went wrong"),
        }
    }

    let result = order_result(calculate_result(&my_map));

    display_result(&result, result.len());

    (line_counter, result)
}

fn main() {
    let path = get_file_name();
    let now = std::time::Instant::now();

    let (line_counter, result) = run(&path);

    let elapsed_time = now.elapsed();

    println!("----STATS----");
    println!("lines read: {}, cities: {}, elapsed time: {}", line_counter, result.len(), elapsed_time.as_secs());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_parser() {
        let test_str = String::from("Ranst;-13.7");
        let test_str2 = String::from("Ranst;2.7");
        let test_str3 = String::from("Longhua;-5.3");

        let mut my_map: MyMap = std::collections::HashMap::new();

        parse_line(&mut my_map, &test_str);

        assert_eq!(my_map.get("Ranst").unwrap(), &vec![-13.7]);

        parse_line(&mut my_map, &test_str2);

        assert_eq!(my_map.len(), 1);
        assert_eq!(my_map.get("Ranst").unwrap(), &vec![-13.7, 2.7]);

        parse_line(&mut my_map, &test_str3);

        assert_eq!(my_map.len(), 2);
        assert_eq!(my_map.get("Longhua").unwrap(), &vec![-5.3]);
        assert_eq!(my_map.get("Ranst").unwrap(), &vec![-13.7, 2.7]);
    }

    #[test]
    fn test_round_value() {
        let input = 10.0;
        
        assert_eq!(round_value(input), String::from("10.0"));

        let input = -20.0;

        assert_eq!(round_value(input), String::from("-20.0"));

        let input = 1.0 / 3.0 * 2.0;

        assert_eq!(round_value(input), String::from("0.7"));
    }

    #[test]
    fn test_calculate_values() {
        let input = vec![10.0, -20.0, 5.0];

        assert_eq!(calculate_values(&input), (
                String::from("-20.0"), 
                String::from("-1.7"),
                String::from("10.0")))
    }

    #[test]
    fn test_order_cities() {
        let mut input: MyResult = std::collections::HashMap::new();

        let tmp = "".to_string();

        let tmp_data = (tmp.clone(), tmp.clone(), tmp.clone());

        input.insert("ACity".to_string(), tmp_data.clone());
        input.insert("CCity".to_string(), tmp_data.clone());
        input.insert("BCity".to_string(), tmp_data.clone());


        let res = order_result(input);
        
        let keys: Vec<_> = res.keys().cloned().collect();

        assert_eq!(keys, vec!["ACity", "BCity", "CCity"]);
    }
}
