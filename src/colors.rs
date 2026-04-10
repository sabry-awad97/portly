/// Framework color mapping module
///
/// Provides centralized color mapping for framework names across all display modes.
/// This ensures consistent coloring between table output, details view, and other displays.
///
/// # Color Scheme
///
/// Frameworks are grouped by category with consistent colors:
///
/// ## JavaScript/TypeScript Frameworks
/// - **Cyan**: Next.js, Nuxt, Gatsby, Go
/// - **Bright Magenta**: Vite, Webpack, Parcel
/// - **Blue**: React, Vue, Angular
/// - **Green**: Node.js, Express
///
/// ## Backend Frameworks
/// - **Yellow**: Django, Flask, FastAPI
/// - **Red**: Rails, Ruby
/// - **Bright Blue**: Laravel, Symfony, PHP, Docker
/// - **Green**: Spring
/// - **Bright Cyan**: .NET
///
/// ## Systems Languages
/// - **Bright Red**: Rust, Trunk
/// - **Cyan**: Go
///
/// ## Databases & Services
/// - **Blue**: PostgreSQL, MySQL
/// - **Green**: Redis, MongoDB
/// - **Bright Green**: nginx, RabbitMQ
/// - **Bright Blue**: Docker
///
/// # Adding New Frameworks
///
/// To add a new framework:
///
/// 1. Add the framework name to the appropriate match arm in `get_framework_color()`
/// 2. Choose a color that matches the framework's category
/// 3. Add test cases in the `tests` module
/// 4. Update this documentation
///
/// # Examples
///
/// ```
/// use portly::colors::apply_framework_color;
///
/// // Apply color to framework name
/// let colored = apply_framework_color("Next.js", true);
/// let plain = apply_framework_color("Next.js", false);
/// assert_eq!(plain, "Next.js");
/// ```
use colored::*;

/// Framework color categories
///
/// Maps framework names to terminal colors based on their category:
/// - JavaScript/TypeScript frameworks
/// - Backend frameworks  
/// - Systems languages
/// - Databases & services
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameworkColor {
    Cyan,
    BrightMagenta,
    Blue,
    Green,
    Yellow,
    Red,
    BrightRed,
    BrightBlue,
    BrightCyan,
    BrightGreen,
    Normal,
}

impl FrameworkColor {
    /// Apply this color to a string
    fn apply(&self, text: &str) -> ColoredString {
        match self {
            FrameworkColor::Cyan => text.cyan(),
            FrameworkColor::BrightMagenta => text.bright_magenta(),
            FrameworkColor::Blue => text.blue(),
            FrameworkColor::Green => text.green(),
            FrameworkColor::Yellow => text.yellow(),
            FrameworkColor::Red => text.red(),
            FrameworkColor::BrightRed => text.bright_red(),
            FrameworkColor::BrightBlue => text.bright_blue(),
            FrameworkColor::BrightCyan => text.bright_cyan(),
            FrameworkColor::BrightGreen => text.bright_green(),
            FrameworkColor::Normal => text.normal(),
        }
    }
}

/// Get the color for a framework name
///
/// Returns the appropriate color based on framework category.
/// Unknown frameworks return `FrameworkColor::Normal`.
///
/// # Examples
///
/// ```
/// use portly::colors::{get_framework_color, FrameworkColor};
///
/// assert_eq!(get_framework_color("Next.js"), FrameworkColor::Cyan);
/// assert_eq!(get_framework_color("Django"), FrameworkColor::Yellow);
/// assert_eq!(get_framework_color("Unknown"), FrameworkColor::Normal);
/// ```
pub fn get_framework_color(framework: &str) -> FrameworkColor {
    match framework {
        // JavaScript/TypeScript frameworks
        "Next.js" | "Nuxt" | "Gatsby" => FrameworkColor::Cyan,
        "Vite" | "Webpack" | "Parcel" => FrameworkColor::BrightMagenta,
        "React" | "Vue" | "Angular" => FrameworkColor::Blue,
        "Node.js" | "Express" => FrameworkColor::Green,

        // Backend frameworks
        "Django" | "Flask" | "FastAPI" => FrameworkColor::Yellow,
        "Rails" | "Ruby" => FrameworkColor::Red,
        "Laravel" | "Symfony" | "PHP" => FrameworkColor::BrightBlue,
        "Spring" => FrameworkColor::Green,
        ".NET" => FrameworkColor::BrightCyan,

        // Systems languages
        "Rust" | "Trunk" => FrameworkColor::BrightRed,
        "Go" => FrameworkColor::Cyan,

        // Databases & services
        "PostgreSQL" | "MySQL" => FrameworkColor::Blue,
        "Redis" | "MongoDB" => FrameworkColor::Green,
        "nginx" | "RabbitMQ" => FrameworkColor::BrightGreen,
        "Docker" => FrameworkColor::BrightBlue,

        // Default
        _ => FrameworkColor::Normal,
    }
}

