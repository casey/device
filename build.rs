use {
  quote::ToTokens,
  std::{collections::BTreeMap, env, fs, path::PathBuf},
  syn::{FnArg, Item, ItemFn, PatType, ReturnType, Type},
};

// todo:
// - can remove commands tests or put them here
// - require pub(crate) visibility, since will get error otherwise
// - replace _ with -

const PATH: &str = "src/commands.rs";

enum Command {
  App,
  State,
}

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

    if function.sig.inputs.len() != 1 {
      panic!("{}", function.sig.inputs.len());
    }

    if function.sig.output != ReturnType::Default {
      panic!("{}", function.sig.output.to_token_stream());
    }

    let FnArg::Typed(PatType { ty, .. }) = function.sig.inputs.first().unwrap() else {
      panic!("{}", function.sig.inputs.first().unwrap().to_token_stream());
    };

    let Type::Reference(r) = ty.as_ref() else {
      todo!();
    };

    let Type::Path(p) = r.elem.as_ref() else {
      todo!();
    };

    if !p.qself.is_none() {
      todo!();
    }

    let ident = p.path.get_ident().unwrap();

    let command = match ident.to_string().as_str() {
      "State" => "State",
      "App" => "App",
      _ => todo!(),
    };

    commands.insert(name, command);
  }

  let mut lines = Vec::new();

  lines.push("use super::*;".into());

  lines.push("".into());

  for (name, variant) in &commands {
    lines.push(format!(
      "pub(crate) const {}: Foo = Foo::{variant}(commands::{name});",
      name.to_uppercase(),
    ));
  }

  lines.push("pub(crate) const COMMANDS: &[(&'static str, Foo)] = &[".into());

  for (name, variant) in commands {
    lines.push(format!(
      "(\"{}\", {}),",
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
