use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::error::Error;
use rust_embed::RustEmbed;
use std::fs;
use regex::Regex;

const OUTPUT: &str = "output/serverless.yml";

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Asset;

// input arguments
pub struct Params {
  pub input: String,
  pub runtime: String
}

// parse input arguments and return a Params struct
impl Params {
    pub fn new(args: &[String]) -> Result<Params, &str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }
      
        let input = args[1].to_string();
        let runtime = args[2].to_string();

        if runtime != "nodejs" && runtime != "csharp" {
            return Err("runtime must be nodejs or csharp");
        }

        Ok(Params { input, runtime })
  }
}

// read the base template from the templates folder
fn read_template(name: &str) -> String {
    let template = Asset::get(name).unwrap();
    std::str::from_utf8(template.data.as_ref()).unwrap().to_string()
}

fn parse_swagger(params: Params) -> Result<(), Box<dyn Error>> {
    let yml = fs::read_to_string(&params.input)?;

    let value: serde_yaml::Value = serde_yaml::from_str(&yml).unwrap();

    let paths: &serde_yaml::Mapping = value["paths"]
        .as_mapping()
        .ok_or("paths is not a mapping or malformed")?;

    for (path, methods) in paths {
        for (method, _method_value) in methods.as_mapping().unwrap() {
            // println!("{:?}", method_value["produces"]);
            // println!("{:?}", method_value["consumes"]);
            
            let s = parse_yml(&path, &method);
            write_output(OUTPUT, &s).expect("Error writing to the output file");
        }
    }

    Ok(())
}

fn parse_yml(path: &serde_yaml::Value, method: &serde_yaml::Value) -> String {
    let mut std_fn = read_template("function.yml");
    let mut str_method = String::new();
    let mut str_path = String::new();            

    match method {
        serde_yaml::Value::String(value) => {
            str_method = value.clone();
            std_fn = std_fn.replace("[method]", value)

        },
        _ =>  std_fn = "get".to_string(),
    };

    match path {
        serde_yaml::Value::String(value) => {                    
            str_path = value.clone();
            std_fn = std_fn.replace("[path]", value)
        },
        _ =>  std_fn = "/".to_string(),
    };

    let mut function_name: String = str_path.to_owned();
    function_name.push_str(&str_method);


    let reg = Regex::new(r"/").unwrap();
    let function_name = reg.replace_all(&function_name, "");

    let reg = Regex::new(r"[^A-Za-z0-9]+").unwrap();

    let function_name = reg.replace_all(&function_name, "-");
    std_fn = std_fn.replace("[function-name]", &function_name);

    std_fn

}

// write the output to the serverless.yml file
fn write_output(path: &str, content: &str) -> Result<(), Box<dyn Error>> {
    
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();

    if let Err(e) = writeln!(file, "{}\n", content) {
        eprintln!("Error writing to file: {}", e);
    }

    Ok(())
}

fn setup_output() -> Result<(), Box<dyn Error>> {
    fs::create_dir_all("./output/")?;
    File::create(OUTPUT)?;
    Ok(())
}

// main function
pub fn run(params: Params) -> Result<(), Box<dyn Error>> {

    // create output directory and files
    setup_output()?;

    let content: &mut String = &mut String::new();

    if params.runtime == "nodejs" {
        *content = read_template("base-nodejs.yml");
    } else if params.runtime == "csharp" {
        *content = read_template("base-csharp.yml");
    } else {
        panic!("runtime must be nodejs or csharp");
    }

    if let Err(e) = write_output(OUTPUT,  &content) {
        println!("Error writing to file {}", OUTPUT);
        println!("{}", e);
    }

    parse_swagger(params)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_result() {        
        assert_eq!(2, 2);
    }
}