use core::{panic, str};
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::io::{BufReader, Read};

const NEW_LINE_CHAR: u8 = b'\n';
const DELIMETER: u8 = b';';

#[derive(Debug, PartialEq)]
struct Stats {
    min: f64,
    max: f64,
    sum: f64,
    count: usize,
}

impl From<f64> for Stats {
    fn from(item: f64) -> Self {
        Stats {
            min: item,
            max: item,
            sum: item,
            count: 1,
        }
    }
}

type MyMap<'a> = std::collections::HashMap<Vec<u8>, Stats>;
// key, min, mean, max 
type MyOrderedResult<'a> = std::collections::BTreeMap<&'a [u8], (String, String, String)>;
type DebugResult = std::collections::BTreeMap<String, (String, String, String)>;

fn calculate_instant_values(item: &mut Stats, new_value: f64) {
    if new_value < item.min {
        item.min = new_value;
    }

    if new_value > item.max {
        item.max = new_value;
    }

    item.count += 1; 
    item.sum += new_value;
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
        let avg = value.sum / value.count as f64;
       my_result.insert(key, round_values((&value.min, &avg, &value.max)));
    }
}

fn display_result(result: &MyOrderedResult, max_line: usize, skip: bool) {
    if skip {
        return;
    }

    for (city, (min, mean, max)) in result.iter().take(max_line) {
        let city = String::from_utf8(city.to_vec()).unwrap();
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

fn parse_line_from_buffer(my_map: &mut MyMap, line: &[u8], delimeter_index: usize) -> Result<(), &'static str>{
    let title = Vec::from(&line[..delimeter_index]);

    match String::from_utf8(title.clone()) {
        Ok(_) => {},
        Err(_err) => {
            println!("{:?}", String::from_utf8(title.to_vec()).unwrap()); 
            println!("{:?}", "Ban An\n;".as_bytes().to_vec());
            println!("{:?}", String::from_utf8(line.to_vec()));
        }
        
    }

    if title[0] == b'a' {
        return Err("Wrong char");
    }

    let temperature = unsafe { str::from_utf8_unchecked(&line[delimeter_index+1..]).parse().unwrap() };
    if my_map.contains_key(&title) {
        let item = my_map.get_mut(&title).unwrap();

        calculate_instant_values(item, temperature);
    } else {
        let stats = Stats::from(temperature);

        my_map.insert(title, stats); 
    }

    Ok(())
}

fn parse_buffer_to_line(buf: &[u8], my_map: &mut MyMap) {
    let mut last_new_line_index = 0;
    let mut delimeter_index = 0;
    for (index, data) in buf.iter().enumerate() {
        if data == &DELIMETER {
            delimeter_index = index;
        }
        if data == &NEW_LINE_CHAR {
            if delimeter_index > 0 {
                let (start_index, line_delimeter_index) = if last_new_line_index == 0 {
                    (0, delimeter_index)
                } else {
                    (last_new_line_index + 1, delimeter_index - last_new_line_index - 1)
                };

                let result = parse_line_from_buffer(my_map, &buf[start_index..index], line_delimeter_index);

                if result.is_err() {
                    panic!("Error happend: {}", result.err().unwrap());
                }
                
            }
            last_new_line_index = index;
        }
    }

    if delimeter_index == 0 && last_new_line_index > 1 {
        panic!("Invalid buffer: {:?}", buf);
    }
}

fn run(path: &Path, debug: bool) -> (usize, usize, Option<DebugResult>) {
    let file = fs::File::open(path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut buf = [0; 256 * 1024];

    let mut start_index = 0;
    let mut loop_counter = 0;

    let (sender, receiver) = crossbeam_channel::bounded::<Box<[u8]>>(100);

    let thread_nums: usize = std::thread::available_parallelism().unwrap().into();

    let mut handlers = Vec::with_capacity(thread_nums);

    for _ in 0..thread_nums {
        let rec = receiver.clone();
        let handler = std::thread::spawn(move || {
            let mut my_map = std::collections::HashMap::new();
            for buf_vector in rec {
                parse_buffer_to_line(buf_vector.as_ref(), &mut my_map)
            }

            my_map
        });     
        
        handlers.push(handler);
    }

    

    loop {
        let bytes_read = match buf_reader.read(&mut buf[start_index..]) {
            Ok(n) => { n },
            _ => panic!("Something went wrong"),
        };

        if bytes_read == 0 {
            break;
        }

        loop_counter += 1;
        if loop_counter % 1000 == 0 {
            println!("loop counter: {loop_counter}");
        }

        let usable_buf = &buf[..start_index + bytes_read];
        let last_new_line_pos = usable_buf.iter().rev().position(|&x| x == b'\n');

        match last_new_line_pos {
            Some(pos) => {
                let end_index = bytes_read + start_index;

                let boxed_buf = Box::from(&usable_buf[..end_index - pos]);
                let result = sender.send(boxed_buf);

                if result.is_err() {
                    panic!("{:?}", result.err());
                }

                buf.copy_within(end_index - pos..end_index, 0);
                start_index = pos;
            },
            None => {
                buf.copy_within(..start_index + bytes_read, 0);
                start_index = start_index + bytes_read;
            }
        }
    }

    drop(sender);

    let mut my_map: MyMap = std::collections::HashMap::new();

    for (index, handler) in handlers.into_iter().enumerate() {
        let result = handler.join().unwrap();
        if index == 0 {
            my_map.extend(result);
        } else {
            for (title, stats) in result {
                my_map.entry(title).and_modify(|s| {
                    s.min = s.min.min(stats.min);
                    s.max = s.max.max(stats.max);
                    s.sum += stats.sum;
                    s.count += stats.count; 
                }).or_insert(stats);
            }
        }

    }

    let mut my_result: MyOrderedResult = std::collections::BTreeMap::new(); 
    create_result(&my_map, &mut my_result);

    display_result(&my_result, my_result.len(), false);

    if debug {
        let mut debug_result = std::collections::BTreeMap::new();

        for (title, values) in my_result.iter() {
            let title = String::from_utf8(title.to_vec()).unwrap();
            debug_result.insert(title, (values.0.clone(), values.1.clone(), values.2.clone()));
        }

        return (loop_counter, my_result.len(), Some(debug_result));
    }

    (loop_counter, my_result.len(), None)
}

fn main() {
    let path = get_file_name();
    let now = std::time::Instant::now();

    let (line_count, city_count, _) = run(&path, false);

    let elapsed_time = now.elapsed();

    println!("----STATS----");
    println!("chunks read: {}, cities: {}, elapsed time: {}", line_count, city_count, elapsed_time.as_secs());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_parser_from_buffer() {
        let buff_input = String::from("Ranst;-13.7");
        let mut my_map: MyMap = std::collections::HashMap::new();

        let _ = parse_line_from_buffer(&mut my_map, buff_input.as_bytes(), 5);

        assert_eq!(my_map.len(), 1);

        let entries: Vec<_> = my_map.iter().take(1).collect();

        assert_eq!(entries[0].0, &Vec::from("Ranst".as_bytes()));
        assert_eq!(entries[0].1, &Stats::from(-13.7));

        let buff_input = String::from("Ranst;2.7");

        let _ = parse_line_from_buffer(&mut my_map, buff_input.as_bytes(), 5);

        assert_eq!(my_map.len(), 1);

        let buff_input = String::from("Longhua;-5.3");

        let _ = parse_line_from_buffer(&mut my_map, buff_input.as_bytes(), 7);

        assert_eq!(my_map.len(), 2);

        assert_eq!(my_map.get("Longhua".as_bytes()).unwrap(), &Stats::from(-5.3));

        let stats = Stats {
            min: -13.7,
            max: 2.7,
            sum: -11.0,
            count: 2,
        };
        assert_eq!(my_map.get("Ranst".as_bytes()).unwrap(), &stats);
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
        let (_line_counter, city_count, maybe_debug_result) = run(Path::new("data/test_measurements.txt"), true);

        let mut result = maybe_debug_result.unwrap();

        assert_eq!(city_count, 8876);
        //assert_eq!(line_counter, 100_000);
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
