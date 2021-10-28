use std::io::Write;
use std::error::Error;
use rust_embed::RustEmbed;
use std::fs;
use regex::Regex;

const OUTPUT: &str = "serverless.yml";

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
    let yml = fs::read_to_string(params.input)?;

    let value: serde_yaml::Value = serde_yaml::from_str(&yml).unwrap();

    let paths: &serde_yaml::Mapping = value["paths"]
        .as_mapping()
        .ok_or("paths is not a mapping or malformed")?;

    for (path, methods) in paths {
        for (method, _method_value) in methods.as_mapping().unwrap() {
            // println!("{:?}", method_value["produces"]);
            // println!("{:?}", method_value["consumes"]);
            
            let s = parse_nodejs(&path, &method);
            println!("{}", s);
        }
    }

    Ok(())
}

fn parse_nodejs(path: &serde_yaml::Value, method: &serde_yaml::Value) -> String {
    let mut nodejs_fn = read_template("nodejs-function.yml");
    let mut str_method = String::new();
    let mut str_path = String::new();            

    match method {
        serde_yaml::Value::String(value) => {
            str_method = value.clone();
            nodejs_fn = nodejs_fn.replace("[method]", value)

        },
        _ =>  nodejs_fn = "get".to_string(),
    };

    match path {
        serde_yaml::Value::String(value) => {                    
            str_path = value.clone();
            nodejs_fn = nodejs_fn.replace("[path]", value)
        },
        _ =>  nodejs_fn = "/".to_string(),
    };

    let mut function_name: String = str_path.to_owned();
    function_name.push_str(&str_method);


    let reg = Regex::new(r"/").unwrap();
    let function_name = reg.replace_all(&function_name, "");

    let reg = Regex::new(r"[^A-Za-z0-9]+").unwrap();

    let function_name = reg.replace_all(&function_name, "-");
    nodejs_fn = nodejs_fn.replace("[function-name]", &function_name);

    nodejs_fn

}

// write the output to the serverless.yml file
fn write_output(path: &str, content: &str) -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

// main function
pub fn run(params: Params) -> Result<(), Box<dyn Error>> {

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