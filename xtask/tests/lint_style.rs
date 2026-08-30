// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use upac_xtask::lint_style::{
    cargo_toml_dependency_order, cargo_toml_package_order, extern_fn_position, macro_visibility_adjacency,
    no_pub_use_reexport, toml_config_field_order,
};

mod no_pub_use_reexport_rule {
    use super::*;

    #[test]
    fn allows_reexport_of_a_private_module() {
        let contents = "mod foo;\npub use self::foo::Bar;\n";

        let violations = no_pub_use_reexport::check(Path::new("lib.rs"), contents);

        assert!(violations.is_empty());
    }

    #[test]
    fn flags_reexport_of_an_already_pub_module() {
        let contents = "pub mod foo;\npub use self::foo::Bar;\n";

        let violations = no_pub_use_reexport::check(Path::new("lib.rs"), contents);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "no-pub-use-reexport");
        assert_eq!(violations[0].line, 2);
    }
}

mod extern_fn_position_rule {
    use super::*;

    #[test]
    fn allows_extern_fns_before_pub_and_private_fns() {
        let contents = "unsafe extern \"C\" fn a() {}\npub fn b() {}\nfn c() {}\n";

        let violations = extern_fn_position::check(Path::new("lib.rs"), contents);

        assert!(violations.is_empty());
    }

    #[test]
    fn flags_extern_fn_after_pub_fn() {
        let contents = "pub fn a() {}\nunsafe extern \"C\" fn b() {}\n";

        let violations = extern_fn_position::check(Path::new("lib.rs"), contents);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("extern fns go first"));
    }

    #[test]
    fn flags_pub_fn_after_private_fn_when_an_extern_fn_is_present() {
        let contents = "unsafe extern \"C\" fn a() {}\nfn b() {}\npub fn c() {}\n";

        let violations = extern_fn_position::check(Path::new("lib.rs"), contents);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("pub fns go before private ones"));
    }

    #[test]
    fn ignores_pub_private_ordering_when_no_extern_fn_is_present() {
        let contents = "fn a() {}\npub fn b() {}\n";

        let violations = extern_fn_position::check(Path::new("lib.rs"), contents);

        assert!(violations.is_empty());
    }
}

mod macro_visibility_adjacency_rule {
    use super::*;

    #[test]
    fn allows_use_immediately_after_the_macros_closing_brace() {
        let contents = "macro_rules! foo {\n    () => {};\n}\npub(crate) use foo;\n";

        let violations = macro_visibility_adjacency::check(Path::new("lib.rs"), contents);

        assert!(violations.is_empty());
    }

    #[test]
    fn flags_use_separated_from_the_macros_closing_brace() {
        let contents = "macro_rules! foo {\n    () => {};\n}\n\npub(crate) use foo;\n";

        let violations = macro_visibility_adjacency::check(Path::new("lib.rs"), contents);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "macro-visibility-adjacency");
    }
}

mod cargo_toml_dependency_order_rule {
    use super::*;

    #[test]
    fn allows_upac_then_workspace_bracketed_then_bracketed_then_bare() {
        let contents = "[dependencies]\nupac-abi = { workspace = true }\n\nfoo = { workspace = true }\n\nbar = { \
                         version = \"1\", features = [\"x\"] }\nbaz = { version = \"1\" }\n\nqux = \"1\"\n";

        let violations = cargo_toml_dependency_order::check(Path::new("Cargo.toml"), contents);

        assert!(violations.is_empty());
    }

    #[test]
    fn flags_bare_dependency_before_a_bracketed_one() {
        let contents = "[dependencies]\nqux = \"1\"\nbar = { version = \"1\" }\n";

        let violations = cargo_toml_dependency_order::check(Path::new("Cargo.toml"), contents);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("out of its expected group"));
    }

    #[test]
    fn flags_ascending_key_count_among_bracketed_dependencies() {
        let contents =
            "[dependencies]\nbar = { version = \"1\" }\nbaz = { version = \"1\", features = [\"x\"] }\n";

        let violations = cargo_toml_dependency_order::check(Path::new("Cargo.toml"), contents);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("descending key count"));
    }
}

mod cargo_toml_package_order_rule {
    use super::*;

    #[test]
    fn allows_workspace_fields_before_a_trailing_custom_override() {
        let contents = "[package]\nname = \"foo\"\nversion.workspace = true\n\nreadme.workspace = true\n\nlicense \
                         = \"GPL-3.0-only\"\n";

        let violations = cargo_toml_package_order::check(Path::new("Cargo.toml"), contents);

        assert!(violations.is_empty());
    }

    #[test]
    fn flags_name_not_being_the_first_field() {
        let contents = "[package]\nversion.workspace = true\nname = \"foo\"\n";

        let violations = cargo_toml_package_order::check(Path::new("Cargo.toml"), contents);

        assert!(violations.iter().any(|violation| violation.message.contains("must be the first field")));
    }

    #[test]
    fn flags_workspace_field_after_a_custom_override() {
        let contents = "[package]\nname = \"foo\"\nlicense = \"GPL-3.0-only\"\nreadme.workspace = true\n";

        let violations = cargo_toml_package_order::check(Path::new("Cargo.toml"), contents);

        assert!(violations.iter().any(|violation| violation.message.contains("workspace fields go first")));
    }
}

mod toml_config_field_order_rule {
    use super::*;

    #[test]
    fn allows_bool_then_string_then_number() {
        let contents = "[section]\nflag = true\nname = \"foo\"\ncount = 1\n";

        let violations = toml_config_field_order::check(Path::new("lib.toml"), contents);

        assert!(violations.is_empty());
    }

    #[test]
    fn flags_bool_after_string() {
        let contents = "[section]\nname = \"foo\"\nflag = true\n";

        let violations = toml_config_field_order::check(Path::new("lib.toml"), contents);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("bool"));
    }

    #[test]
    fn resets_ordering_per_section() {
        let contents = "[a]\ncount = 1\n\n[b]\nflag = true\n";

        let violations = toml_config_field_order::check(Path::new("lib.toml"), contents);

        assert!(violations.is_empty());
    }
}
