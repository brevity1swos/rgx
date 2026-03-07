pub struct Recipe {
    pub name: &'static str,
    pub pattern: &'static str,
    pub description: &'static str,
    pub test_string: &'static str,
}

pub const RECIPES: &[Recipe] = &[
    Recipe {
        name: "Email address",
        pattern: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
        description: "Match email addresses",
        test_string: "Contact us at hello@example.com or support@company.co.uk",
    },
    Recipe {
        name: "URL (http/https)",
        pattern: r"https?://[^\s/$.?#].[^\s]*",
        description: "Match HTTP and HTTPS URLs",
        test_string: "Visit https://example.com or http://docs.rs/regex/latest",
    },
    Recipe {
        name: "IPv4 address",
        pattern: r"\b(?:\d{1,3}\.){3}\d{1,3}\b",
        description: "Match IPv4 addresses (basic)",
        test_string: "Server at 192.168.1.1 and gateway 10.0.0.1",
    },
    Recipe {
        name: "Date (YYYY-MM-DD)",
        pattern: r"\d{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12]\d|3[01])",
        description: "Match ISO 8601 dates",
        test_string: "Created on 2024-01-15, updated 2024-12-31",
    },
    Recipe {
        name: "Phone number (US)",
        pattern: r"\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}",
        description: "Match US phone numbers in common formats",
        test_string: "Call (555) 123-4567 or 555.987.6543",
    },
    Recipe {
        name: "UUID",
        pattern: r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
        description: "Match UUIDs (v1-v5)",
        test_string: "ID: 550e8400-e29b-41d4-a716-446655440000",
    },
    Recipe {
        name: "Hex color code",
        pattern: r"#(?:[0-9a-fA-F]{3}){1,2}\b",
        description: "Match 3 or 6 digit hex color codes",
        test_string: "Colors: #fff, #FF5733, #a1b2c3",
    },
    Recipe {
        name: "Semantic version",
        pattern: r"\bv?\d+\.\d+\.\d+(?:-[\w.]+)?(?:\+[\w.]+)?\b",
        description: "Match semver versions (e.g., 1.2.3, v0.4.0-beta.1)",
        test_string: "Updated from v1.2.3 to v2.0.0-rc.1+build.42",
    },
    Recipe {
        name: "Log level",
        pattern: r"\b(?:DEBUG|INFO|WARN(?:ING)?|ERROR|FATAL|TRACE)\b",
        description: "Match common log levels",
        test_string: "[ERROR] Connection failed\n[INFO] Server started\n[WARN] Low memory",
    },
    Recipe {
        name: "Key=value pairs",
        pattern: r#"(\w+)=("[^"]*"|\S+)"#,
        description: "Match key=value pairs (quoted or unquoted)",
        test_string: "host=localhost port=8080 name=\"my app\" debug=true",
    },
];
