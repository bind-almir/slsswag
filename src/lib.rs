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
            
            let s = parse_yml(&path, &method, &params);
            write_output(OUTPUT, &s).expect("Error writing to the output file");
        }
    }


    // add general api info into api.yml
    create_api_yaml(&value["info"])?;
    // add models defined in the swagger into models.yml
    create_models_yml(&value["definitions"])?;

    Ok(())
}

fn update_model_ref(model_ref: String) -> String {
    let mut updated_ref = "{{".to_owned(); 
    updated_ref.push_str(model_ref.replace("#/definitions/", "").as_str());
    updated_ref.push_str("}}");
    return updated_ref;
}

fn traverse_model_map(mut model_map: serde_yaml::Mapping) -> serde_yaml::Value {
    let map = model_map.clone();
    for (key, value) in map {
        if value.is_mapping() {
            traverse_model_map(value.as_mapping().unwrap().clone());
        } else {
            if key == "$ref" {
                let mut mut_model_value: serde_yaml::Value = model_map[&key].clone();
                match mut_model_value {
                    serde_yaml::Value::String(ref mut s) => {
                        *s = update_model_ref(s.to_string());
                        model_map[&key] = serde_yaml::Value::String(s.to_string());
                        println!("{:?}", model_map[&key]);
                    },
                    _ => {
                        println!("{:?}", mut_model_value);
                    }
                }

            }
        }
    }
    serde_yaml::Value::Mapping(model_map)
}

fn create_models_yml(definitions: &serde_yaml::Value) -> Result<(), Box<dyn Error>> {

    const MODELS_YML: &str = "output/docs/models.yml";
    File::create(MODELS_YML)?;

    for (model, model_value) in definitions.as_mapping().unwrap() {
        let str_model: String;
        let mut str_model_value: String;
        match model {
            serde_yaml::Value::String(value) => {
                str_model = value.clone();    
            },
            _ =>  str_model = "".to_string(),
        };

        let model_map = model_value
            .as_mapping()
            .ok_or("model is not a mapping or malformed")?;

        let model_value: serde_yaml::Value = traverse_model_map(model_map.clone());

        match model_value {
            serde_yaml::Value::Mapping(value) => {
                str_model_value = serde_yaml::to_string(&value)?;
            },
            _ =>  str_model_value = "".to_string(),
        };

        let mut model_definition: String = "- \n  ".to_owned();
        model_definition.push_str("name: ");
        model_definition.push_str(&str_model);
        // TODO get this from path definition
        model_definition.push_str("\n  contentType: 'application/json'");
        model_definition.push_str("\n  schema: ");
        str_model_value = str_model_value.replace("\n", "\n    ");
        model_definition.push_str(&str_model_value);
        model_definition = model_definition.replace("---", "");

        write_output(MODELS_YML, &model_definition).expect("Error writing to the output models.yml file");
    }
    Ok(())
}

fn create_api_yaml(info: &serde_yaml::Value) -> Result<(), Box<dyn Error>>  {
    const API_YML: &str = "output/docs/api.yml";
    File::create(API_YML)?;
    let mut str_info = "info:".to_owned();
    let mut info_indented = serde_yaml::to_string(&info)?;
    info_indented = info_indented.replace("\n", "\n  ");
    info_indented = info_indented.replace("---", "");
    str_info.push_str(&info_indented.to_string());
    write_output(API_YML, &str_info).expect("Error writing to the output api.yml file");
    Ok(())
}

// fn create_docs() -> Result<(), Box<dyn Error>> {

//      TODO Add docs to functions 
//     Ok(())
// }

fn parse_yml(path: &serde_yaml::Value, method: &serde_yaml::Value, params: &Params) -> String {
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

    if params.runtime == "nodejs" {
        let mut function_handler = String::new();
        function_handler.push_str("functions/");
        function_handler.push_str(&function_name);
        let mut function_file = function_handler.clone();
        function_file.push_str(".js");
        function_handler.push_str(".handler");    
        std_fn = std_fn.replace("[function-handler]", &function_handler);
        let mut node_fn_dest = String::new();
        node_fn_dest.push_str("output/");
        node_fn_dest.push_str(&function_file);
        copy_template("node-function.js", &node_fn_dest).expect("Error copying the node function");
    } else if params.runtime == "csharp" {
        // TODO: implement csharp
    }

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
    fs::create_dir_all("./output/functions")?;
    fs::create_dir_all("./output/helpers")?;
    fs::create_dir_all("./output/docs")?;

    File::create(OUTPUT)?;
    Ok(())
}

fn copy_template(name: &str, dest: &str) -> Result<(), Box<dyn Error>> {
    let content = read_template(name);
    File::create(&dest)?;
    write_output(&dest, &content)?;
    Ok(())
}

// main function
pub fn run(params: Params) -> Result<(), Box<dyn Error>> {

    // create output directory and files
    setup_output()?;

    let content: &mut String = &mut String::new();

    if params.runtime == "nodejs" {
        // setup nodejs project
        copy_template("package.json", "output/package.json")?;
        copy_template("node-response.js", "output/helpers/parse-response.js")?;
        *content = read_template("base-nodejs.yml");
    } else if params.runtime == "csharp" {
        // TODO: setup csharp project
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