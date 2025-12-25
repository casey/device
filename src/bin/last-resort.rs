use {
  anyhow::Result,
  regex::Regex,
  std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    fs,
  },
  ucd_parse::{UcdFile, UnicodeData},
};

struct LastResort<'a> {
  glyphs: Vec<&'a str>,
}

impl Display for LastResort<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    writeln!(
      f,
      r#"<?xml version="1.0" encoding="UTF-8"?>
<ttFont sfntVersion="\x00\x01\x00\x00" ttLibVersion="4.33">

  <cmap>
    <tableVersion version="0"/>
    <cmap_format_13 platformID="0" platEncID="6" format="13" reserved="0" length="18760" language="0" nGroups="1562">"#
    )?;

    for (i, name) in self.glyphs.iter().enumerate() {
      let codepoint = 0xE000 + i;
      writeln!(f, r#"      <map code="0x{codepoint:x}" name="{name}"/>"#)?;
    }

    writeln!(
      f,
      r#"    </cmap_format_13>
    <cmap_format_4 platformID="3" platEncID="1" language="0">
    </cmap_format_4>
  </cmap>

</ttFont>"#
    )?;

    Ok(())
  }
}

fn main() -> Result<()> {
  let mut categories = BTreeMap::<char, String>::new();

  for record in UnicodeData::from_dir("static/ucd")? {
    let record = record?;

    let Some(c) = record.codepoint.scalar() else {
      continue;
    };

    categories.insert(c, record.general_category);
  }

  let ttx = fs::read_to_string("static/last-resort-font/cmap-f13.ttx")?;

  let mut glyphs = BTreeMap::<&str, char>::new();

  let re = Regex::new(r#"<map code="0x(?<codepoint>[^"]+)" name="(?<glyph>[^"]+)"/>"#).unwrap();

  for captures in re.captures_iter(&ttx) {
    let codepoint = captures.name("codepoint").unwrap().as_str();

    let codepoint = u32::from_str_radix(codepoint, 16).unwrap();

    let Ok(c) = char::try_from(codepoint) else {
      continue;
    };

    glyphs
      .entry(captures.name("glyph").unwrap().as_str())
      .and_modify(|current| *current = (*current).min(c))
      .or_insert(c);
  }

  fs::write(
    "static/last-resort.ttx",
    LastResort {
      glyphs: glyphs
        .into_iter()
        .map(|(name, c)| (c, name))
        .collect::<BTreeMap<char, &str>>()
        .into_values()
        .collect(),
    }
    .to_string(),
  )?;

  Ok(())
}
