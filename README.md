# FerricCP

An incredibly fast static analyzer for various languages to spot tiny errors for Competititve Programming (CP) to prevent WA, Runtime Errors and endless hours of detective work to find that one error/bug causing these issues.  
FerricCP is meant to locate various common bugs that unintentionally happen while solving CP problems, so that you spend more time on the actual logic and implementation instead of scratching yourself bald by scanning each line and dry running your code.  

Currently Supported Languages: C++ (Both Semantically and Syntactically), Python (Syntactically).  
Built in Rust and powered by Tree-sitter.

## The Problem
In competitive programming, we unintentionally make very tiny mistakes that result in Wrong Answers, hair loss and cortisol spike :(. Standard compilers like GCC will compile perfectly if you use an uninitialized variable for vector size, or if an implicit type conversion causes an integer overflow. FerricCP acts as a CP-specific frontend to catch these tiny logical and/or non-logical bugs, leading to satisfying ACs :D

## Quick Start

### Installation
Ensure you have [Rust and Cargo](https://rustup.rs/) installed. Clone the repository and build the release binary:

```
git clone https://github.com/xDestructo/FerricCP.git
cd FerricCP
cargo build --release
```

### Usage
Run the analyzer against your source code. The engine will automatically load the YAML rules and execute the checks.
```
cargo run -- -help // to print help
cargo run --release -- -f path/to/your/file.cpp
```
Example Output:
```
WARNING: Variable 'n' used as array bound but is completely uninitialized. (ERROR)
Rule: uninitialized_arr_bound
Tip: Initialize 'n' (e.g., cin >> n) before using it to size a container.
Location: Line 13, Column 5
Code: `vector<int> arr(n);`
```

## Why This Isn't Just Another Linter

FerricCP isn't a simple regex script. It is powered by a **Hybrid Query-Driven Semantic Analyzer**, an enterprise-grade architecture that I independently derived and architected specifically for the competitive programming niche. 

To maintain $O(1)$ state lookups and sub-millisecond execution times (ensuring it never slows down your contest workflow), the engine enforces a strict separation of concerns between data (YAML), event routing (The Dispatcher), and semantic logic (Rust Traits).

Here are just a few points of the system's architecture:

* **Language-Agnostic Core:** Because the engine relies on Tree-sitter's universal C-bindings, the Rust backend is completely detached from any specific language. Whether parsing C++, Python, Rust or any x language, the engine uniformly processes AST nodes. Supporting a new language requires zero changes to the core execution loop.
* **Polymorphic State Tracking:** The internal `SymbolTable` is designed using trait-based Object-Oriented principles. It provides a universal, language-agnostic base for memory and state tracking. You only need to plug in the specific behavioral logic for a new language, and the engine's state lookups work instantly.
* **The Open/Closed Principle:** The core event router (The Dispatcher) is entirely blind to the business logic. You can scale the engine infinitely by adding new traps and semantic rules, without ever modifying or risking the stability of the central parser.
* **Template-Driven Diagnostics:** The Rust binary is completely agnostic to diagnostic text. All error messages, tips, and severities are dynamically injected directly from the YAML config. You can rewrite the entire tool's feedback without recompiling the codebase.
* **C-Backed AST Offloading:** By utilizing Tree-sitter’s `QueryCursor`, the heavy lifting of searching the syntax tree is offloaded to a highly optimized C library, leaving Rust to handle the pure mathematical state validation.

**[Read the deep dive into the Systems Architecture, Compiler Theory implementation and Hybrid Engine in ARCHITECTURE.md](./ARCHITECTURE.md)**

## Adding a Custom Rule

Because of the modular architecture, adding a new rule takes minutes.

**1. Define the syntax trigger in the YAML rules (`rules/{your_language}/{rule_name.yaml}`):**
```yaml
id: ban_goto
language: cpp
rule_type: syntactic
severity: critical
message: "Use of 'goto' statement is banned."
tip: "Refactor this control flow using loops or early returns."
query: |
  (goto_statement) @violation
```
- **Id**: any unique string you want to provide to your rule.  
- **Language**: the language which the rule is for.  
- **Rule_Type**: either Semantic or Syntactic.  
- **Severity**: currently not implemented, you can write anything.  
- **Message**: A brief text describing the rule.
- **Tip**: Is an <Option> enum, i.e., it isn't required, you can leave it blank or - include a brief tip on how to fix the violation.  
- **Query**: It follows the [Tree-Sitter query syntax](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/index.html). You can use the [Tree-Sitter playground](https://tree-sitter.github.io/tree-sitter/7-playground.html) to find out the nodes from the AST which you want to query.

2. **If your rule_type is semantic, register the semantic checker in `src/rules.rs`**
```rust
pub struct ArrayBoundRule;

impl SemanticRule for ArrayBoundRule {
    fn check(&self, node: Node, source: &[u8], table: &SymbolTable, rule: &RuleConfig) -> Option<Diagnostic> {
        // Your O(1) logic here
    }
}
```

## Contributing

Tired of getting WAs because of a specific edge case? Maybe your language isn't implemented yet? Add it to the engine! Pull requests for new rules, C++ traps, Python checks, all are always welcome.
```bash
Fork the Project
Create your Feature Branch (git checkout -b feature/AmazingRule)
Commit your Changes (git commit -m 'Add some AmazingRule')
Push to the Branch (git push origin feature/AmazingRule)
Open a Pull Request
```

## Upcoming Features

### C++
- More syntactic rules
- More semantic rules and their correspoding `rules.rs` implementation
- More features added to the symbol table
### Python
- More syntactic rules
- Add semantic rules and their correspoding `rules.rs` implementation
- Implement the symbol table
### Other
- Add CLI prettier
- Add comments where required
- Clean up yaml rules
- Clean up `main.rs`
- Add support for more languages