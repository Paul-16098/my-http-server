use crate::{cofg::config::Cofg, parser::md2html, test::support::create_test_dir};
use simple_test_case::test_case;
use std::fs;

#[test_case("base", ":arrow_down:"; "base")]
#[test_case("base-2", r#"\\:arrow_down:"#; "base-2")]
#[test_case("base-3", "> :arrow_down:"; "base-3")]
#[test_case("base-4", r#"\:arrow_down:"#; "base-4")]
// 3 backslashes: Escaped backslash + escaped pattern -> \\:arrow_down:
#[test_case("base-5", r#"\\\:arrow_down:"#; "base-5")]
// Multiple patterns in a single string
#[test_case("base-6", r#":arrow_down: and \:arrow_down:"#; "base-6")]
// Pattern inside inline code block (Markdown shouldn't parse emojis in `code`)
#[test_case("base-7", r#"`:arrow_down:`"#; "base-7")]
#[test]
fn test_md2html(case: &str, md: &str) {
	crate::test::support::init_test_setup();

	let temp_dir = create_test_dir();
	let template_path = temp_dir.path().join("test-template.hbs");

	// Create a minimal template
	fs::write(
		&template_path,
		"<!DOCTYPE html><html>\n<head><title>{{{title}}}</title></head>\n<body>\n{{{body}}}\n</body></html>",
	)
	.expect("Should write template");

	let config = Cofg {
		hbs_path: template_path.to_string_lossy().to_string(),
		templating: crate::cofg::config::CofgTemplating {
			hot_reload: false,
			..Default::default()
		},
		..Cofg::default()
	};

	let html = md2html(md.to_string(), &config, vec![]).unwrap();
	insta::with_settings!({
		description => md,
		omit_expression => true,
	}, {
		insta::assert_snapshot!(format!("test_md2html-emojis-case-{case}"), html, md);
	});
}
