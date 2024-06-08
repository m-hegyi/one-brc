use core::panic;
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};

type MyMap<'a> = std::collections::HashMap<String, (f64, f64, f64, usize)>;
// key, min, mean, max 
type MyOrderedResult<'a> = std::collections::BTreeMap<&'a str, (String, String, String)>;
type DebugResult = std::collections::BTreeMap<String, (String, String, String)>;

fn parse_line(my_map: &mut MyMap, line: &str) -> Option<bool> {
    let mut split_index = 0;
    for (index, value) in line.char_indices() {
        if value == ';' {
            split_index = index;
        }
    }

    if split_index == 0 {
        panic!("split not found");
    }

    let name = &line[0..split_index];

    let value = &line[split_index+1..];
    let value = if value.contains("\n") {
        value[0..value.len() - 1].parse().expect("Invalid number found!")
    } else {
        value.parse().expect("Invalid number found!")
    };

    if my_map.contains_key(name) {
        let item = my_map.get_mut(name).unwrap();
        
        calculate_instant_values(item, value);
    } else {
        my_map.insert(name.to_string(), (value, value, value, 1));
    }
    Some(true)
}

fn calculate_instant_values(item: &mut (f64, f64, f64, usize), new_value: f64) {
    if new_value < item.0 {
        item.0 = new_value;
    }

    if new_value > item.2 {
        item.2 = new_value;
    }

    item.1 = ((item.1 * item.3 as f64) + new_value) / (item.3 + 1) as f64;

    item.3 += 1;
}

fn round_values(values: (&f64, &f64, &f64)) -> (String, String, String) {
    (
        format!("{:.1}", (values.0 * 10.0).round() / 10.0),
        format!("{:.1}", (values.1 * 10.0).round() / 10.0),
        format!("{:.1}", (values.2 * 10.0).round() / 10.0)
    )
}

fn create_result<'a ,'b>(my_map: &'a MyMap<'a>, my_result: &'b mut MyOrderedResult<'a>) {
    for (key, value) in my_map.iter() {
       my_result.insert(key, round_values((&value.0, &value.1, &value.2)));
    }
}

fn display_result(result: &MyOrderedResult, max_line: usize, skip: bool) {
    if skip {
        return;
    }

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

fn run(path: &Path, debug: bool) -> (usize, usize, Option<DebugResult>) {
    let mut my_map: MyMap = std::collections::HashMap::new();
    let mut my_result: MyOrderedResult = std::collections::BTreeMap::new(); 

    let mut line_counter = 0;

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

    create_result(&my_map, &mut my_result);

    display_result(&my_result, my_result.len(), false);

    if debug {
        let mut debug_result = std::collections::BTreeMap::new();

        for (title, values) in my_result.iter() {
            debug_result.insert(title.to_string(), (values.0.clone(), values.1.clone(), values.2.clone()));
        }

        return (line_counter, my_result.len(), Some(debug_result));
    }

    (line_counter, my_result.len(), None)
}

fn main() {
    let path = get_file_name();
    let now = std::time::Instant::now();

    let (line_count, city_count, _) = run(&path, false);

    let elapsed_time = now.elapsed();

    println!("----STATS----");
    println!("lines read: {}, cities: {}, elapsed time: {}", line_count, city_count, elapsed_time.as_secs());
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

        assert_eq!(my_map.get("Ranst").unwrap(), &(-13.7, -13.7, -13.7,  1));

        parse_line(&mut my_map, &test_str2);

        assert_eq!(my_map.len(), 1);
        assert_eq!(my_map.get("Ranst").unwrap(), &(-13.7, -5.5, 2.7, 2));

        parse_line(&mut my_map, &test_str3);

        assert_eq!(my_map.len(), 2);
        assert_eq!(my_map.get("Longhua").unwrap(), &(-5.3, -5.3, -5.3, 1));
        assert_eq!(my_map.get("Ranst").unwrap(), &(-13.7, -5.5, 2.7, 2));
    }

    #[test]
    fn test_round_value() {
        let input = (10.0, -20.0, 1.0 / 3.0 * 2.0);
        
        assert_eq!(
            round_values((&input.0, &input.1, &input.2)), 
            ("10.0".to_string(), "-20.0".to_string(), "0.7".to_string())
        );
    }

    #[test]
    fn test_run() {
        let (line_counter, city_count, maybe_debug_result) = run(Path::new("data/test_measurements.txt"), true);

        let mut result = maybe_debug_result.unwrap();

        assert_eq!(city_count, 8876);
        assert_eq!(line_counter, 100_000);
        let last = result.pop_last().unwrap();

        assert_eq!(last.0, "’Aïn Roua".to_string());
        assert_eq!(last.1, (
                "-79.2".to_string(),
                "-6.7".to_string(),
                "59.8".to_string(),
        ));

        let item = result.get("Szentgotthárd").unwrap();
        assert_eq!(item, &(
                "-59.5".to_string(),
                "-10.4".to_string(),
                "56.9".to_string(),
        ));
    }
}