/// Apply color to framework name
///
/// Returns the framework name with color applied if `use_colors` is true,
/// otherwise returns the plain framework name.
///
/// # Examples
///
/// ```
/// use portly::colors::apply_framework_color;
///
/// // With colors enabled
/// let colored = apply_framework_color("Next.js", true);
/// // Returns colored string with ANSI codes
///
/// // With colors disabled
/// let plain = apply_framework_color("Next.js", false);
/// assert_eq!(plain, "Next.js");
/// ```
pub fn apply_framework_color(framework: &str, use_colors: bool) -> String {
    if !use_colors {
        return framework.to_string();
    }

    let color = get_framework_color(framework);
    color.apply(framework).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Phase 1: Framework color mapping tests
    
    #[test]
    fn test_framework_colors_javascript() {
        assert_eq!(get_framework_color("Next.js"), FrameworkColor::Cyan);
        assert_eq!(get_framework_color("Nuxt"), FrameworkColor::Cyan);
        assert_eq!(get_framework_color("Gatsby"), FrameworkColor::Cyan);
        
        assert_eq!(get_framework_color("Vite"), FrameworkColor::BrightMagenta);
        assert_eq!(get_framework_color("Webpack"), FrameworkColor::BrightMagenta);
        assert_eq!(get_framework_color("Parcel"), FrameworkColor::BrightMagenta);
        
        assert_eq!(get_framework_color("React"), FrameworkColor::Blue);
        assert_eq!(get_framework_color("Vue"), FrameworkColor::Blue);
        assert_eq!(get_framework_color("Angular"), FrameworkColor::Blue);
        
        assert_eq!(get_framework_color("Node.js"), FrameworkColor::Green);
        assert_eq!(get_framework_color("Express"), FrameworkColor::Green);
    }

    #[test]
    fn test_framework_colors_backend() {
        assert_eq!(get_framework_color("Django"), FrameworkColor::Yellow);
        assert_eq!(get_framework_color("Flask"), FrameworkColor::Yellow);
        assert_eq!(get_framework_color("FastAPI"), FrameworkColor::Yellow);
        
        assert_eq!(get_framework_color("Rails"), FrameworkColor::Red);
        assert_eq!(get_framework_color("Ruby"), FrameworkColor::Red);
        
        assert_eq!(get_framework_color("Laravel"), FrameworkColor::BrightBlue);
        assert_eq!(get_framework_color("Symfony"), FrameworkColor::BrightBlue);
        assert_eq!(get_framework_color("PHP"), FrameworkColor::BrightBlue);
        
        assert_eq!(get_framework_color("Spring"), FrameworkColor::Green);
        assert_eq!(get_framework_color(".NET"), FrameworkColor::BrightCyan);
    }

    #[test]
    fn test_framework_colors_systems() {
        assert_eq!(get_framework_color("Rust"), FrameworkColor::BrightRed);
        assert_eq!(get_framework_color("Trunk"), FrameworkColor::BrightRed);
        assert_eq!(get_framework_color("Go"), FrameworkColor::Cyan);
    }

    #[test]
    fn test_framework_colors_databases() {
        assert_eq!(get_framework_color("PostgreSQL"), FrameworkColor::Blue);
        assert_eq!(get_framework_color("MySQL"), FrameworkColor::Blue);
        
        assert_eq!(get_framework_color("Redis"), FrameworkColor::Green);
        assert_eq!(get_framework_color("MongoDB"), FrameworkColor::Green);
        
        assert_eq!(get_framework_color("nginx"), FrameworkColor::BrightGreen);
        assert_eq!(get_framework_color("RabbitMQ"), FrameworkColor::BrightGreen);
        
        assert_eq!(get_framework_color("Docker"), FrameworkColor::BrightBlue);
    }

    #[test]
    fn test_framework_colors_unknown() {
        assert_eq!(get_framework_color("Unknown"), FrameworkColor::Normal);
        assert_eq!(get_framework_color("CustomFramework"), FrameworkColor::Normal);
        assert_eq!(get_framework_color(""), FrameworkColor::Normal);
    }

    // Phase 2: Color application tests

    #[test]
    fn test_apply_framework_color_with_colors() {
        // Force enable colors for this test
        colored::control::set_override(true);
        
        let result = apply_framework_color("Next.js", true);
        // Should contain ANSI color codes
        assert!(result.contains("\x1b["));
        assert!(result.contains("Next.js"));
        
        // Reset color override
        colored::control::unset_override();
    }

    #[test]
    fn test_apply_framework_color_without_colors() {
        let result = apply_framework_color("Next.js", false);
        // Should be plain text without ANSI codes
        assert_eq!(result, "Next.js");
        assert!(!result.contains("\x1b["));
    }

    #[test]
    fn test_apply_framework_color_all_categories() {
        colored::control::set_override(true);
        
        let frameworks = vec![
            "Next.js", "Vite", "React", "Node.js",
            "Django", "Rails", "Laravel", "Spring", ".NET",
            "Rust", "Go",
            "PostgreSQL", "Redis", "nginx", "Docker",
            "Unknown"
        ];
        
        for framework in frameworks {
            let colored_result = apply_framework_color(framework, true);
            let plain_result = apply_framework_color(framework, false);
            
            // Colored version should contain ANSI codes (except for "Unknown" which is Normal)
            if framework != "Unknown" {
                assert!(colored_result.contains("\x1b["), 
                    "Framework {} should have color codes", framework);
            }
            
            // Plain version should match framework name exactly
            assert_eq!(plain_result, framework);
        }
        
        colored::control::unset_override();
    }

    #[test]
    fn test_apply_framework_color_empty_string() {
        let result = apply_framework_color("", false);
        assert_eq!(result, "");
        
        let result_colored = apply_framework_color("", true);
        assert_eq!(result_colored, "");
    }

    // Phase 5: Comprehensive edge case tests

    #[test]
    fn test_framework_color_case_sensitivity() {
        // Framework names are case-sensitive
        assert_eq!(get_framework_color("next.js"), FrameworkColor::Normal);
        assert_eq!(get_framework_color("NEXT.JS"), FrameworkColor::Normal);
        assert_eq!(get_framework_color("Next.js"), FrameworkColor::Cyan);
    }

    #[test]
    fn test_framework_color_whitespace() {
        // Whitespace should not match
        assert_eq!(get_framework_color(" Next.js"), FrameworkColor::Normal);
        assert_eq!(get_framework_color("Next.js "), FrameworkColor::Normal);
        assert_eq!(get_framework_color(" Next.js "), FrameworkColor::Normal);
    }

    #[test]
    fn test_apply_framework_color_special_characters() {
        // Test frameworks with special characters
        let frameworks = vec![".NET", "Node.js"];
        
        for framework in frameworks {
            let plain = apply_framework_color(framework, false);
            assert_eq!(plain, framework);
            
            colored::control::set_override(true);
            let colored_result = apply_framework_color(framework, true);
            assert!(colored_result.contains(framework));
            colored::control::unset_override();
        }
    }

    #[test]
    fn test_all_framework_colors_are_valid() {
        // Ensure all frameworks return a valid color
        let all_frameworks = vec![
            // JavaScript/TypeScript
            "Next.js", "Nuxt", "Gatsby",
            "Vite", "Webpack", "Parcel",
            "React", "Vue", "Angular",
            "Node.js", "Express",
            // Backend
            "Django", "Flask", "FastAPI",
            "Rails", "Ruby",
            "Laravel", "Symfony", "PHP",
            "Spring", ".NET",
            // Systems
            "Rust", "Trunk", "Go",
            // Databases & Services
            "PostgreSQL", "MySQL",
            "Redis", "MongoDB",
            "nginx", "RabbitMQ",
            "Docker",
        ];
        
        for framework in all_frameworks {
            let color = get_framework_color(framework);
            // Should not panic and should return a valid color
            assert!(matches!(color, 
                FrameworkColor::Cyan | FrameworkColor::BrightMagenta | 
                FrameworkColor::Blue | FrameworkColor::Green | 
                FrameworkColor::Yellow | FrameworkColor::Red | 
                FrameworkColor::BrightRed | FrameworkColor::BrightBlue | 
                FrameworkColor::BrightCyan | FrameworkColor::BrightGreen | 
                FrameworkColor::Normal
            ));
        }
    }

    #[test]
    fn test_color_consistency_across_modules() {
        // Verify that the same framework always gets the same color
        let frameworks = vec!["Next.js", "Django", "Rust", "PostgreSQL"];
        
        for framework in frameworks {
            let color1 = get_framework_color(framework);
            let color2 = get_framework_color(framework);
            assert_eq!(color1, color2, "Color should be consistent for {}", framework);
        }
    }

    #[test]
    fn test_apply_framework_color_no_ansi_when_disabled() {
        // Ensure no ANSI codes when colors are disabled
        let frameworks = vec!["Next.js", "Django", "Rust", "PostgreSQL", "Unknown"];
        
        for framework in frameworks {
            let result = apply_framework_color(framework, false);
            assert!(!result.contains("\x1b["), 
                "Framework {} should not have ANSI codes when colors disabled", framework);
            assert_eq!(result, framework);
        }
    }

    #[test]
    fn test_framework_color_enum_equality() {
        // Test FrameworkColor enum equality
        assert_eq!(FrameworkColor::Cyan, FrameworkColor::Cyan);
        assert_ne!(FrameworkColor::Cyan, FrameworkColor::Blue);
        
        // Test that same framework always returns same color
        let color1 = get_framework_color("Next.js");
        let color2 = get_framework_color("Next.js");
        assert_eq!(color1, color2);
    }

    #[test]
    fn test_unicode_framework_names() {
        // Test that unicode characters don't cause issues
        let unicode_framework = "框架";
        let result = apply_framework_color(unicode_framework, false);
        assert_eq!(result, unicode_framework);
        
        let result_colored = apply_framework_color(unicode_framework, true);
        assert!(result_colored.contains(unicode_framework));
    }
}
