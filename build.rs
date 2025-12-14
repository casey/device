use {
  quote::ToTokens,
  std::{collections::BTreeMap, env, fs, path::PathBuf},
  syn::{FnArg, Item, ItemFn, PatType, ReturnType, Type},
};

const PATH: &str = "src/commands.rs";

fn main() {
  println!("cargo:rerun-if-changed={PATH}");

  let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

  let path = root.join(PATH);

  let src = fs::read_to_string(&path).unwrap();

  let ast = syn::parse_file(&src).unwrap();

  let functions: Vec<&ItemFn> = ast
    .items
    .iter()
    .filter_map(|item| {
      if let Item::Fn(function) = item {
        Some(function)
      } else {
        None
      }
    })
    .collect();

  let mut commands = BTreeMap::new();

  for function in functions {
    let name = function.sig.ident.to_string();

    assert_eq!(
      function.sig.inputs.len(),
      1,
      "command function {name} has incorrect number of parameters: {}",
      function.sig.inputs.len(),
    );

    assert_eq!(
      function.sig.output,
      ReturnType::Default,
      "command function {name} has non-default return type: {}",
      function.sig.output.to_token_stream(),
    );

    let FnArg::Typed(PatType { ty, .. }) = function.sig.inputs.first().unwrap() else {
      panic!(
        "command function {name} has self receiver: {}",
        function.sig.inputs.first().unwrap().to_token_stream(),
      );
    };

    let Type::Reference(r) = ty.as_ref() else {
      panic!(
        "command function {name} has non-reference argument: {}",
        ty.to_token_stream(),
      );
    };

    let Type::Path(p) = r.elem.as_ref() else {
      panic!(
        "command function {name} has non-path argument: {}",
        r.elem.to_token_stream(),
      );
    };

    if p.qself.is_some() {
      panic!("command function {name} has qualified argument");
    }

    let ident = p.path.get_ident().unwrap();

    let command = match ident.to_string().as_str() {
      "State" => "State",
      "App" => "App",
      other => panic!("command {name} has unexpected argument type {other}"),
    };

    commands.insert(name, command);
  }

  let mut lines = Vec::new();

  lines.push("use {super::*, commands::*, Command::*};".into());

  lines.push(String::new());

  for (name, variant) in &commands {
    lines.push(format!(
      "pub(crate) const {}: Command = {variant}({name});",
      name.to_uppercase(),
    ));
  }

  lines.push(String::new());

  lines.push("pub(crate) const COMMANDS: &[(&str, Command)] = &[".into());

  for name in commands.keys() {
    lines.push(format!(
      "  (\"{}\", {}),",
      name.replace('_', "-"),
      name.to_uppercase(),
    ));
  }

  lines.push("];\n".into());

  fs::write(
    PathBuf::from(env::var("OUT_DIR").unwrap()).join("generated.rs"),
    lines.join("\n"),
  )
  .unwrap();
}
