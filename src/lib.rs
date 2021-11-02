use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::error::Error;
use rust_embed::RustEmbed;
use std::fs;
use regex::Regex;

const OUTPUT: &str = "output/serverless.yml";
const FUNCTION_DOC_BASE_PATH: &str = "output/docs/functions/";
const TESTS_BASE_PATH: &str = "output/tests/";

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Asset;

// input arguments
pub struct Params {
  pub input: String,
  pub runtime: String
}

struct Yaml {
    path: serde_yaml::Value, 
    method: serde_yaml::Value, 
    params: Params, 
    method_value: serde_yaml::Value
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

fn parse_swagger(params: &Params) -> Result<(), Box<dyn Error>> {
    let yml = fs::read_to_string(&params.input)?;

    let value: serde_yaml::Value = serde_yaml::from_str(&yml).unwrap();

    let paths: &serde_yaml::Mapping = value["paths"]
        .as_mapping()
        .ok_or("paths is not a mapping or malformed")?;

    for (path, methods) in paths {
        for (method, method_value) in methods.as_mapping().unwrap() {
             let yaml: Yaml = Yaml {
                path: path.clone(),
                method: method.clone(),
                params: Params {
                    input: params.input.clone(),
                    runtime: params.runtime.clone()
                },
                method_value: method_value.clone()
            };
            let s = parse_yml(yaml);
            write_output(OUTPUT, &s).expect("Error writing to the output file");
        }
    }

    // add general api info into api.yml
    create_api_yaml(&value["info"])?;
    // add models defined in the swagger into models.yml
    create_models_yml(&value["definitions"])?;

    Ok(())
}

fn replace_model_references(model_definition: &str) -> String {
    let new_model_definition = &model_definition.replace("#/definitions/", "{{model:");

    let mut lines = String::new();
    for ln in new_model_definition.lines() {
        if ln.contains("{{model:") {
            let mut updated_line = ln.to_owned();
            updated_line = updated_line[0..updated_line.len() - 1].to_string();
            // updated_line = updated_line.replace(ln, "#/definitions/");
            updated_line.push_str("}}\"\n");
            lines.push_str(&updated_line);

        } else {
            lines.push_str(ln);
            lines.push_str("\n");
        }
    }
    lines
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

        // TODO: LEARN RUST fix this hack! /1
        model_definition = replace_model_references(&model_definition);

        write_output(MODELS_YML, &model_definition).expect("Error writing to the output models.yml file");

    }
    Ok(())
}


