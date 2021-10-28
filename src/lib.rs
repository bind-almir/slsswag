use std::io::Write;
use std::error::Error;
use rust_embed::RustEmbed;
use std::fs;

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

fn parse_yml(input: &str) -> Result<serde_yaml::Value, Box<dyn Error>> {
    let yml = fs::read_to_string(input)?;
    Ok(serde_yaml::from_str(&yml)?)
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


    println!("{}", &params.input);
    parse_yml(&params.input)?;
    
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