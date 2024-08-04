# 1 billion row challenge in rust #

Current version: 0.1

## Build and run ## 

- clone the repository
- run the src/resource/create_measurements.py python file with 1_000_000_000 to generate the test data in the data folder
- build and run with release flag: `cargo run --release`
- 1 extra parameter can be provided, which is the file name in the data folder if that is not measurements.txt eg: `cargo run --release another_measurement.txt`

> Disclaimer: these result where achived on a 2018 Mac mini with Intel i5 and 8 Gb of RAM

## Version history ## 

### Version 0.2 ##

**Running time:** 365 seconds

Replacing the file reading method to a buf reader and only reading the file one line at a time

### Version 0.1 ### 

**Running time:** 630 seconds

Very basic implementation just to have a working solution
