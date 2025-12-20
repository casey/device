use {
  quote::ToTokens,
  std::{collections::BTreeMap, env, fs, path::PathBuf},
  syn::{FnArg, Item, ItemFn, Pat, PatType, ReturnType, Type},
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

    let inputs = function
      .sig
      .inputs
      .iter()
      .map(|input| {
        let FnArg::Typed(PatType { pat, .. }) = input else {
          panic!(
            "command function {name} has self receiver: {}",
            input.to_token_stream(),
          );
        };

        let Pat::Ident(pat_ident) = pat.as_ref() else {
          panic!(
            "command function {name} has input without ident pattern: {}",
            input.to_token_stream(),
          );
        };

        pat_ident.ident.to_string()
      })
      .collect::<Vec<String>>();

    let fallible = match &function.sig.output {
      ReturnType::Default => false,
      ReturnType::Type(_, ty) => {
        let Type::Path(p) = ty.as_ref() else {
          panic!(
            "command function {name} has unexpected return type: {}",
            ty.to_token_stream(),
          );
        };

        assert!(
          p.qself.is_none(),
          "command function {name} has qualified return type",
        );

        let ident = p.path.get_ident().unwrap();

        assert_eq!(
          ident,
          "Result",
          "command function {name} has unexpected return type: {}",
          ty.to_token_stream(),
        );

        true
      }
    };

    let inputs = inputs.iter().map(String::as_str).collect::<Vec<&str>>();

    let variant = match (inputs.as_slice(), fallible) {
      (["app", "event_loop"], false) => "AppEventLoop",
      (["app"], false) => "App",
      (["app"], true) => "AppFallible",
      (["history", "state"], false) => "HistoryState",
      (["history"], false) => "History",
      (["state"], false) => "State",
      _ => panic!(
        "unsupported combination of inputs and fallibility: ({}, {fallible})",
        inputs.join(" ")
      ),
    };

    commands.insert(name, variant);
  }

  let mut lines = Vec::new();

  lines.push("use {super::*, commands::*, Command::*};".into());

  lines.push(String::new());

  for (name, variant) in &commands {
    lines.push(format!(
      "pub(crate) const {}: (&str, Command) = (\"{}\", {variant}({name}));",
      name.to_uppercase(),
      name.replace('_', "-"),
    ));
  }

  lines.push(String::new());

  lines.push("pub(crate) const COMMANDS: &[(&str, Command)] = &[".into());

  for name in commands.keys() {
    lines.push(format!("  {},", name.to_uppercase()));
  }

  lines.push("];\n".into());

  fs::write(
    PathBuf::from(env::var("OUT_DIR").unwrap()).join("generated.rs"),
    lines.join("\n"),
  )
  .unwrap();
}