fn create_file(file: &str) -> Result<(), Box<dyn Error>> {
    File::create(file)?;
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


fn create_function_docs(yaml: Yaml, file: &str) -> Result<(), Box<dyn Error>> {
    let (path, method, _params, method_value) = (&yaml.path, &yaml.method, &yaml.params, &yaml.method_value);
    let mut doc = String::new();
    
    doc.push_str("summary: ");
    doc.push_str(method_value["summary"].as_str().unwrap());
    doc.push_str("\n");
    doc.push_str("description: ");
    doc.push_str(method_value["description"].as_str().unwrap());
    doc.push_str("\n");
    
    match &method_value["tags"] {
        serde_yaml::Value::Sequence(tags) => {
            doc.push_str("tags: \n");

            for tag in tags {
                doc.push_str("  - ");
                doc.push_str(tag.as_str().unwrap());
                doc.push_str("\n");
            }
        },
        _ => {},
    };

    match &method_value["parameters"] {
        serde_yaml::Value::Sequence(parameters) => {
            doc.push_str("pathParameters:\n");
            for param in parameters {
                if param["in"] == "path".to_string() {
                    match &param["name"] {
                        serde_yaml::Value::String(value) => {
                            doc.push_str("  - name: ");
                            doc.push_str(&value);
                            doc.push_str("\n");
                        },
                        _ => {},
                    };
                }
            }
        },
        _ => {}
    };

    match &method_value["parameters"] {
        serde_yaml::Value::Sequence(parameters) => {
            doc.push_str("queryStringParameters:\n");
            for param in parameters {
                if param["in"] == "query".to_string() {
                    match &param["name"] {
                        serde_yaml::Value::String(value) => {
                            doc.push_str("  - name: ");
                            doc.push_str(&value);
                            doc.push_str("\n");
                        },
                        _ => {},
                    };
                }
            }
        },
        _ => {}
    };

    match method {
        serde_yaml::Value::String(value) => {
            doc.push_str("method: ");
            doc.push_str(value);
            doc.push_str("\n");        },
        _ =>  println!("method not found "),
    };

    match path {
        serde_yaml::Value::String(value) => {                    
            doc.push_str("path: ");
            doc.push_str(value);
            doc.push_str("\n");
        },
        _ =>  println!("path not found "),
    };

    doc.push_str("methodResponses: \n");

    match &method_value["responses"] {
        serde_yaml::Value::Mapping(responses) => {
            for (code, response) in responses {
                doc.push_str("-\n");
                doc.push_str("  statusCode: ");
                let mut code_str = code.as_str().unwrap().to_string();
                if code_str == "default" {
                    code_str.clear();
                    code_str.push_str("200");
                }
                doc.push_str(code_str.as_str());
                doc.push_str("\n");
                doc.push_str("  description: ");
                doc.push_str(response["description"].as_str().unwrap());
                doc.push_str("\n");

                // TODO: parse response models properly
                // if response["schema"] != serde_yaml::Value::Null {
                //     let mut s = serde_yaml::to_string(&response["schema"])?;
                //     doc.push_str("  responseModels:");
                //     s = s.replace("\n", "\n    ");
                //     doc.push_str(&s);
                //     doc.push_str("\n");
                //     doc = doc.replace("---", "");
                // }

                if response["headers"] != serde_yaml::Value::Null {
                    let mut s = serde_yaml::to_string(&response["headers"])?;
                    doc.push_str("  headers:");
                    s = s.replace("\n", "\n    ");
                    doc.push_str(&s);
                    doc.push_str("\n");
                    doc = doc.replace("---", "");
                }

            }
        },
        _ => {},
    }
    
    doc = doc.replace("#/definitions/", "");
    doc = doc.replace("$ref", "\"application/json\"");


    write_output(&file, &doc)?;

    Ok(())
}


fn parse_yml(yaml: Yaml) -> String {

    let (path, method, params, _method_value) = (&yaml.path, &yaml.method, &yaml.params, &yaml.method_value);

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

        // nodejs function file destination path
        let mut node_fn_dest = String::new();
        node_fn_dest.push_str("output/");
        node_fn_dest.push_str(&function_file);
        copy_template("node-function.js", &node_fn_dest).expect("Error copying the node function");

        // nodejs function test destination path
        let mut node_test_dest = String::new();
        node_test_dest.push_str(TESTS_BASE_PATH);
        node_test_dest.push_str(&function_name);
        node_test_dest.push_str(".test");
        node_test_dest.push_str(".js");
        let mut test_content = read_template("node-test.js");
        test_content = test_content.replace("[function-name]", &function_file);
        create_file(&node_test_dest).expect("Error creating function test file");
        write_output(&node_test_dest, &test_content).expect("Error writing content to function test file");
        // nodejs function docs destination path
        let mut function_doc_path = String::from(FUNCTION_DOC_BASE_PATH);
        function_doc_path.push_str(&function_name);
        function_doc_path.push_str(".yml");
        create_file(&function_doc_path).expect("Error creating function yaml doc file");
        create_function_docs(yaml, &function_doc_path).expect("Error creating function docs");

        function_doc_path = String::from("docs/functions/");
        function_doc_path.push_str(&function_name);
        function_doc_path.push_str(".yml");
        std_fn = std_fn.replace("[function-doc-path]", &function_doc_path);

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
    fs::create_dir_all(FUNCTION_DOC_BASE_PATH)?;
    fs::create_dir_all(TESTS_BASE_PATH)?;
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

    parse_swagger(&params)?;
    
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