mod utils;

use std::fs::File;
use std::io::Read;

use clap::Parser;

use rand::thread_rng;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use eyre::{eyre, Context};

use std::io::{self, Cursor};

// Here are a bunch of includes for running the shader
use reflection::PipelineDescription;

#[derive(Parser)]
pub struct Options {
    #[clap(action, default_value = "-")]
    pub input: String,

    #[clap(action, default_value = "-")]
    pub output: String,

    #[clap(action)]
    pub input_data: Option<String>,

    #[clap(short, long, value_parser, default_value_t = 1)]
    pub count: u32,
}

pub fn run(options: Options) -> eyre::Result<()> {
    println!("Adding {} instances of undefined behavior...", options.count);
    println!("Running shader...");
    
    // Grab necessary data from the command execution
    let shader = read_shader_from_path(&options.input)?;
    let input_data = read_input_data(&options.input, options.input_data.as_deref())?;
   
    // Get the pipeline desc to run, and the runner config
    let (pipeline_desc, _) = reflect_shader(&shader, input_data);
    let runner_config = harness::query_configs().into_iter().nth(1).unwrap().id; // For now lets just get the 1st config
   
    // Run the shader and get the output
    
    let run_output = harness::execute_config(&shader, &pipeline_desc, &runner_config).expect("Run Failed");
    let flow_output = u8s_to_u32s(run_output.last().expect("Missing Flow"));
    println!("Flow found adding undefined behavior...");

    // Randomly compute the blocks that we want to have UB
    // We will use the count variable to do this.

    let mut flow_indices = vec![];
    for (pos, &e) in flow_output.iter().enumerate() {
        if e > 0 { flow_indices.push(pos as u32); }
    }
    
    let mut rng = thread_rng();
    flow_indices.shuffle(&mut rng);

    let mut random_indices: Vec<u32> = flow_indices.into_iter().take(options.count.try_into().unwrap()).collect();
    random_indices.sort();

    // Build the AST and pass it into the undefined behaviour generator, along
    // with the randomly generated locations for UB.
    
    let ast = parser::parse(&shader);
    // TODO: Modify the AST here
    let result = ast; // This is a placeholder so that the rest works.
    // Set result to the return value of our traversal

    // Rewrite the AST back to the file for further testing.
    struct Output(Box<dyn std::io::Write>);

    impl std::fmt::Write for Output {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            use std::io::Write;
            self.0.write_all(s.as_bytes()).unwrap();
            Ok(())
        }
    }

    let output: Box<dyn std::io::Write> = match options.output.as_str() {
        "-" => Box::new(std::io::stdout()),
        path => Box::new(File::create(path)?),
    };

    ast::writer::Writer::default()
        .write_module(&mut Output(output), &result)
        .unwrap();
    
    Ok(())
}

fn read_shader_from_path(path: &str) -> eyre::Result<String> {
    let mut input: Box<dyn Read> = match path {
        "-" => Box::new(std::io::stdin()),
        path => Box::new(File::open(path)?),
    };

    let mut shader = String::new();
    input.read_to_string(&mut shader)?;

    Ok(shader)
}

fn read_input_data(
    shader: &str,
    input_data: Option<&str>,
) -> eyre::Result<HashMap<String, Vec<u8>>> {
    match input_data {
        Some(input_data) => {
            match serde_json::from_str(input_data)
                .wrap_err_with(|| eyre!("failed to parse input data"))
            {
                Ok(input_data) => Ok(input_data),
                Err(parse_err) => match File::open(input_data) {
                    Ok(file) => serde_json::from_reader(file)
                        .wrap_err_with(|| eyre!("failed to parse input data")),
                    Err(e) if e.kind() == io::ErrorKind::NotFound => Err(parse_err),
                    Err(e) => Err(e.into()),
                },
            }
        }
        None => {
            if shader != "-" {
                if let Some(path) = Path::new(shader).parent().map(|it| it.join("inputs.json")) {
                    if path.exists() {
                        return Ok(serde_json::from_reader(File::open(path)?)?);
                    }
                }

                let path = Path::new(shader).with_extension("json");
                if path.exists() {
                    return Ok(serde_json::from_reader(File::open(path)?)?);
                }
            }
            Ok(Default::default())
        }
    }
}

fn reflect_shader(
    shader: &str,
    mut input_data: HashMap<String, Vec<u8>>,
) -> (PipelineDescription, Vec<common::Type>) {
    // This is innefficient in this module, since we use the parser twice
    // TODO: Make this more efficient (for now I will leave it the same since it comes from
    // harness stuff
    let module = parser::parse(shader);

    let (mut pipeline_desc, type_descs) = reflection::reflect(&module, |resource| {
        input_data.remove(&format!("{}:{}", resource.group, resource.binding))
    });

    let mut resource_vars = HashSet::new();

    for resource in &pipeline_desc.resources {
        resource_vars.insert(resource.name.clone());
    }

    utils::remove_accessed_vars(&mut resource_vars, &module);

    pipeline_desc
        .resources
        .retain(|resource| !resource_vars.contains(&resource.name));

    (pipeline_desc, type_descs)
}

fn u8s_to_u32s(from: &Vec<u8>) -> Vec<u32> {
    use byteorder::{LittleEndian, ReadBytesExt};
    let mut rdr = Cursor::new(from);
    let mut vec32: Vec<u32> = vec![];
    while let Ok(u) = rdr.read_u32::<LittleEndian>() {
        vec32.push(u);
    }
    vec32
}
