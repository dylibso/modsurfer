use std::fmt::Write;

use anyhow::Error;

use colored::Colorize;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(transparent)]
pub struct Diff(String);

impl std::fmt::Display for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Diff {
    pub fn new(
        a: &modsurfer_module::Module,
        b: &modsurfer_module::Module,
        color_term: bool,
        with_context: bool,
    ) -> Result<Self, Error> {
        let a_string = serde_yaml::to_string(&crate::generate_checkfile(a)?.validate)?;
        let b_string = serde_yaml::to_string(&crate::generate_checkfile(b)?.validate)?;
        let diff = similar::TextDiff::from_lines(a_string.as_str(), b_string.as_str());

        let mut output = String::new();
        let mut changes = 0;
        for change in diff.iter_all_changes() {
            let (sign, color) = match change.tag() {
                similar::ChangeTag::Delete => {
                    changes += 1;
                    ("- ", "red")
                }
                similar::ChangeTag::Insert => {
                    changes += 1;
                    ("+ ", "green")
                }
                similar::ChangeTag::Equal if with_context => ("  ", ""),
                _ => continue,
            };

            if color_term {
                write!(
                    &mut output,
                    "{}{}",
                    sign.color(color),
                    change.as_str().unwrap_or_default().color(color)
                )?;
            } else {
                write!(&mut output, "{}{}", sign, change)?;
            }
        }

        if changes == 0 {
            return Ok(Diff(String::new()));
        }

        Ok(Diff(output))
    }
}

impl From<Diff> for String {
    fn from(x: Diff) -> String {
        x.0
    }
}

impl AsRef<str> for Diff {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
